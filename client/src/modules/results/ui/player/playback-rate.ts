export const PLAYBACK_RATES = [0.25, 0.5, 1] as const;

export type PlaybackRate = (typeof PLAYBACK_RATES)[number];

/** 用意した速度を1段ずつ移動する。端では留まる。 */
export function stepPlaybackRate(
  current: PlaybackRate,
  direction: -1 | 1,
): PlaybackRate {
  const index = PLAYBACK_RATES.indexOf(current) + direction;
  const clamped = Math.max(0, Math.min(PLAYBACK_RATES.length - 1, index));
  return PLAYBACK_RATES[clamped] as PlaybackRate;
}
