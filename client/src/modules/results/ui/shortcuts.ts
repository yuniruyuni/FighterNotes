import type { FrameNavigationAction } from "../domain/frame-navigation.js";

/**
 * 結果画面のキー操作。動画プレイヤーと認識デバッグで同じ表を使う。
 *
 * 同じキーが画面ごとに違う意味になると、見直しの最中に手が止まる。移動キーは
 * 両画面で共通にし、その画面に無い操作（再生、保存など）は無視するだけにする。
 */
export type ViewerShortcutAction =
  | { type: "frame"; move: FrameNavigationAction }
  | { type: "playback" }
  | { type: "loop" }
  | { type: "rate"; direction: -1 | 1 }
  | { type: "sceneStart" }
  | { type: "saveFrame" }
  | { type: "saveFrameData" };

export interface ShortcutModifiers {
  ctrl: boolean;
  shift: boolean;
}

export function shortcutActionForKey(
  key: string,
  modifiers: ShortcutModifiers,
): ViewerShortcutAction | null {
  // Shift を押した英字は "A" で届く。押していない時と同じ意味にする。
  const normalized = key.length === 1 ? key.toLowerCase() : key;

  if (normalized === "ArrowLeft" || normalized === "a") {
    return { type: "frame", move: backward(modifiers) };
  }
  if (normalized === "ArrowRight" || normalized === "d") {
    return { type: "frame", move: forward(modifiers) };
  }
  if (normalized === ",") return { type: "frame", move: "skip-backward" };
  if (normalized === ".") return { type: "frame", move: "skip-forward" };
  if (normalized === "[") return { type: "frame", move: "jump-backward" };
  if (normalized === "]") return { type: "frame", move: "jump-forward" };
  if (normalized === " " || normalized === "k") return { type: "playback" };
  if (normalized === "l") return { type: "loop" };
  if (normalized === "ArrowUp") return { type: "rate", direction: 1 };
  if (normalized === "ArrowDown") return { type: "rate", direction: -1 };
  if (normalized === "Home") return { type: "sceneStart" };
  if (normalized === "s") {
    return { type: modifiers.shift ? "saveFrameData" : "saveFrame" };
  }
  return null;
}

function backward(modifiers: ShortcutModifiers): FrameNavigationAction {
  if (modifiers.ctrl) return "jump-backward";
  if (modifiers.shift) return "skip-backward";
  return "step-backward";
}

function forward(modifiers: ShortcutModifiers): FrameNavigationAction {
  if (modifiers.ctrl) return "jump-forward";
  if (modifiers.shift) return "skip-forward";
  return "step-forward";
}

export interface ShortcutHelpEntry {
  keys: string;
  label: string;
}

/** 両画面に出す共通の移動キー。 */
export const FRAME_SHORTCUT_HELP: readonly ShortcutHelpEntry[] = [
  { keys: "← →", label: "1フレーム" },
  { keys: "Shift + ← →", label: "10フレーム" },
  { keys: "Ctrl + ← →", label: "60フレーム" },
];

/** 両画面が持つ再生の操作。 */
export const PLAYBACK_SHORTCUT_HELP: readonly ShortcutHelpEntry[] = [
  { keys: "Space / K", label: "再生・停止" },
  { keys: "↑ ↓", label: "再生速度" },
];

/** 動画プレイヤーだけが持つ操作。 */
export const PLAYER_SHORTCUT_HELP: readonly ShortcutHelpEntry[] = [
  { keys: "L", label: "区間ループ" },
  { keys: "Home", label: "場面の先頭" },
];

/** 認識デバッグだけが持つ操作。 */
export const DEBUG_SHORTCUT_HELP: readonly ShortcutHelpEntry[] = [
  { keys: "S", label: "画像を保存" },
  { keys: "Shift + S", label: "フレームデータを保存" },
];
