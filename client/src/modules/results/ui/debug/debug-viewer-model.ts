import type {
  FrameSample,
  HpFrameData,
  RustTimeline,
  TrackedInputRow,
  VideoCodecConfig,
} from "~/modules/analysis/contracts.js";

export interface DebugViewerData {
  file: File;
  timeline: RustTimeline;
  hpFeatures: HpFrameData[];
  trackedInputs: { p1: TrackedInputRow[]; p2: TrackedInputRow[] } | null;
  frameCount: number;
  frameTimestamps: number[];
  sampleData: FrameSample[] | null;
  videoArrayBuffer: ArrayBuffer | null;
  codecConfig: VideoCodecConfig | null;
  frameToSampleIndex: number[] | null;
}

export interface DebugOverlayVisibility {
  raw: boolean;
  hue: boolean;
  hp: boolean;
  drive: boolean;
  input: boolean;
}

export function initialDebugOverlayVisibility(): DebugOverlayVisibility {
  return {
    raw: false,
    hue: false,
    hp: false,
    drive: false,
    input: false,
  };
}
