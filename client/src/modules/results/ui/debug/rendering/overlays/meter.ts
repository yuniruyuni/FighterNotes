// フレームメーターの確定値/生読みオーバーレイ描画と共有定数。

import {
  finalValueAt,
  type IndexedTimeline,
  type RustTimelineEntry,
} from "~/modules/analysis/contracts.js";
import type { DebugMeterRowInspection } from "../../../../application/debug-frame-inspection.js";

// ─── 定数 ────────────────────────────────────────────────────────────────────

export const CELL_COUNT = 80;
export const FPS = 60;

// 1920×1080 基準 ROI（Python viewer.py の定数と一致）
export const METER_X1 = 359;
export const METER_X2 = 1559;
const _ANNOT_ROWS: Record<string, [number, number]> = {
  left: [880, 916],
  right: [918, 954],
};
const _ANNOT_PANEL_Y1 = 876;
const _ANNOT_PANEL_Y2 = 958;

// 確定値（final）バーはゲームメーター直下のアノテーションパネルに描く
// 生読み（raw）バーはその下段に描く（トグル ON 時のみ）
export const FINAL_ROWS: Record<string, [number, number]> = {
  left: [880, 916],
  right: [918, 954],
};
export const RAW_ROWS: Record<string, [number, number]> = {
  left: [962, 992],
  right: [1000, 1030],
};
export const FINAL_PANEL_Y1 = 876;
export const FINAL_PANEL_Y2 = 958;
export const RAW_PANEL_Y1 = 958;
export const RAW_PANEL_Y2 = 1034;

// 状態 → RGB（Python _STATE_BGR を BGR→RGB 変換済み）
const STATE_RGB: Record<string, [number, number, number]> = {
  counter: [0, 180, 90],
  punish_counter: [0, 80, 220],
  motion_recovery: [0, 220, 220],
  active: [220, 0, 130],
  projectile_active: [255, 140, 0],
  parry: [180, 0, 180],
  stun: [220, 220, 0],
  inv_full: [150, 150, 150],
  inv_strike: [200, 60, 60],
  inv_proj: [230, 160, 50],
  empty: [40, 40, 40],
  other: [30, 30, 30],
  unknown: [20, 20, 20],
};

const STRIPE_STATES = new Set(["inv_full", "inv_strike", "inv_proj"]);
const DARK_STATES = new Set(["empty", "other", "unknown"]);

// Rust MeterTimeline の JSON 形式
// { "side": "left", "segments": [{ "segment_id": 0, "entries": [...] }] }
// ─── 描画ユーティリティ ───────────────────────────────────────────────────────

export function drawStripedRect(
  ctx: CanvasRenderingContext2D,
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  r: number,
  g: number,
  b: number,
  period = 3,
): void {
  const h = Math.ceil(y2 - y1);
  for (let row = 0; row < h; row++) {
    ctx.fillStyle =
      Math.floor(row / period) % 2 === 0
        ? "rgb(255,255,255)"
        : `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
    ctx.fillRect(x1, y1 + row, x2 - x1, 1);
  }
}

export function fillCell(
  ctx: CanvasRenderingContext2D,
  state: string,
  dimmed: boolean,
  cx1: number,
  y1: number,
  cx2: number,
  y2: number,
): void {
  if (DARK_STATES.has(state)) {
    ctx.fillStyle = "rgb(28,28,28)";
    ctx.fillRect(cx1, y1, cx2 - cx1, y2 - y1);
    return;
  }
  let [r, g, b] = STATE_RGB[state] ?? [50, 50, 50];
  if (dimmed) {
    r *= 0.55;
    g *= 0.55;
    b *= 0.55;
  }
  if (STRIPE_STATES.has(state)) {
    drawStripedRect(ctx, cx1, y1, cx2, y2, r, g, b);
  } else {
    ctx.fillStyle = `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
    ctx.fillRect(cx1, y1, cx2 - cx1, y2 - y1);
  }
}

// ─── 確定値バー描画（常時表示・主） ──────────────────────────────────────────

