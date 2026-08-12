import { describe, expect, test } from "bun:test";
import {
  DEBUG_SHORTCUT_HELP,
  FRAME_SHORTCUT_HELP,
  PLAYBACK_SHORTCUT_HELP,
  PLAYER_SHORTCUT_HELP,
  shortcutActionForKey,
} from "./shortcuts.js";

const plain = { ctrl: false, shift: false };

describe("結果画面のキー操作", () => {
  test("左右のキーを修飾キーに応じた移動へ変換する", () => {
    expect(shortcutActionForKey("ArrowLeft", plain)).toEqual({
      type: "frame",
      move: "step-backward",
    });
    expect(shortcutActionForKey("a", { ctrl: false, shift: true })).toEqual({
      type: "frame",
      move: "skip-backward",
    });
    expect(
      shortcutActionForKey("ArrowLeft", { ctrl: true, shift: true }),
    ).toEqual({ type: "frame", move: "jump-backward" });
    expect(shortcutActionForKey("d", plain)).toEqual({
      type: "frame",
      move: "step-forward",
    });
    expect(
      shortcutActionForKey("ArrowRight", { ctrl: false, shift: true }),
    ).toEqual({ type: "frame", move: "skip-forward" });
    expect(shortcutActionForKey("d", { ctrl: true, shift: true })).toEqual({
      type: "frame",
      move: "jump-forward",
    });
  });

  test("記号キーを固定幅の移動へ変換する", () => {
    expect(shortcutActionForKey(",", plain)).toEqual({
      type: "frame",
      move: "skip-backward",
    });
    expect(shortcutActionForKey(".", plain)).toEqual({
      type: "frame",
      move: "skip-forward",
    });
    expect(shortcutActionForKey("[", plain)).toEqual({
      type: "frame",
      move: "jump-backward",
    });
    expect(shortcutActionForKey("]", plain)).toEqual({
      type: "frame",
      move: "jump-forward",
    });
  });

  test("再生とループと速度と場面先頭を区別する", () => {
    expect(shortcutActionForKey(" ", plain)).toEqual({ type: "playback" });
    expect(shortcutActionForKey("k", plain)).toEqual({ type: "playback" });
    expect(shortcutActionForKey("l", plain)).toEqual({ type: "loop" });
    expect(shortcutActionForKey("ArrowUp", plain)).toEqual({
      type: "rate",
      direction: 1,
    });
    expect(shortcutActionForKey("ArrowDown", plain)).toEqual({
      type: "rate",
      direction: -1,
    });
    expect(shortcutActionForKey("Home", plain)).toEqual({ type: "sceneStart" });
    expect(shortcutActionForKey("s", plain)).toEqual({ type: "saveFrame" });
    expect(shortcutActionForKey("S", { ctrl: false, shift: true })).toEqual({
      type: "saveFrameData",
    });
  });

  test("Shiftを押した英字も同じ意味にする", () => {
    const shifted = { ctrl: false, shift: true };
    expect(shortcutActionForKey("K", shifted)).toEqual({ type: "playback" });
    expect(shortcutActionForKey("L", shifted)).toEqual({ type: "loop" });
  });

  test("割り当てのないキーは受け取らない", () => {
    expect(shortcutActionForKey("x", plain)).toBeNull();
    expect(shortcutActionForKey("Enter", plain)).toBeNull();
    expect(shortcutActionForKey("Escape", plain)).toBeNull();
  });

  test("操作一覧は画面に出す表そのものを持つ", () => {
    expect(FRAME_SHORTCUT_HELP).toEqual([
      { keys: "← →", label: "1フレーム" },
      { keys: "Shift + ← →", label: "10フレーム" },
      { keys: "Ctrl + ← →", label: "60フレーム" },
    ]);
    expect(PLAYBACK_SHORTCUT_HELP).toEqual([
      { keys: "Space / K", label: "再生・停止" },
      { keys: "↑ ↓", label: "再生速度" },
    ]);
    expect(PLAYER_SHORTCUT_HELP).toEqual([
      { keys: "L", label: "区間ループ" },
      { keys: "Home", label: "場面の先頭" },
    ]);
    expect(DEBUG_SHORTCUT_HELP).toEqual([
      { keys: "S", label: "画像を保存" },
      { keys: "Shift + S", label: "フレームデータを保存" },
    ]);
  });
});
