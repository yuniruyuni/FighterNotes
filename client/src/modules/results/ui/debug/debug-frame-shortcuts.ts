import type { DebugFrameNavigationAction } from "../../domain/debug-frame-navigation.js";

export interface DebugFrameKeyboardModifiers {
  ctrl: boolean;
  shift: boolean;
}

export function navigationActionForKey(
  key: string,
  modifiers: DebugFrameKeyboardModifiers,
): DebugFrameNavigationAction | null {
  if (key === "ArrowLeft" || key === "a") {
    if (modifiers.ctrl) return "jump-backward";
    if (modifiers.shift) return "skip-backward";
    return "step-backward";
  }
  if (key === "ArrowRight" || key === "d") {
    if (modifiers.ctrl) return "jump-forward";
    if (modifiers.shift) return "skip-forward";
    return "step-forward";
  }
  if (key === ",") return "skip-backward";
  if (key === ".") return "skip-forward";
  if (key === "[") return "jump-backward";
  if (key === "]") return "jump-forward";
  return null;
}
