import { describe, expect, test } from "bun:test";
import { initialDebugOverlayVisibility } from "./debug-viewer-model.js";

describe("debug viewer model", () => {
  test("overlayをすべて非表示の独立した状態で初期化する", () => {
    const first = initialDebugOverlayVisibility();
    const second = initialDebugOverlayVisibility();

    expect(first).toEqual({
      raw: false,
      hue: false,
      hp: false,
      drive: false,
      super: false,
      input: false,
    });
    expect(first).not.toBe(second);
  });
});