export function drawFinalOverlay(
  ctx: CanvasRenderingContext2D,
  side: "left" | "right",
  tl: IndexedTimeline,
  frameIdx: number,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;
  const rx1 = METER_X1 * sx;
  const rx2 = METER_X2 * sx;
  const rw = rx2 - rx1;

  const [y1Ref, y2Ref] = FINAL_ROWS[side];
  const y1 = y1Ref * sy;
  const y2 = y2Ref * sy;

  // 背景パネル（left 描画時にまとめて塗る）
  if (side === "left") {
    ctx.fillStyle = "rgb(18,18,18)";
    ctx.fillRect(
      rx1 - 40 * sx,
      FINAL_PANEL_Y1 * sy,
      rw + 44 * sx,
      (FINAL_PANEL_Y2 - FINAL_PANEL_Y1) * sy,
    );
  }

  // 現在ビデオフレームに対応するゲームフレームを逆引き
  // video_map があれば確定マッピングを優先（dwell ギャップも正確に埋まる）
  let cur = finalValueAt(tl.ivals, frameIdx);
  let anchor: (RustTimelineEntry & { segment_id: number }) | null = cur;
  const vmEntry = tl.videoMap.get(frameIdx);
  if (vmEntry) {
    const vmAnchor =
      tl.byGf.get(`${vmEntry.segment_id}:${vmEntry.game_frame}`) ?? null;
    if (vmAnchor) anchor = vmAnchor;
    // cur は ivals 上の厳密な dwell 区間マッチのみ（テキスト表示用）
    if (!cur) cur = vmAnchor;
  }
  if (!anchor && tl.ivals.length > 0) {
    let lo = 0,
      hi = tl.ivals.length;
    while (lo < hi) {
      const mid = (lo + hi) >> 1;
      if (tl.ivals[mid][0] <= frameIdx) lo = mid + 1;
      else hi = mid;
    }
    if (lo > 0) anchor = tl.ivals[lo - 1][2];
  }

  if (anchor) {
    const g = anchor.game_frame;
    const si = anchor.segment_id;
    const lapStart = g - (g % CELL_COUNT);
    for (let gf = g - CELL_COUNT + 1; gf <= g; gf++) {
      if (gf < 0) continue;
      const e = tl.byGf.get(`${si}:${gf}`);
      const cell = gf % CELL_COUNT;
      const cx1 = rx1 + (rw * cell) / CELL_COUNT;
      const cx2 = rx1 + (rw * (cell + 1)) / CELL_COUNT;
      if (!e) {
        ctx.fillStyle = "rgb(25,25,25)";
        ctx.fillRect(cx1, y1, cx2 - cx1, y2 - y1);
        continue;
      }
      fillCell(ctx, e.state, gf < lapStart, cx1, y1, cx2, y2);
      if (e.confidence < 1.0) {
        ctx.strokeStyle = "rgba(255,255,255,0.7)";
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(cx1 + 1, y2 - 2);
        ctx.lineTo(cx2 - 2, y2 - 2);
        ctx.stroke();
      }
    }
    // 現在位置マーカー
    const mx = rx1 + (rw * ((g % CELL_COUNT) + 0.5)) / CELL_COUNT;
    ctx.strokeStyle = "white";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(mx, y1 - 2);
    ctx.lineTo(mx, y2 + 2);
    ctx.stroke();
  } else {
    // タイムラインにデータがない区間は空バーを塗る
    ctx.fillStyle = "rgb(25,25,25)";
    ctx.fillRect(rx1, y1, rw, y2 - y1);
  }

  // ラベル
  ctx.fillStyle = "rgb(200,200,200)";
  ctx.font = `${Math.round(11 * sy)}px sans-serif`;
  ctx.fillText(side === "left" ? "P1" : "P2", rx1 - 36 * sx, y1 + 22 * sy);

  // 右端に現在フレームの確定値テキスト
  const text = cur
    ? `${cur.state} c=${cur.confidence.toFixed(1)} gf=${cur.game_frame}`
    : "(no data)";
  const [tr, tg, tb] = cur
    ? (STATE_RGB[cur.state] ?? [200, 200, 200])
    : [110, 110, 110];
  ctx.fillStyle = `rgb(${tr},${tg},${tb})`;
  ctx.font = `${Math.round(11 * sy)}px monospace`;
  ctx.fillText(text, rx2 + 8 * sx, y1 + 20 * sy);
}

