import { describe, expect, test } from "bun:test";
import { PLAYBACK_RATES, stepPlaybackRate } from "./playback-rate.js";

describe("stepPlaybackRate", () => {
  test("用意した速度を1段ずつ移動する", () => {
    expect(PLAYBACK_RATES).toEqual([0.25, 0.5, 1]);
    expect(stepPlaybackRate(0.25, 1)).toBe(0.5);
    expect(stepPlaybackRate(0.5, 1)).toBe(1);
    expect(stepPlaybackRate(1, -1)).toBe(0.5);
    expect(stepPlaybackRate(0.5, -1)).toBe(0.25);
  });

  test("端では留まる", () => {
    expect(stepPlaybackRate(1, 1)).toBe(1);
    expect(stepPlaybackRate(0.25, -1)).toBe(0.25);
  });
});
