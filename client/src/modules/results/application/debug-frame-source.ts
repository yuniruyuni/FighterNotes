import type {
  FrameSample,
  VideoCodecConfig,
} from "~/modules/analysis/contracts.js";

export interface DebugFrameSourceData {
  file: File;
  frameTimestamps: readonly number[];
  sampleData: FrameSample[] | null;
  videoArrayBuffer: ArrayBuffer | null;
  codecConfig: VideoCodecConfig | null;
  frameToSampleIndex: number[] | null;
}

export interface DebugFrameSource {
  readonly fallbackSource: CanvasImageSource;
  readonly usesExactFrames: boolean;
  initialize(): Promise<void>;
  decode(index: number): Promise<VideoFrame | null>;
  seekFallback(index: number): void;
  destroy(): void;
}

export interface DebugFrameSourceFactory {
  create(
    data: DebugFrameSourceData,
    onFallbackFrame: () => void,
  ): DebugFrameSource;
}
