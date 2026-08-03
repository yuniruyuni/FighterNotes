import type {
  AnalysisProgress,
  FrameSample,
  SpatialCandidateWindow,
  SpatialFrameHints,
  VideoCodecConfig,
} from "../../domain/result.js";
import type { BlobSampleReaderStats } from "../video-decoding/blob-sample-reader.js";
import { decodeSampleRange } from "../video-decoding/webcodecs-frame-decoder.js";
import {
  EMPTY_SPATIAL_DECODE_STATS,
  SPATIAL_DECODER_OUTSTANDING_WATERMARKS,
  SPATIAL_DECODER_QUEUE_WATERMARKS,
  type SpatialDecodeStats,
} from "./backpressure.js";
import { SPATIAL_HEIGHT, SPATIAL_WIDTH } from "./layout.js";
import { SpatialDecodePlan, spatialHintsAt } from "./spatial-decode-plan.js";

export async function decodeSpatialWindows(options: {
  readonly windows: readonly SpatialCandidateWindow[];
  readonly sampleData: readonly FrameSample[];
  readonly frameToSampleIdx: readonly number[];
  readonly videoFile: Blob;
  readonly codecConfig: VideoCodecConfig;
  readonly resetWindow: () => Promise<void>;
  readonly sendFrame: (
    frameIndex: number,
    createRgbaBuffer: () => ArrayBuffer,
    hints: SpatialFrameHints,
    signal: AbortSignal,
  ) => Promise<void>;
  readonly drain: () => Promise<void>;
  readonly onProgress: AnalysisProgress;
  readonly signal: AbortSignal;
}): Promise<SpatialDecodeStats> {
  const canvas = new OffscreenCanvas(SPATIAL_WIDTH, SPATIAL_HEIGHT);
  const context = canvas.getContext("2d", {
    willReadFrequently: true,
  }) as OffscreenCanvasRenderingContext2D;
  const totalFrames = options.windows.reduce(
    (sum, window) => sum + window.end_frame - window.start_frame + 1,
    0,
  );
  let processedFrames = 0;
  let peakDecoderQueueSize = 0;
  let peakDecoderOutstandingFrames = 0;
  const sampleReads = emptySampleReadStats();

  for (const window of options.windows) {
    throwIfAborted(options.signal);
    const plan = SpatialDecodePlan.create(
      window,
      options.sampleData,
      options.frameToSampleIdx,
    );
    if (!plan) continue;

    await options.resetWindow();
    const targets = new Map(
      plan.targets.map((target) => [target.timestampUs, target.frameIndex]),
    );
    const decodeStats = await decodeSampleRange({
      samples: options.sampleData,
      videoBlob: options.videoFile,
      codecConfig: options.codecConfig,
      firstSampleIndex: plan.firstSampleIndex,
      lastSampleIndex: plan.lastSampleIndex,
      signal: options.signal,
      backpressure: {
        queueHighWatermark: SPATIAL_DECODER_QUEUE_WATERMARKS.high,
        queueLowWatermark: SPATIAL_DECODER_QUEUE_WATERMARKS.low,
        outstandingHighWatermark: SPATIAL_DECODER_OUTSTANDING_WATERMARKS.high,
        outstandingLowWatermark: SPATIAL_DECODER_OUTSTANDING_WATERMARKS.low,
      },
      shouldProcessFrame: (frame) => targets.has(frame.timestamp),
      async onFrame(frame, processingSignal) {
        throwIfAborted(processingSignal);
        const frameIndex = targets.get(frame.timestamp);
        if (frameIndex === undefined) return;
        await options.sendFrame(
          frameIndex,
          () => {
            throwIfAborted(processingSignal);
            context.drawImage(frame, 0, 0, SPATIAL_WIDTH, SPATIAL_HEIGHT);
            return context.getImageData(0, 0, SPATIAL_WIDTH, SPATIAL_HEIGHT)
              .data.buffer;
          },
          spatialHintsAt(window, frameIndex),
          processingSignal,
        );
        processedFrames += 1;
        options.onProgress(
          0.9 + (0.09 * processedFrames) / Math.max(1, totalFrames),
          `位置関係 ${processedFrames} / ${totalFrames}`,
        );
      },
    });
    peakDecoderQueueSize = Math.max(
      peakDecoderQueueSize,
      decodeStats.peakDecoderQueueSize,
    );
    peakDecoderOutstandingFrames = Math.max(
      peakDecoderOutstandingFrames,
      decodeStats.peakDecoderOutstandingFrames,
    );
    accumulateSampleReadStats(sampleReads, decodeStats.sampleReads);
    await options.drain();
  }
  if (sampleReads.readCount > 0) {
    console.log(
      `[perf] spatial-blob reads=${sampleReads.readCount} read=${sampleReads.totalBytesRead} peak_blob=${sampleReads.peakReadBytes} peak_cache=${sampleReads.peakCacheBytes} peak_spatial_retained=${sampleReads.peakRetainedBytes} cache_hits=${sampleReads.cacheHits} cache_misses=${sampleReads.cacheMisses}`,
    );
  }
  return {
    ...EMPTY_SPATIAL_DECODE_STATS,
    peakDecoderQueueSize,
    peakDecoderOutstandingFrames,
  };
}

function emptySampleReadStats(): MutableSampleReadStats {
  return {
    readCount: 0,
    totalBytesRead: 0,
    peakReadBytes: 0,
    cacheHits: 0,
    cacheMisses: 0,
    peakCacheBytes: 0,
    peakRetainedBytes: 0,
  };
}

interface MutableSampleReadStats {
  readCount: number;
  totalBytesRead: number;
  peakReadBytes: number;
  cacheHits: number;
  cacheMisses: number;
  peakCacheBytes: number;
  peakRetainedBytes: number;
}

function accumulateSampleReadStats(
  total: MutableSampleReadStats,
  next: BlobSampleReaderStats,
): void {
  total.readCount += next.readCount;
  total.totalBytesRead += next.totalBytesRead;
  total.peakReadBytes = Math.max(total.peakReadBytes, next.peakReadBytes);
  total.cacheHits += next.cacheHits;
  total.cacheMisses += next.cacheMisses;
  total.peakCacheBytes = Math.max(total.peakCacheBytes, next.peakCacheBytes);
  total.peakRetainedBytes = Math.max(
    total.peakRetainedBytes,
    next.peakRetainedBytes,
  );
}

function throwIfAborted(signal: AbortSignal): void {
  if (!signal.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new Error("動画解析を中断しました");
}
