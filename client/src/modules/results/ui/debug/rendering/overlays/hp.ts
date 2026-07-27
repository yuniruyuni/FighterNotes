// HP バーデバッグオーバーレイ（ROI 枠・再現バー・列ドット・タイムライン）。

import type { HpFrameData } from "~/modules/analysis/contracts.js";
import type {
  DebugHpGeometry,
  DebugHpInspection,
  DebugHpParallelogram,
} from "../../../../application/debug-frame-inspection.js";
import { METER_X1, METER_X2 } from "./meter.js";

// ─── HP デバッグ表示 ──────────────────────────────────────────────────────────

// HP デバッグパネル（HP バー y=64-95 の直下）
const HP_BAR_DEBUG_Y1 = 98; // 再現 HP バー開始 Y
const HP_BAR_DEBUG_H = 16; // 再現 HP バーの高さ
const HP_COL_Y1 = 115; // col_active ドット開始 Y（P1/P2 共通）
const HP_COL_H = 4; // col_active ドット高さ
const HP_COL_GAP = 1;
const HP_TL_ROW_H = 10; // タイムライン 1 行の高さ
const HP_TL_GAP = 2; // Own/Opp 間のギャップ
const HP_TL_WINDOW = 150; // ±150F（合計 300F）

// HP 値を色相（0=赤 → 120=緑）にマッピングして hsl 文字列を返す
function hpHsl(val: number): string {
  if (val < 0) return "#555"; // -1 = 不明（uncertain）
  return `hsl(${Math.round(val * 120)},70%,45%)`;
}

/// ① HP ROI 検出領域の枠線のみ描画（映像上オーバーレイ）
/// classify_hp_col の実スキャン範囲と完全一致する平行四辺形で描画する。
export function drawHpRoiOverlay(
  ctx: CanvasRenderingContext2D,
  geometry: DebugHpGeometry,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;

  const drawPoly = (para: DebugHpParallelogram) => {
    const tl = para.top_left,
      tr = para.top_right;
    const br = para.bottom_right,
      bl = para.bottom_left;
    ctx.beginPath();
    // top_right / bottom_right は inclusive のため +1px で右端を含む
    ctx.moveTo(tl.x * sx, tl.y * sy);
    ctx.lineTo((tr.x + 1) * sx, tr.y * sy);
    ctx.lineTo((br.x + 1) * sx, br.y * sy);
    ctx.lineTo(bl.x * sx, bl.y * sy);
    ctx.closePath();
    ctx.stroke();
  };

  ctx.strokeStyle = "rgba(255,220,0,0.9)";
  ctx.lineWidth = 2;
  drawPoly(geometry.p1);
  drawPoly(geometry.p2);
}

