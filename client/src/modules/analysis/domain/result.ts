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

export interface SpatialFrameRange {
  readonly start_frame: number;
  readonly end_frame: number;
}

export interface SpatialCandidateWindow {
  readonly start_frame: number;
  readonly end_frame: number;
  readonly teleport_hints: readonly SpatialHintRange[];
  readonly airborne_hints: readonly SpatialHintRange[];
  readonly contact_hints: readonly SpatialFrameRange[];
  readonly certain_side_hints: readonly SpatialFrameRange[];
}

export interface SpatialFrameHints {
  readonly p1Teleport: boolean;
  readonly p2Teleport: boolean;
  readonly p1Airborne: boolean;
  readonly p2Airborne: boolean;
  /** 第一段が確定した contact(hitstop)区間。スパーク検出を許可する。 */
  readonly contact: boolean;
  /** round 開始直後で側が確定しており、色シグネチャの学習に使える。 */
  readonly sidesCertain: boolean;
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

export type AttackAttribute = "upper" | "middle" | "lower" | "throw";

export interface AttackInfoSide {
  readonly last_damage: number;
  readonly scaling_percent: number;
  readonly combo_damage: number;
  readonly max_combo_damage: number;
  readonly attribute: AttackAttribute;
}

export interface AttackInfoObservation {
  readonly frame_index: number;
  readonly p1: AttackInfoSide;
  readonly p2: AttackInfoSide;
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
  readonly left_super_value: number;
  readonly right_super_value: number;
  readonly left_super_uncertain: boolean;
  readonly right_super_uncertain: boolean;
  readonly left_ca_ready: boolean;
  readonly right_ca_ready: boolean;
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
  readonly attackInfo: AttackInfoObservation[];
  readonly hpFeatures: HpFrameData[];
  readonly frameCount: number;
  readonly frameTimestamps: number[];
  readonly sampleData: FrameSample[] | null;
  readonly videoArrayBuffer: ArrayBuffer | null;
  readonly codecConfig: VideoCodecConfig | null;
  readonly frameToSampleIdx: number[] | null;
  readonly spatialObservations: object[];
}
