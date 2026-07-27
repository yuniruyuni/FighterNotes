export type DebugFrameNavigationAction =
  | "jump-backward"
  | "skip-backward"
  | "step-backward"
  | "step-forward"
  | "skip-forward"
  | "jump-forward";

const FRAME_DELTA: Readonly<Record<DebugFrameNavigationAction, number>> = {
  "jump-backward": -60,
  "skip-backward": -10,
  "step-backward": -1,
  "step-forward": 1,
  "skip-forward": 10,
  "jump-forward": 60,
};

function clampDebugFrame(index: number, totalFrames: number): number {
  return Math.max(0, Math.min(index, Math.max(0, totalFrames - 1)));
}

function debugFrameDelta(action: DebugFrameNavigationAction): number {
  return FRAME_DELTA[action];
}

function moveDebugFrame(
  current: number,
  totalFrames: number,
  action: DebugFrameNavigationAction,
): number {
  return clampDebugFrame(current + debugFrameDelta(action), totalFrames);
}

export const DebugFrameNavigation = {
  clamp: clampDebugFrame,
  delta: debugFrameDelta,
  move: moveDebugFrame,
};
