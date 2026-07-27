import { describe, expect, test } from "bun:test";
import {
  DebugFrameNavigation,
  type DebugFrameNavigationAction,
} from "./debug-frame-navigation.js";

describe("DebugFrameNavigation", () => {
  test("意味のある操作をフレーム移動量へ変換する", () => {
    const cases: Array<[DebugFrameNavigationAction, number]> = [
      ["jump-backward", -60],
      ["skip-backward", -10],
      ["step-backward", -1],
      ["step-forward", 1],
      ["skip-forward", 10],
      ["jump-forward", 60],
    ];

    for (const [action, expected] of cases) {
      expect(DebugFrameNavigation.delta(action)).toBe(expected);
    }
  });

  test("操作後のカーソルを解析フレーム範囲へ収める", () => {
    expect(DebugFrameNavigation.move(50, 100, "skip-backward")).toBe(40);
    expect(DebugFrameNavigation.move(5, 100, "skip-backward")).toBe(0);
    expect(DebugFrameNavigation.move(95, 100, "skip-forward")).toBe(99);
    expect(DebugFrameNavigation.move(0, 0, "step-forward")).toBe(0);
    expect(DebugFrameNavigation.clamp(45, 100)).toBe(45);
  });
});
