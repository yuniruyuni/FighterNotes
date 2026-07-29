export const ANALYSIS_WIDTH = 1920;
export const ANALYSIS_HEIGHT = 1080;

export interface AnalysisStrip {
  readonly y: number;
  readonly height: number;
  readonly byteLength: number;
}

function strip(y: number, height: number): AnalysisStrip {
  return {
    y,
    height,
    byteLength: ANALYSIS_WIDTH * height * 4,
  };
}

export const ANALYSIS_STRIPS = {
  hud: strip(64, 70),
  input: strip(232, 36),
  meter: strip(796, 78),
} as const;

/**
 * FIGHT の中央画像を低頻度で縮小し、HUD strip の未使用中央領域へ埋め込む。
 * target は HP（x<=853 / x>=1067）と drive（x<=895 / x>=1025）の
 * 読み取り範囲に重ならない。
 */
export const FIGHT_MARKER_LAYOUT = {
  sampleInterval: 4,
  source: {
    x: 400,
    y: 300,
    width: 1120,
    height: 455,
  },
  target: {
    x: 896,
    y: 9,
    width: 128,
    height: 52,
  },
} as const;
