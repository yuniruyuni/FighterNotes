import type { AnalysisContext } from "./context.js";
import type { AdviceReport } from "./report.js";
import type { RustTimeline } from "./timeline.js";

export type AnalysisProgress = (progress: number, message: string) => void;

/** Encoded frame metadata retained for exact WebCodecs seeking. */
export interface FrameSample {
  readonly isSync: boolean;
  readonly timestampUs: number;
  readonly offset: number;
  readonly size: number;
}

export interface VideoCodecConfig {
  readonly codec: string;
  readonly width: number;
  readonly height: number;
  readonly description?: Uint8Array;
}

export interface SpatialHintRange {
  readonly side: number;
  readonly start_frame: number;
  readonly end_frame: number;
}

export interface SpatialCandidateWindow {
  readonly start_frame: number;
  readonly end_frame: number;
  readonly teleport_hints: readonly SpatialHintRange[];
  readonly airborne_hints: readonly SpatialHintRange[];
}

export interface SpatialFrameHints {
  readonly p1Teleport: boolean;
  readonly p2Teleport: boolean;
  readonly p1Airborne: boolean;
  readonly p2Airborne: boolean;
}

export interface TrackedInputRow {
  readonly count: number | null;
  readonly dir: string;
  readonly badges: string;
  readonly auto: boolean;
  readonly throw: boolean;
  readonly repaired: boolean;
  readonly uncertain: boolean;
}

export interface HpFrameData {
  readonly frame_index: number;
  readonly fps: number;
  /** -1 means the value is uncertain; otherwise this is a 0..1 fill ratio. */
  readonly own_hp: number;
  readonly opponent_hp: number;
  readonly is_match_screen: boolean;
  readonly left_hp_score: number;
  readonly right_hp_score: number;
  readonly left_drive_ratio: number;
  readonly right_drive_ratio: number;
  readonly left_burnout: boolean;
  readonly right_burnout: boolean;
  readonly left_drive_uncertain: boolean;
  readonly right_drive_uncertain: boolean;
  readonly left_hp_raw: number;
  readonly right_hp_raw: number;
}

export interface AnalysisResult {
  readonly analysisContext: AnalysisContext;
  readonly report: AdviceReport;
  readonly timeline: RustTimeline;
  readonly trackedInputs: {
    readonly p1: TrackedInputRow[];
    readonly p2: TrackedInputRow[];
  } | null;
  readonly hpFeatures: HpFrameData[];
  readonly frameCount: number;
  readonly frameTimestamps: number[];
  readonly sampleData: FrameSample[] | null;
  readonly videoArrayBuffer: ArrayBuffer | null;
  readonly codecConfig: VideoCodecConfig | null;
  readonly frameToSampleIdx: number[] | null;
  readonly spatialObservations: object[];
}
