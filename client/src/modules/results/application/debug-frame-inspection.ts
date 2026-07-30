export interface DebugMeterRowInspection {
  states: string[];
  bright: string[];
  fresh_edge: number;
  bgr: [number, number, number][];
  v: number[];
}

export interface DebugMeterInspection {
  left: DebugMeterRowInspection;
  right: DebugMeterRowInspection;
}

export interface DebugHpInspection {
  left_score: number;
  right_score: number;
  left_fill: number;
  right_fill: number;
  left_drive: number;
  right_drive: number;
  left_col_active: boolean[];
  right_col_active: boolean[];
  left_col_orange: boolean[];
  right_col_orange: boolean[];
  left_col_yellow: boolean[];
  right_col_yellow: boolean[];
  left_orange_fill: number;
  right_orange_fill: number;
  left_yellow_fill: number;
  right_yellow_fill: number;
}

export interface DebugDriveSideInspection {
  value: number;
  burnout: boolean;
  recovery: number;
  uncertain: boolean;
  roi: { x1: number; x2: number; y1: number; y2: number; slope: number };
  cols: string;
  runs: { c: string; s: number; e: number; w: number }[];
}

export interface DebugDriveInspection {
  left: DebugDriveSideInspection;
  right: DebugDriveSideInspection;
}

export interface DebugSuperRoi {
  x1: number;
  x2: number;
  y1: number;
  y2: number;
}

export interface DebugSuperSideInspection {
  value: number;
  displayed_level: number | null;
  critical_art: boolean;
  uncertain: boolean;
  label_roi: DebugSuperRoi;
  bar_roi: DebugSuperRoi;
}

export interface DebugSuperInspection {
  left: DebugSuperSideInspection;
  right: DebugSuperSideInspection;
}

export interface DebugInputRowInspection {
  count: number | null;
  dir: string;
  badges: string;
  auto: boolean;
  throw: boolean;
  empty: boolean;
  uncertain: boolean;
}

export interface DebugInputInspection {
  p1: { side: string; rows: DebugInputRowInspection[] };
  p2: { side: string; rows: DebugInputRowInspection[] };
}

export interface DebugAttackInfoRoi {
  x1: number;
  x2: number;
  y1: number;
  y2: number;
}

export interface DebugAttackInfoSideInspection {
  last_damage: number;
  scaling_percent: number;
  combo_damage: number;
  max_combo_damage: number;
  attribute: "upper" | "middle" | "lower" | "throw";
  numeric_score: number;
  attribute_score: number;
  attribute_margin: number;
}

export interface DebugAttackInfoInspection {
  p1: DebugAttackInfoSideInspection | null;
  p2: DebugAttackInfoSideInspection | null;
  rois: {
    p1: { numeric: DebugAttackInfoRoi; attribute: DebugAttackInfoRoi };
    p2: { numeric: DebugAttackInfoRoi; attribute: DebugAttackInfoRoi };
  };
}

export interface DebugHpParallelogram {
  top_left: { x: number; y: number };
  top_right: { x: number; y: number };
  bottom_right: { x: number; y: number };
  bottom_left: { x: number; y: number };
}

export interface DebugHpGeometry {
  p1: DebugHpParallelogram;
  p2: DebugHpParallelogram;
}

export interface DebugFrameInspector {
  initialize(): Promise<DebugHpGeometry>;
  inspectMeter(
    rgba: Uint8Array,
    width: number,
    height: number,
  ): DebugMeterInspection;
  inspectHp(rgba: Uint8Array, width: number, height: number): DebugHpInspection;
  inspectDrive(
    rgba: Uint8Array,
    width: number,
    height: number,
  ): DebugDriveInspection;
  inspectSuper(
    rgba: Uint8Array,
    width: number,
    height: number,
  ): DebugSuperInspection;
  inspectInput(
    rgba: Uint8Array,
    width: number,
    height: number,
  ): DebugInputInspection;
  inspectAttackInfo(
    rgba: Uint8Array,
    width: number,
    height: number,
  ): DebugAttackInfoInspection;
}
