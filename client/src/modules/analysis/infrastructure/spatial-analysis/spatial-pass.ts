import type {
  AnalysisProgress,
  FrameSample,
  SpatialCandidateWindow,
  SpatialFrameHints,
  VideoCodecConfig,
} from "../../domain/result.js";
import { decodeSampleRange } from "../video-decoding/webcodecs-frame-decoder.js";
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
  ) => void;
  readonly drain: () => Promise<void>;
  readonly onProgress: AnalysisProgress;
  readonly signal: AbortSignal;
}): Promise<void> {
  const canvas = new OffscreenCanvas(SPATIAL_WIDTH, SPATIAL_HEIGHT);
  const context = canvas.getContext("2d", {
    willReadFrequently: true,
  }) as OffscreenCanvasRenderingContext2D;
  const totalFrames = options.windows.reduce(
    (sum, window) => sum + window.end_frame - window.start_frame + 1,
    0,
  );
  let processedFrames = 0;

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
    await decodeSampleRange({
      samples: options.sampleData,
      videoArrayBuffer: options.videoArrayBuffer,
      codecConfig: options.codecConfig,
      firstSampleIndex: plan.firstSampleIndex,
      lastSampleIndex: plan.lastSampleIndex,
      signal: options.signal,
      onFrame(frame) {
        if (options.signal.aborted) {
          frame.close();
          return;
        }
        const frameIndex = targets.get(frame.timestamp);
        if (frameIndex === undefined) {
          frame.close();
          return;
        }
        context.drawImage(frame, 0, 0, SPATIAL_WIDTH, SPATIAL_HEIGHT);
        frame.close();
        const rgbaBuf = context.getImageData(
          0,
          0,
          SPATIAL_WIDTH,
          SPATIAL_HEIGHT,
        ).data.buffer;
        options.sendFrame(
          frameIndex,
          rgbaBuf,
          spatialHintsAt(window, frameIndex),
        );
        processedFrames += 1;
        options.onProgress(
          0.9 + (0.09 * processedFrames) / Math.max(1, totalFrames),
          `位置関係 ${processedFrames} / ${totalFrames}`,
        );
      },
    });
    await options.drain();
  }
}

function throwIfAborted(signal: AbortSignal): void {
  if (!signal.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new Error("動画解析を中断しました");
}
