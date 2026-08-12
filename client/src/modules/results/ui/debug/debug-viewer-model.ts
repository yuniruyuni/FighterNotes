import type {
  AttackInfoObservation,
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
  attackInfo: AttackInfoObservation[];
  frameCount: number;
  frameTimestamps: number[];
  sampleData: FrameSample[] | null;
  codecConfig: VideoCodecConfig | null;
  frameToSampleIndex: number[] | null;
}

export interface DebugOverlayVisibility {
  raw: boolean;
  hue: boolean;
  hp: boolean;
  drive: boolean;
  super: boolean;
  input: boolean;
  attackInfo: boolean;
}

export interface DebugFrameDataExport {
  frame: number;
  timeSeconds: number | null;
  /** フレームメーターから確定したゲームフレームの対応。 */
  timeline: { segmentId: number; gameFrame: number } | null;
  hp: HpFrameData | null;
  input: { p1: TrackedInputRow | null; p2: TrackedInputRow | null };
  attackInfo: AttackInfoObservation | null;
}

/**
 * 表示中フレームの認識結果を、動画を含まない形で取り出す。
 * 録画環境ごとの読み取りずれを、画像なしで報告できるようにするための書き出し。
 */
export function frameDataAt(
  data: DebugViewerData,
  frame: number,
): DebugFrameDataExport {
  const mapped = data.timeline.video_map[String(frame)];
  return {
    frame,
    timeSeconds: data.frameTimestamps[frame] ?? null,
    timeline: mapped ? { segmentId: mapped[0], gameFrame: mapped[1] } : null,
    hp: data.hpFeatures[frame] ?? null,
    input: {
      p1: data.trackedInputs?.p1[frame] ?? null,
      p2: data.trackedInputs?.p2[frame] ?? null,
    },
    attackInfo:
      data.attackInfo.find(
        (observation) => observation.frame_index === frame,
      ) ?? null,
  };
}

export function initialDebugOverlayVisibility(): DebugOverlayVisibility {
  return {
    raw: false,
    hue: false,
    hp: false,
    drive: false,
    super: false,
    input: false,
    attackInfo: false,
  };
}