/// ② 平行四辺形 HP バーの再現（実際の HP バー直下に同形状で描画）
///
/// 実ゲームの HP バーと同じ平行四辺形にクリップし、検出値を可視化する。
/// - P1: 残存 HP は右側（cap 端から左へ充填が伸びる）
/// - P2: 残存 HP は左側（cap 端から右へ充填が伸びる）
export function drawHpBarReproduced(
  ctx: CanvasRenderingContext2D,
  geometry: DebugHpGeometry,
  hp: DebugHpInspection,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;

  const drawBar = (
    para: DebugHpParallelogram,
    fill: number,
    orangeFill: number,
    yellowFill: number,
    score: number,
    drive: number,
    label: string,
    hpOnRight: boolean, // HP の位置: true=右側(P1), false=左側(P2)
    hpColor: string,
  ) => {
    // 平行四辺形の傾き（水平オフセット）= bottom_left.x − top_left.x
    // P1: +16（右下がり）、P2: −16（左下がり）
    const slant = (para.bottom_left.x - para.top_left.x) * sx;
    const x1 = para.top_left.x * sx;
    const x2 = (para.top_right.x + 1) * sx;
    const y1 = HP_BAR_DEBUG_Y1 * sy;
    const h = HP_BAR_DEBUG_H * sy;
    const y2 = y1 + h;
    const rw = x2 - x1;

    // 平行四辺形クリップパス（実スキャン範囲と同じ傾き、Y のみデバッグ位置へオフセット）
    ctx.save();
    ctx.beginPath();
    ctx.moveTo(x1, y1);
    ctx.lineTo(x2, y1);
    ctx.lineTo(x2 + slant, y2);
    ctx.lineTo(x1 + slant, y2);
    ctx.closePath();
    ctx.clip();

    // 背景（半透明ダーク）
    ctx.fillStyle = "rgba(18,18,18,0.88)";
    ctx.fillRect(x1, y1, rw, h);

    const fillW = rw * fill;
    const orangeW = rw * orangeFill;
    // 補正済み HP 値 (fill) が 25% 以下のとき黄色表示（SF6 の低 HP 表示を再現）。
    // 生の yellowFill（列単位の黄色検出）は使わない。キャラクター肌色が黄色検出範囲に
    // 入って yellowFill が発生しても、補正 HP が正常ならバーは正常色のままにする。
    const barColor = fill <= 0.25 ? "rgba(255,220,0,0.90)" : hpColor;

    if (hpOnRight) {
      // P1: 残存 HP は右側 → 右端から左へ充填、黒領域は左側
      ctx.fillStyle = barColor;
      ctx.fillRect(x2 - fillW, y1, fillW, h);
      if (orangeFill > 0.001) {
        ctx.fillStyle = "rgba(255,140,0,0.90)";
        ctx.fillRect(x2 - fillW - orangeW, y1, orangeW, h);
      }
    } else {
      // P2: 残存 HP は左側 → 左端から右へ充填、黒領域は右側
      ctx.fillStyle = barColor;
      ctx.fillRect(x1, y1, fillW, h);
      if (orangeFill > 0.001) {
        ctx.fillStyle = "rgba(255,140,0,0.90)";
        ctx.fillRect(x1 + fillW, y1, orangeW, h);
      }
    }

    // 充填境界の縦線
    ctx.strokeStyle = "rgba(255,255,255,0.5)";
    ctx.lineWidth = 1;
    ctx.beginPath();
    const bx = hpOnRight ? x2 - fillW : x1 + fillW;
    ctx.moveTo(bx, y1);
    ctx.lineTo(bx, y2);
    ctx.stroke();

    ctx.restore();

    // テキスト（クリップ外、バー内に重ねて表示）
    ctx.fillStyle = "rgba(255,255,255,0.92)";
    ctx.font = `bold ${Math.round(10 * sy)}px monospace`;
    const orangeStr =
      orangeFill > 0.001 ? ` org:${(orangeFill * 100).toFixed(0)}%` : "";
    const yellowStr =
      yellowFill > 0.001 ? ` ylw:${(yellowFill * 100).toFixed(0)}%` : "";
    const text = `${label} ${(fill * 100).toFixed(0)}%${orangeStr}${yellowStr}  s:${score.toFixed(2)} drv:${drive.toFixed(2)}`;
    ctx.fillText(text, x1 + 4 * sx, y1 + h * 0.74);
  };

  // P1: 残存HP=右側(hpOnRight=true)、P2: 残存HP=左側(hpOnRight=false)
  drawBar(
    geometry.p1,
    hp.left_fill,
    hp.left_orange_fill,
    hp.left_yellow_fill,
    hp.left_score,
    hp.left_drive,
    "P1",
    true,
    "rgba(0,210,80,0.85)",
  );
  drawBar(
    geometry.p2,
    hp.right_fill,
    hp.right_orange_fill,
    hp.right_yellow_fill,
    hp.right_score,
    hp.right_drive,
    "P2",
    false,
    "rgba(0,160,220,0.85)",
  );
}

/// ② HP ROI 列単位ドット（col_active / col_orange / col_yellow の可視化）
///
/// 各サイドにつき 3 行: active（HP色）/ orange（ダメージ中）/ yellow（低HP）。
export function drawHpColActive(
  ctx: CanvasRenderingContext2D,
  geometry: DebugHpGeometry,
  hp: DebugHpInspection,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;

  const drawRow = (
    para: DebugHpParallelogram,
    cols: boolean[],
    yBase: number,
    activeColor: string,
  ) => {
    // top_left.x / top_right.x+1 は roi.x1 / roi.x2 と等価（列幅 = roi_w）
    const x1 = para.top_left.x * sx,
      x2 = (para.top_right.x + 1) * sx;
    const rw = x2 - x1;
    const n = cols.length;
    if (n === 0) return;
    const colW = rw / n;
    const y1 = yBase * sy;
    const h = HP_COL_H * sy;
    for (let i = 0; i < n; i++) {
      ctx.fillStyle = cols[i] ? activeColor : "rgb(28,28,28)";
      ctx.fillRect(x1 + i * colW, y1, Math.max(1, colW), h);
    }
  };

  const stride = HP_COL_H + HP_COL_GAP;
  // P1 と P2 は X 位置が異なるため同一 Y 行に並べる（計 3 行）
  drawRow(
    geometry.p1,
    hp.left_col_active,
    HP_COL_Y1 + stride * 0,
    "rgb(0,210,80)",
  );
  drawRow(
    geometry.p1,
    hp.left_col_orange,
    HP_COL_Y1 + stride * 1,
    "rgb(255,140,0)",
  );
  drawRow(
    geometry.p1,
    hp.left_col_yellow,
    HP_COL_Y1 + stride * 2,
    "rgb(255,220,0)",
  );
  drawRow(
    geometry.p2,
    hp.right_col_active,
    HP_COL_Y1 + stride * 0,
    "rgb(0,160,220)",
  );
  drawRow(
    geometry.p2,
    hp.right_col_orange,
    HP_COL_Y1 + stride * 1,
    "rgb(255,140,0)",
  );
  drawRow(
    geometry.p2,
    hp.right_col_yellow,
    HP_COL_Y1 + stride * 2,
    "rgb(255,220,0)",
  );
}

