export type FrameNavigationAction =
  | "jump-backward"
  | "skip-backward"
  | "step-backward"
  | "step-forward"
  | "skip-forward"
  | "jump-forward";

const FRAME_DELTA: Readonly<Record<FrameNavigationAction, number>> = {
  "jump-backward": -60,
  "skip-backward": -10,
  "step-backward": -1,
  "step-forward": 1,
  "skip-forward": 10,
  "jump-forward": 60,
};

function clampFrame(index: number, totalFrames: number): number {
  return Math.max(0, Math.min(index, Math.max(0, totalFrames - 1)));
}

function frameDelta(action: FrameNavigationAction): number {
  return FRAME_DELTA[action];
}

function moveFrame(
  current: number,
  totalFrames: number,
  action: FrameNavigationAction,
): number {
  return clampFrame(current + frameDelta(action), totalFrames);
}

export const FrameNavigation = {
  clamp: clampFrame,
  delta: frameDelta,
  move: moveFrame,
};