// ─── 生読みバー描画（トグル表示・補助） ──────────────────────────────────────

export function drawRawOverlay(
  ctx: CanvasRenderingContext2D,
  side: "left" | "right",
  obs: DebugMeterRowInspection,
  cw: number,
  ch: number,
  showHue: boolean,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;
  const rx1 = METER_X1 * sx;
  const rx2 = METER_X2 * sx;
  const rowW = rx2 - rx1;

  const [ary1Ref, ary2Ref] = RAW_ROWS[side];
  const ary1 = ary1Ref * sy;
  const ary2 = ary2Ref * sy;

  // 背景パネル（left 描画時にまとめて塗る）
  if (side === "left") {
    ctx.fillStyle = "rgb(14,14,14)";
    ctx.fillRect(
      rx1 - 40 * sx,
      RAW_PANEL_Y1 * sy,
      rowW + 44 * sx,
      (RAW_PANEL_Y2 - RAW_PANEL_Y1) * sy,
    );
  }

  for (let i = 0; i < CELL_COUNT; i++) {
    const state = obs.states[i] ?? "unknown";
    const cx1 = rx1 + (rowW * i) / CELL_COUNT;
    const cx2 = rx1 + (rowW * (i + 1)) / CELL_COUNT;
    fillCell(ctx, state, obs.bright[i] === "low", cx1, ary1, cx2, ary2);

    if (showHue && obs.bgr[i]) {
      const [bv, gv, rv] = obs.bgr[i];
      const hue = bgrToHue(bv, gv, rv);
      ctx.fillStyle = "rgba(255,255,255,0.85)";
      ctx.font = `${Math.max(6, 9 * sy)}px monospace`;
      ctx.fillText(String(hue), cx1 + 1, ary1 + (ary2 - ary1) * 0.75);
    }
  }

  // ラベル
  ctx.fillStyle = "rgb(150,150,150)";
  ctx.font = `${Math.round(10 * sy)}px monospace`;
  ctx.fillText(
    side === "left" ? "P1 raw" : "P2 raw",
    rx1 - 38 * sx,
    ary1 + 18 * sy,
  );

  // fresh_edge マーカー
  if (obs.fresh_edge >= 0) {
    const cc = obs.fresh_edge;
    const cx = rx1 + (rowW * (cc + 0.5)) / CELL_COUNT;
    ctx.strokeStyle = "rgba(255,255,255,0.8)";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(cx, ary1 - 2);
    ctx.lineTo(cx, ary2 + 2);
    ctx.stroke();
    const label = `${obs.states[cc]}[${cc}]`;
    ctx.fillStyle = "white";
    ctx.font = `${Math.round(10 * sy)}px monospace`;
    ctx.fillText(label, Math.min(cx + 4, rx2 - 140 * sx), ary1 - 3);
  }
}

// ─── HUE 計算（OpenCV 互換: H 0-179） ────────────────────────────────────────

export function bgrToHue(b: number, g: number, r: number): number {
  const bN = b / 255,
    gN = g / 255,
    rN = r / 255;
  const max = Math.max(rN, gN, bN);
  const min = Math.min(rN, gN, bN);
  const delta = max - min;
  if (delta < 1e-6) return 0;
  let h: number;
  if (Math.abs(max - rN) < 1e-6) h = 60 * (((gN - bN) / delta) % 6);
  else if (Math.abs(max - gN) < 1e-6) h = 60 * ((bN - rN) / delta + 2);
  else h = 60 * ((rN - gN) / delta + 4);
  if (h < 0) h += 360;
  return Math.round(h / 2);
}
