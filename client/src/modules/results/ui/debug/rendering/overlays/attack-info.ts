import type {
  AttackInfoObservation,
  AttackInfoSide,
} from "~/modules/analysis/contracts.js";
import type {
  DebugAttackInfoInspection,
  DebugAttackInfoRoi,
  DebugAttackInfoSideInspection,
} from "../../../../application/debug-frame-inspection.js";

const ATTRIBUTE_LABEL = {
  upper: "上段",
  middle: "中段",
  lower: "下段",
  throw: "投げ",
} as const;

const PANEL_X = { p1: 500, p2: 1040 } as const;
const PANEL_Y = 276;
const PANEL_W = 380;
const PANEL_H = 52;
const TIMELINE_X1 = 600;
const TIMELINE_X2 = 1320;
const TIMELINE_Y = 338;
const TIMELINE_WINDOW = 180;

export function drawAttackInfoDebug(
  context: CanvasRenderingContext2D,
  inspection: DebugAttackInfoInspection,
  observations: AttackInfoObservation[],
  frameIndex: number,
  canvasWidth: number,
  canvasHeight: number,
): void {
  const scaleX = canvasWidth / 1920;
  const scaleY = canvasHeight / 1080;
  for (const side of ["p1", "p2"] as const) {
    drawRoi(
      context,
      inspection.rois[side].numeric,
      side === "p1" ? "rgb(255,110,190)" : "rgb(80,205,255)",
      `${side.toUpperCase()} damage`,
      scaleX,
      scaleY,
    );
    drawRoi(
      context,
      inspection.rois[side].attribute,
      "rgb(255,220,70)",
      `${side.toUpperCase()} attribute`,
      scaleX,
      scaleY,
    );
  }

  const current = observationAt(observations, frameIndex);
  drawPanel(context, "p1", inspection.p1, current?.p1, scaleX, scaleY);
  drawPanel(context, "p2", inspection.p2, current?.p2, scaleX, scaleY);
  drawTimeline(context, observations, frameIndex, canvasWidth, scaleX, scaleY);
}

function drawRoi(
  context: CanvasRenderingContext2D,
  roi: DebugAttackInfoRoi,
  color: string,
  label: string,
  scaleX: number,
  scaleY: number,
): void {
  const x = roi.x1 * scaleX;
  const y = roi.y1 * scaleY;
  context.strokeStyle = color;
  context.lineWidth = 2;
  context.strokeRect(
    x,
    y,
    (roi.x2 - roi.x1) * scaleX,
    (roi.y2 - roi.y1) * scaleY,
  );
  context.fillStyle = color;
  context.font = `bold ${Math.round(10 * scaleY)}px monospace`;
  context.fillText(label, x, y - 3 * scaleY);
}

function drawPanel(
  context: CanvasRenderingContext2D,
  side: "p1" | "p2",
  raw: DebugAttackInfoSideInspection | null,
  confirmed: AttackInfoSide | undefined,
  scaleX: number,
  scaleY: number,
): void {
  const x = PANEL_X[side] * scaleX;
  const y = PANEL_Y * scaleY;
  context.fillStyle = "rgba(8,10,18,0.90)";
  context.fillRect(x, y, PANEL_W * scaleX, PANEL_H * scaleY);
  context.font = `bold ${Math.round(12 * scaleY)}px monospace`;
  context.fillStyle = side === "p1" ? "rgb(255,130,200)" : "rgb(100,215,255)";
  context.fillText(
    `${side.toUpperCase()} raw  ${formatSide(raw)}`,
    x + 6 * scaleX,
    y + 18 * scaleY,
  );
  context.fillStyle = "rgb(150,245,160)";
  context.fillText(
    `${side.toUpperCase()} 確定 ${formatSide(confirmed)}`,
    x + 6 * scaleX,
    y + 39 * scaleY,
  );
  if (raw) {
    context.fillStyle = "rgb(185,185,185)";
    context.font = `${Math.round(9 * scaleY)}px monospace`;
    context.fillText(
      `digit ${raw.numeric_score} / attr ${raw.attribute_score} margin ${raw.attribute_margin}`,
      x + 190 * scaleX,
      y + 50 * scaleY,
    );
  }
}

function formatSide(
  value: AttackInfoSide | DebugAttackInfoSideInspection | null | undefined,
): string {
  if (!value) return "認識保留";
  return `${value.last_damage} (${value.scaling_percent}%)  combo ${value.combo_damage} (${value.max_combo_damage})  ${ATTRIBUTE_LABEL[value.attribute]}`;
}

function drawTimeline(
  context: CanvasRenderingContext2D,
  observations: AttackInfoObservation[],
  frameIndex: number,
  canvasWidth: number,
  scaleX: number,
  scaleY: number,
): void {
  const x1 = TIMELINE_X1 * scaleX;
  const x2 = Math.min(TIMELINE_X2 * scaleX, canvasWidth);
  const width = x2 - x1;
  const columnWidth = width / (TIMELINE_WINDOW * 2);
  for (const [row, side] of ["p1", "p2"].entries()) {
    const y = (TIMELINE_Y + row * 12) * scaleY;
    context.fillStyle = "rgba(8,8,8,0.88)";
    context.fillRect(x1, y, width, 10 * scaleY);
    for (let delta = -TIMELINE_WINDOW; delta < TIMELINE_WINDOW; delta++) {
      const state = observationAt(observations, frameIndex + delta);
      if (!state) continue;
      const value = state[side as "p1" | "p2"];
      context.fillStyle = attributeColor(value.attribute, value.combo_damage);
      context.fillRect(
        x1 + (delta + TIMELINE_WINDOW) * columnWidth,
        y,
        Math.max(1, columnWidth),
        10 * scaleY,
      );
    }
  }
  const markerX = x1 + TIMELINE_WINDOW * columnWidth;
  context.strokeStyle = "white";
  context.lineWidth = 1;
  context.beginPath();
  context.moveTo(markerX, TIMELINE_Y * scaleY);
  context.lineTo(markerX, (TIMELINE_Y + 22) * scaleY);
  context.stroke();

  const previous = previousChange(observations, frameIndex);
  const next = observations.find((item) => item.frame_index > frameIndex);
  context.fillStyle = "rgb(220,220,220)";
  context.font = `${Math.round(10 * scaleY)}px monospace`;
  context.fillText(
    `攻撃情報 change ${previous?.frame_index ?? "-"} ← f${frameIndex} → ${next?.frame_index ?? "-"}`,
    x1,
    (TIMELINE_Y + 36) * scaleY,
  );
}

function attributeColor(
  attribute: AttackInfoSide["attribute"],
  comboDamage: number,
): string {
  const alpha = Math.min(0.95, 0.45 + comboDamage / 8000);
  switch (attribute) {
    case "middle":
      return `rgba(255,190,45,${alpha})`;
    case "lower":
      return `rgba(70,190,255,${alpha})`;
    case "throw":
      return `rgba(255,80,100,${alpha})`;
    default:
      return `rgba(120,225,130,${alpha})`;
  }
}

function observationAt(
  observations: AttackInfoObservation[],
  frameIndex: number,
): AttackInfoObservation | undefined {
  let low = 0;
  let high = observations.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if (observations[middle].frame_index <= frameIndex) low = middle + 1;
    else high = middle;
  }
  return low === 0 ? undefined : observations[low - 1];
}

function previousChange(
  observations: AttackInfoObservation[],
  frameIndex: number,
): AttackInfoObservation | undefined {
  return observationAt(observations, frameIndex);
}
