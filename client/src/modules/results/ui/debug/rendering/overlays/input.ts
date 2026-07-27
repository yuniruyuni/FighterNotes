// 入力履歴デバッグオーバーレイ。

import type { TrackedInputRow } from "~/modules/analysis/contracts.js";
import type { DebugInputInspection } from "../../../../application/debug-frame-inspection.js";

// ─── 入力履歴デバッグ表示 ────────────────────────────────────────────────────

// 入力パネルのジオメトリ（Rust 側 input_history.rs と一致）
const INPUT_ROW0_Y = 236;
const INPUT_ROW_PITCH = 34;
// 読み取り結果テキストの表示位置（実パネルのすぐ内側）
const INPUT_TEXT_X: Record<string, number> = { p1: 196, p2: 1566 };

// 方向の矢印表示
const DIR_ARROWS: Record<string, string> = {
  N: "N",
  U: "↑",
  UR: "↗",
  R: "→",
  DR: "↘",
  D: "↓",
  DL: "↙",
  L: "←",
  UL: "↖",
  "?": "?",
};

/// 各行の読み取り結果を実パネルの隣にテキスト表示する。
/// 緑 = 確実、赤 = uncertain、バッジは色文字（g/y/r/b 円、大文字 = 箱）
export function drawInputHistoryDebug(
  ctx: CanvasRenderingContext2D,
  inp: DebugInputInspection,
  cw: number,
  ch: number,
): void {
  const sx = cw / 1920,
    sy = ch / 1080;
  const fontPx = Math.round(13 * sy);
  ctx.font = `bold ${fontPx}px monospace`;

  for (const side of ["p1", "p2"] as const) {
    const rows = inp[side].rows;
    const tx = INPUT_TEXT_X[side] * sx;
    for (let ri = 0; ri < rows.length; ri++) {
      const r = rows[ri];
      if (r.empty) continue;
      const y = (INPUT_ROW0_Y + INPUT_ROW_PITCH * ri) * sy;

      const cnt = r.count === null ? "?" : String(r.count);
      const dir = DIR_ARROWS[r.dir] ?? r.dir;
      const badge = r.badges ? ` ${r.badges}` : "";
      const auto = r.auto ? " AUTO" : "";
      const thr = r.throw ? " THROW" : "";
      const text = `${cnt.padStart(3)} ${dir}${badge}${auto}${thr}`;

      // 背景ボックス
      const tw = ctx.measureText(text).width;
      ctx.fillStyle = "rgba(10,10,10,0.75)";
      ctx.fillRect(tx - 2 * sx, y, tw + 6 * sx, 17 * sy);

      ctx.fillStyle = r.uncertain ? "rgb(255,90,90)" : "rgb(120,235,120)";
      ctx.fillText(text, tx, y + 13.5 * sy);
    }
  }
}

/** 確定層（入力トラッカー補修後）の row0 を実パネル読みの上に並記する。
 *  生読み（drawInputHistoryDebug）と見比べて、分析が実際に使う値を確認できる。 */
export function drawTrackedInputRow0(
  ctx: CanvasRenderingContext2D,
  tracked: { p1: TrackedInputRow[]; p2: TrackedInputRow[] } | null,
  frameIdx: number,
  cw: number,
  ch: number,
): void {
  if (!tracked) return;
  const sx = cw / 1920,
    sy = ch / 1080;
  const fontPx = Math.round(13 * sy);
  ctx.font = `bold ${fontPx}px monospace`;

  for (const side of ["p1", "p2"] as const) {
    const t = tracked[side]?.[frameIdx];
    if (!t) continue;
    const cnt = t.count === null ? "?" : String(t.count);
    const dir = DIR_ARROWS[t.dir] ?? t.dir;
    const badge = t.badges ? ` ${t.badges}` : "";
    const auto = t.auto ? " AUTO" : "";
    const thr = t.throw ? " THROW" : "";
    const mark = t.uncertain ? " ?" : t.repaired ? " ›補修" : "";
    const text = `確定 ${cnt.padStart(3)} ${dir}${badge}${auto}${thr}${mark}`;

    const tx = INPUT_TEXT_X[side] * sx;
    const y = (INPUT_ROW0_Y - 22) * sy; // row0 の 1 行上に表示
    const tw = ctx.measureText(text).width;
    ctx.fillStyle = "rgba(10,10,30,0.8)";
    ctx.fillRect(tx - 2 * sx, y, tw + 6 * sx, 17 * sy);
    ctx.fillStyle = t.uncertain
      ? "rgb(255,150,80)"
      : t.repaired
        ? "rgb(255,220,90)"
        : "rgb(110,200,255)";
    ctx.fillText(text, tx, y + 13.5 * sy);
  }
}
