// OD（ドライブ）ゲージデバッグオーバーレイ。

import type { HpFrameData } from "~/modules/analysis/contracts.js";
import type {
  DebugDriveInspection,
  DebugDriveSideInspection,
} from "../../../../application/debug-frame-inspection.js";
import { METER_X1, METER_X2 } from "./meter.js";

// ─── OD（ドライブ）ゲージ デバッグ表示 ───────────────────────────────────────

// レイアウト（1080p 基準）: HP デバッグ領域（〜y=155）の下に配置
const DRIVE_BAR_DEBUG_Y1 = 160; // 再現ゲージ開始 Y
const DRIVE_BAR_DEBUG_H = 14; // 再現ゲージの高さ
const DRIVE_COL_Y1 = 177; // 列分類ドット開始 Y
const DRIVE_COL_H = 4;
const DRIVE_TL_Y1 = 185; // タイムライン開始 Y
const DRIVE_TL_ROW_H = 10;
const DRIVE_TL_GAP = 2;
const DRIVE_TL_WINDOW = 150; // ±150F

const DRIVE_COL_COLORS: Record<string, string> = {
  L: "rgb(120,220,0)", // Lit
  G: "rgb(200,200,200)", // Gray（バーンアウト回復バー）
  F: "rgb(255,60,60)", // Foreign（遮蔽）
  ".": "rgb(28,28,28)", // Rest
};

/// ① ドライブゲージ ROI の平行四辺形枠線（映像上オーバーレイ）
export function drawDriveRoiOverlay(
  ctx: CanvasRenderingContext2D,
  drv: DebugDriveInspection,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;
  const drawPoly = (d: DebugDriveSideInspection) => {
    const { x1, x2, y1, y2, slope } = d.roi;
    const slant = (y2 - y1) * slope * sx;
    ctx.beginPath();
    ctx.moveTo(x1 * sx, y1 * sy);
    ctx.lineTo(x2 * sx, y1 * sy);
    ctx.lineTo(x2 * sx + slant, y2 * sy);
    ctx.lineTo(x1 * sx + slant, y2 * sy);
    ctx.closePath();
    ctx.stroke();
  };
  ctx.strokeStyle = "rgba(0,220,160,0.9)";
  ctx.lineWidth = 2;
  drawPoly(drv.left);
  drawPoly(drv.right);
}

/// ② 再現ドライブゲージ（6 セル区切り + 値/バーンアウト/uncertain 表示）
export function drawDriveBarReproduced(
  ctx: CanvasRenderingContext2D,
  drv: DebugDriveInspection,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;
  const y1 = DRIVE_BAR_DEBUG_Y1 * sy;
  const h = DRIVE_BAR_DEBUG_H * sy;

  const drawBar = (
    d: DebugDriveSideInspection,
    anchorRight: boolean,
    label: string,
  ) => {
    const x1 = d.roi.x1 * sx,
      x2 = d.roi.x2 * sx;
    const rw = x2 - x1;

    // 背景
    ctx.fillStyle = "rgba(18,18,18,0.88)";
    ctx.fillRect(x1, y1, rw, h);

    // uncertain でも保持値（パイプラインの forward fill）があれば減光表示する
    const hasHeld = d.uncertain && (d.value > 0 || d.burnout);
    const alpha = d.uncertain ? 0.45 : 0.9;
    if (d.uncertain && !hasHeld) {
      ctx.fillStyle = "rgba(120,60,60,0.6)";
      ctx.fillRect(x1, y1, rw, h);
    } else if (d.burnout) {
      // バーンアウト回復バー: アンカー（中央側）から灰白で伸びる
      const w = rw * d.recovery;
      ctx.fillStyle = `rgba(210,210,210,${alpha})`;
      if (anchorRight) ctx.fillRect(x2 - w, y1, w, h);
      else ctx.fillRect(x1, y1, w, h);
    } else {
      // 通常: アンカーから value/6 まで緑で充填
      const w = (rw * d.value) / 6;
      ctx.fillStyle =
        d.value <= 1.0
          ? `rgba(255,150,0,${alpha})`
          : `rgba(120,220,0,${alpha})`;
      if (anchorRight) ctx.fillRect(x2 - w, y1, w, h);
      else ctx.fillRect(x1, y1, w, h);
    }

    // 6 セルの区切り線
    ctx.strokeStyle = "rgba(255,255,255,0.35)";
    ctx.lineWidth = 1;
    for (let i = 1; i < 6; i++) {
      const cx = anchorRight ? x2 - (rw * i) / 6 : x1 + (rw * i) / 6;
      ctx.beginPath();
      ctx.moveTo(cx, y1);
      ctx.lineTo(cx, y1 + h);
      ctx.stroke();
    }

    // テキスト
    const hold = d.uncertain && hasHeld ? " (hold)" : "";
    const text =
      d.uncertain && !hasHeld
        ? `${label} OD:?`
        : d.burnout
          ? `${label} BURNOUT ${(d.recovery * 100).toFixed(0)}%${hold}`
          : `${label} OD:${d.value.toFixed(2)}${hold}`;
    ctx.fillStyle = "rgba(255,255,255,0.92)";
    ctx.font = `bold ${Math.round(10 * sy)}px monospace`;
    ctx.fillText(text, x1 + 4 * sx, y1 + h * 0.75);
  };

  drawBar(drv.left, true, "P1"); // 左ゲージ: アンカー = 右端（中央側）
  drawBar(drv.right, false, "P2"); // 右ゲージ: アンカー = 左端（中央側）
}

