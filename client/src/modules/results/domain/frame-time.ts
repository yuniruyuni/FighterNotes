// プレイヤーの純ロジック（DOM 非依存）。bun test で検査する。

export const FPS = 60;
export const CLIP_BEFORE_SECONDS = 1.5;
export const CLIP_AFTER_SECONDS = 1;
// 既存の60fpsフォールバックとテスト用の換算値。
export const CLIP_BEFORE_FRAMES = CLIP_BEFORE_SECONDS * FPS;
export const CLIP_AFTER_FRAMES = CLIP_AFTER_SECONDS * FPS;

export interface ClipRange {
  startSec: number;
  endSec: number;
}

/** 動画フレーム番号を実際のメディア時刻へ変換する。 */
export function frameToSeconds(
  frame: number,
  // Stryker disable next-line ArrayDeclaration: Any non-numeric default member is ignored and is observationally equivalent to no timestamps.
  frameTimestamps: readonly number[] = [],
): number {
  const index = Math.max(0, Math.round(frame));
  const exact = frameTimestamps[index];
  if (Number.isFinite(exact)) return exact;

  const first = frameTimestamps[0];
  const lastIndex = frameTimestamps.length - 1;
  const last = frameTimestamps[lastIndex];
  const step = (last - first) / lastIndex;
  if (Number.isFinite(step) && step > 0) {
    return first + index * step;
  }
  return index / FPS;
}

/** 実際のメディア時刻に最も近い動画フレーム番号を返す。 */
export function secondsToFrame(
  seconds: number,
  // Stryker disable next-line ArrayDeclaration: Any non-numeric default member is ignored and is observationally equivalent to no timestamps.
  frameTimestamps: readonly number[] = [],
): number {
  const time = Math.max(0, seconds);
  if (frameTimestamps.length === 0) return Math.round(time * FPS);

  const insertionIndex = lowerBound(frameTimestamps, time);
  const after = Math.min(insertionIndex, frameTimestamps.length - 1);
  const before = Math.max(0, after - 1);
  return time - frameTimestamps[before] <= frameTimestamps[after] - time
    ? before
    : after;
}

function lowerBound(values: readonly number[], target: number): number {
  let low = 0;
  let high = values.length;
  for (const _value of values) {
    // Stryker disable next-line ConditionalExpression,EqualityOperator: Continuing after convergence preserves the same lower bound; this break only keeps the search logarithmic.
    if (low >= high) break;
    const middle = low + Math.floor((high - low) / 2);
    if (values[middle] < target) low = middle + 1;
    else high = middle;
  }
  return low;
}

/** 証拠クリップの再生範囲。endFrame があれば frame→endFrame の区間クリップ。 */
export function clipRange(
  frame: number,
  endFrame?: number,
  // Stryker disable next-line ArrayDeclaration: Any non-numeric default member is ignored and is observationally equivalent to no timestamps.
  frameTimestamps: readonly number[] = [],
): ClipRange {
  const startSec = Math.max(
    0,
    frameToSeconds(frame, frameTimestamps) - CLIP_BEFORE_SECONDS,
  );
  const endSec =
    frameToSeconds(endFrame ?? frame, frameTimestamps) + CLIP_AFTER_SECONDS;
  return { startSec, endSec };
}

/**
 * 区間ループの巻き戻し判定。
 * 「終端を内側から通過した」ときだけ true（区間外へのシークでは巻き戻さない
 * ため、単純な t >= endSec 判定にしない。seeking で prevSec を追従させる前提）。
 */
export function shouldLoopBack(
  loopEnabled: boolean,
  range: ClipRange | null,
  prevSec: number,
  currentSec: number,
): boolean {
  return (
    loopEnabled &&
    range !== null &&
    prevSec < range.endSec &&
    currentSec >= range.endSec
  );
}
