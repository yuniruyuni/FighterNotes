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
 * フレームメーターと画面下端の SA ゲージを、1 回の createImageBitmap で
 * 切り出すための下部アトラス。WASM へ渡す strip の大きさは変更せず、
 * 必要な領域だけを各 strip canvas へ描画する。
 */
export const LOWER_ATLAS_LAYOUT = {
  source: {
    x: 55,
    y: ANALYSIS_STRIPS.meter.y,
    width: 1810,
    height: 234,
  },
  meter: {
    source: {
      x: 304,
      y: 0,
      width: 1200,
      height: ANALYSIS_STRIPS.meter.height,
    },
    target: {
      x: 359,
      y: 0,
      width: 1200,
      height: ANALYSIS_STRIPS.meter.height,
    },
  },
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

/**
 * 画面下端の SA ゲージから、数値/CA ラベルと部分ゲージだけを HUD strip の
 * 未使用領域へ埋め込む。HUD 全体の転送量・getImageData 回数は増やさない。
 *
 * bar target は y>=32 のため HP（y<31）と重ならず、左右端に置くことで
 * drive / FIGHT の読み取り範囲にも重ならない。
 */
export const SUPER_GAUGE_LAYOUT = {
  left: {
    label: {
      source: { x: 0, y: 159, width: 90, height: 75 },
      target: { x: 0, y: 0, width: 90, height: 70 },
    },
    bar: {
      source: { x: 90, y: 179, width: 265, height: 50 },
      target: { x: 100, y: 32, width: 265, height: 38 },
    },
  },
  right: {
    bar: {
      source: { x: 1455, y: 179, width: 265, height: 50 },
      target: { x: 1555, y: 32, width: 265, height: 38 },
    },
    label: {
      source: { x: 1720, y: 159, width: 90, height: 75 },
      target: { x: 1830, y: 0, width: 90, height: 70 },
    },
  },
} as const;