/// ③ 列分類ドット（cols 文字列の可視化: Lit/Gray/Foreign/Rest）
export function drawDriveColRow(
  ctx: CanvasRenderingContext2D,
  drv: DebugDriveInspection,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;
  const y1 = DRIVE_COL_Y1 * sy;
  const h = DRIVE_COL_H * sy;

  const drawRow = (d: DebugDriveSideInspection) => {
    const x1 = d.roi.x1 * sx,
      x2 = d.roi.x2 * sx;
    const n = d.cols.length;
    if (n === 0) return;
    const colW = (x2 - x1) / n;
    for (let i = 0; i < n; i++) {
      ctx.fillStyle = DRIVE_COL_COLORS[d.cols[i]] ?? "rgb(28,28,28)";
      ctx.fillRect(x1 + i * colW, y1, Math.max(1, colW), h);
    }
  };
  drawRow(drv.left);
  drawRow(drv.right);
}

// ドライブ値 → 表示色（タイムライン用）
function driveColor(
  ratio: number,
  burnout: boolean,
  uncertain: boolean,
): string {
  if (uncertain) return "#454545";
  if (burnout)
    return `rgb(${100 + Math.round(ratio * 120)},${100 + Math.round(ratio * 120)},${110 + Math.round(ratio * 120)})`;
  // 0=橙 → 1=緑
  return `hsl(${Math.round(30 + ratio * 80)},80%,42%)`;
}

/// ④ ドライブタイムライン（±DRIVE_TL_WINDOW フレームの履歴）
export function drawDriveTimeline(
  ctx: CanvasRenderingContext2D,
  hpData: HpFrameData[],
  frameIdx: number,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;
  const x1 = METER_X1 * sx,
    x2 = METER_X2 * sx;
  const tlW = x2 - x1;
  const totalCols = DRIVE_TL_WINDOW * 2;
  const colW = tlW / totalCols;

  const p1y = DRIVE_TL_Y1 * sy;
  const p2y = p1y + (DRIVE_TL_ROW_H + DRIVE_TL_GAP) * sy;
  const panelH = (DRIVE_TL_ROW_H * 2 + DRIVE_TL_GAP + 4) * sy;

  ctx.fillStyle = "rgb(14,14,14)";
  ctx.fillRect(x1 - 40 * sx, p1y - 2 * sy, tlW + 44 * sx, panelH);

  for (let di = -DRIVE_TL_WINDOW; di < DRIVE_TL_WINDOW; di++) {
    const fi = frameIdx + di;
    const xi = x1 + (di + DRIVE_TL_WINDOW) * colW;
    const cw2 = Math.max(1, colW);

    if (fi < 0 || fi >= hpData.length) {
      ctx.fillStyle = "#0a0a0a";
      ctx.fillRect(xi, p1y, cw2, DRIVE_TL_ROW_H * sy);
      ctx.fillRect(xi, p2y, cw2, DRIVE_TL_ROW_H * sy);
      continue;
    }

    const d = hpData[fi];
    ctx.fillStyle = d.is_match_screen
      ? driveColor(d.left_drive_ratio, d.left_burnout, d.left_drive_uncertain)
      : "#1a1a1a";
    ctx.fillRect(xi, p1y, cw2, DRIVE_TL_ROW_H * sy);
    ctx.fillStyle = d.is_match_screen
      ? driveColor(
          d.right_drive_ratio,
          d.right_burnout,
          d.right_drive_uncertain,
        )
      : "#1a1a1a";
    ctx.fillRect(xi, p2y, cw2, DRIVE_TL_ROW_H * sy);
  }

  // 現在フレームマーカー
  const mx = x1 + DRIVE_TL_WINDOW * colW;
  ctx.strokeStyle = "white";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(mx, p1y);
  ctx.lineTo(mx, p2y + DRIVE_TL_ROW_H * sy);
  ctx.stroke();

  // ラベル
  ctx.fillStyle = "rgb(180,180,180)";
  ctx.font = `${Math.round(10 * sy)}px monospace`;
  ctx.fillText("OD L", x1 - 38 * sx, p1y + DRIVE_TL_ROW_H * 0.72 * sy);
  ctx.fillText("OD R", x1 - 38 * sx, p2y + DRIVE_TL_ROW_H * 0.72 * sy);

  // 現フレームの数値
  if (frameIdx >= 0 && frameIdx < hpData.length) {
    const d = hpData[frameIdx];
    const fmt = (ratio: number, burnout: boolean, unc: boolean) =>
      unc
        ? "?"
        : burnout
          ? `BO ${(ratio * 100).toFixed(0)}%`
          : (ratio * 6).toFixed(2);
    ctx.fillStyle = "rgb(220,220,220)";
    ctx.font = `${Math.round(10 * sy)}px monospace`;
    ctx.fillText(
      `L=${fmt(d.left_drive_ratio, d.left_burnout, d.left_drive_uncertain)}`,
      x2 + 8 * sx,
      p1y + DRIVE_TL_ROW_H * 0.72 * sy,
    );
    ctx.fillText(
      `R=${fmt(d.right_drive_ratio, d.right_burnout, d.right_drive_uncertain)}`,
      x2 + 8 * sx,
      p2y + DRIVE_TL_ROW_H * 0.72 * sy,
    );
  }
}