/// ③ HP タイムライン（±HP_TL_WINDOW フレームの履歴）
export function drawHpTimeline(
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
  const totalCols = HP_TL_WINDOW * 2;
  const colW = tlW / totalCols;

  // タイムライン開始 Y（col_active 3行分の下: P1/P2 共有）
  const panelY = (HP_COL_Y1 + (HP_COL_H + HP_COL_GAP) * 3 + 3) * sy;
  const p1y = panelY;
  const p2y = panelY + (HP_TL_ROW_H + HP_TL_GAP) * sy;
  const panelH = (HP_TL_ROW_H * 2 + HP_TL_GAP) * sy;

  // 背景パネル（フレームメーターと同じ幅 + ラベル分）
  ctx.fillStyle = "rgb(14,14,14)";
  ctx.fillRect(x1 - 40 * sx, panelY, tlW + 44 * sx, panelH);

  for (let di = -HP_TL_WINDOW; di < HP_TL_WINDOW; di++) {
    const fi = frameIdx + di;
    const xi = x1 + (di + HP_TL_WINDOW) * colW;
    const cw2 = Math.max(1, colW);

    if (fi < 0 || fi >= hpData.length) {
      ctx.fillStyle = "#0a0a0a";
      ctx.fillRect(xi, p1y, cw2, HP_TL_ROW_H * sy);
      ctx.fillRect(xi, p2y, cw2, HP_TL_ROW_H * sy);
      continue;
    }

    const d = hpData[fi];
    ctx.fillStyle = d.is_match_screen ? hpHsl(d.own_hp) : "#1a1a1a";
    ctx.fillRect(xi, p1y, cw2, HP_TL_ROW_H * sy);
    ctx.fillStyle = d.is_match_screen ? hpHsl(d.opponent_hp) : "#1a1a1a";
    ctx.fillRect(xi, p2y, cw2, HP_TL_ROW_H * sy);
  }

  // 現在フレームマーカー（中央縦線）
  const mx = x1 + HP_TL_WINDOW * colW;
  ctx.strokeStyle = "white";
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(mx, p1y);
  ctx.lineTo(mx, p2y + HP_TL_ROW_H * sy);
  ctx.stroke();

  // ラベル
  ctx.fillStyle = "rgb(180,180,180)";
  ctx.font = `${Math.round(10 * sy)}px monospace`;
  ctx.fillText("Own", x1 - 36 * sx, p1y + HP_TL_ROW_H * 0.72 * sy);
  ctx.fillText("Opp", x1 - 36 * sx, p2y + HP_TL_ROW_H * 0.72 * sy);

  // 現フレームの数値（補正済み + 生値 + is_match_screen）
  if (frameIdx >= 0 && frameIdx < hpData.length) {
    const d = hpData[frameIdx];
    ctx.fillStyle = "rgb(220,220,220)";
    ctx.font = `${Math.round(10 * sy)}px monospace`;
    const fmtHp = (v: number) => (v < 0 ? "  ?" : v.toFixed(2));
    ctx.fillText(
      `own=${fmtHp(d.own_hp)} opp=${fmtHp(d.opponent_hp)}`,
      x2 + 8 * sx,
      p1y + HP_TL_ROW_H * 0.72 * sy,
    );
    ctx.fillStyle = "rgb(255,180,60)";
    ctx.fillText(
      `raw:  L=${(d.left_hp_raw ?? 0).toFixed(2)} R=${(d.right_hp_raw ?? 0).toFixed(2)}  match=${d.is_match_screen}`,
      x2 + 8 * sx,
      p2y + HP_TL_ROW_H * 0.72 * sy,
    );
  }
}
