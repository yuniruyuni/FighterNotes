import type {
  AnalysisProgress,
  FrameSample,
  SpatialCandidateWindow,
  SpatialFrameHints,
  VideoCodecConfig,
} from "../../domain/result.js";
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
  readonly videoArrayBuffer: ArrayBuffer;
  readonly codecConfig: VideoCodecConfig;
  readonly resetWindow: () => Promise<void>;
  readonly sendFrame: (
    frameIndex: number,
    rgbaBuf: ArrayBuffer,
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
      videoArrayBuffer: options.videoArrayBuffer,
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
      async onFrame(frame, processingSignal) {
        try {
          throwIfAborted(processingSignal);
          const frameIndex = targets.get(frame.timestamp);
          if (frameIndex === undefined) return;
          context.drawImage(frame, 0, 0, SPATIAL_WIDTH, SPATIAL_HEIGHT);
          const rgbaBuf = context.getImageData(
            0,
            0,
            SPATIAL_WIDTH,
            SPATIAL_HEIGHT,
          ).data.buffer;
          await options.sendFrame(
            frameIndex,
            rgbaBuf,
            spatialHintsAt(window, frameIndex),
            processingSignal,
          );
          processedFrames += 1;
          options.onProgress(
            0.9 + (0.09 * processedFrames) / Math.max(1, totalFrames),
            `位置関係 ${processedFrames} / ${totalFrames}`,
          );
        } finally {
          frame.close();
        }
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
    await options.drain();
  }
  return {
    ...EMPTY_SPATIAL_DECODE_STATS,
    peakDecoderQueueSize,
    peakDecoderOutstandingFrames,
  };
}

function throwIfAborted(signal: AbortSignal): void {
  if (!signal.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new Error("動画解析を中断しました");
}
