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
