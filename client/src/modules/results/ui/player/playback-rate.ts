export const PLAYBACK_RATES = [0.25, 0.5, 1] as const;

export type PlaybackRate = (typeof PLAYBACK_RATES)[number];
