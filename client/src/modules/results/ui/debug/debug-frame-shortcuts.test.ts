import { describe, expect, test } from "bun:test";
import { navigationActionForKey } from "./debug-frame-shortcuts.js";

const modifiers = { ctrl: false, shift: false };

describe("debug frame shortcuts", () => {
  test("左右のキーを修飾キーに応じた移動操作へ変換する", () => {
    expect(navigationActionForKey("ArrowLeft", modifiers)).toBe(
      "step-backward",
    );
    expect(navigationActionForKey("a", { ctrl: false, shift: true })).toBe(
      "skip-backward",
    );
    expect(
      navigationActionForKey("ArrowLeft", { ctrl: true, shift: true }),
    ).toBe("jump-backward");
    expect(navigationActionForKey("d", modifiers)).toBe("step-forward");
    expect(
      navigationActionForKey("ArrowRight", { ctrl: false, shift: true }),
    ).toBe("skip-forward");
    expect(navigationActionForKey("d", { ctrl: true, shift: true })).toBe(
      "jump-forward",
    );
  });

  test("記号キーを固定幅の移動操作へ変換する", () => {
    expect(navigationActionForKey(",", modifiers)).toBe("skip-backward");
    expect(navigationActionForKey(".", modifiers)).toBe("skip-forward");
    expect(navigationActionForKey("[", modifiers)).toBe("jump-backward");
    expect(navigationActionForKey("]", modifiers)).toBe("jump-forward");
    expect(navigationActionForKey("x", modifiers)).toBeNull();
  });
});
