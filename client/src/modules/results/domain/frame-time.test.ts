import { describe, expect, test } from "bun:test";
import {
  CLIP_AFTER_FRAMES,
  CLIP_AFTER_SECONDS,
  CLIP_BEFORE_FRAMES,
  CLIP_BEFORE_SECONDS,
  clipRange,
  FPS,
  frameToSeconds,
  secondsToFrame,
  shouldLoopBack,
} from "./frame-time";

describe("clipRange", () => {
  test("単点クリップは frame の ±固定窓", () => {
    const r = clipRange(600);
    expect(r.startSec).toBeCloseTo((600 - CLIP_BEFORE_FRAMES) / FPS, 6);
    expect(r.endSec).toBeCloseTo((600 + CLIP_AFTER_FRAMES) / FPS, 6);
  });

  test("区間クリップは frame→endFrame + 後方マージン", () => {
    // lead_loss の実例: R1 最大リード f2246 → 逆転 f2849
    const r = clipRange(2246, 2849);
    expect(r.startSec).toBeCloseTo((2246 - CLIP_BEFORE_FRAMES) / FPS, 6);
    expect(r.endSec).toBeCloseTo((2849 + CLIP_AFTER_FRAMES) / FPS, 6);
  });

  test("動画先頭付近では開始が負にならない", () => {
    const r = clipRange(30);
    expect(r.startSec).toBe(0);
  });

  test("endFrame = frame は単点と同じ", () => {
    expect(clipRange(600, 600)).toEqual(clipRange(600));
  });

  test("29.97fps動画は実タイムスタンプで証拠区間を求める", () => {
    const timestamps = Array.from(
      { length: 5845 },
      (_, frame) => (frame * 1001) / 30000,
    );
    const r = clipRange(2039, 2040, timestamps);
    expect(frameToSeconds(2039, timestamps)).toBeCloseTo(68.0346, 3);
    expect(r.startSec).toBeCloseTo(timestamps[2039] - CLIP_BEFORE_SECONDS, 6);
    expect(r.endSec).toBeCloseTo(timestamps[2040] + CLIP_AFTER_SECONDS, 6);
    expect(r.startSec).not.toBeCloseTo((2039 - CLIP_BEFORE_FRAMES) / FPS, 1);
  });
});

describe("frame/time conversion", () => {
  const timestamps = Array.from(
    { length: 300 },
    (_, frame) => (frame * 1001) / 30000,
  );

  test("実時刻から最も近い動画フレームへ戻せる", () => {
    expect(secondsToFrame(timestamps[120], timestamps)).toBe(120);
    expect(secondsToFrame(timestamps[120] + 0.005, timestamps)).toBe(120);
    expect(secondsToFrame(timestamps[120] + 0.03, timestamps)).toBe(121);
  });

  test("不等間隔timestampの観測値を外挿より優先する", () => {
    expect(frameToSeconds(1, [0, 0.1, 0.4])).toBe(0.1);
  });

  test("タイムスタンプが無い場合は60fpsへフォールバックする", () => {
    expect(frameToSeconds(600)).toBe(10);
    expect(secondsToFrame(10)).toBe(600);
  });

  test("記録範囲外のframeは観測stepで外挿する", () => {
    expect(frameToSeconds(4, [1, 1.5])).toBe(3);
    expect(frameToSeconds(4, [1, 1.5, 2])).toBe(3);
    expect(frameToSeconds(4, [1, 1])).toBeCloseTo(4 / FPS);
    expect(frameToSeconds(-2, [1, 1.5])).toBe(1);
  });

  test("時刻を先頭・末尾へclampし、等距離なら手前を選ぶ", () => {
    expect(secondsToFrame(-1, timestamps)).toBe(0);
    expect(secondsToFrame(999, timestamps)).toBe(timestamps.length - 1);
    const midpoint = (timestamps[10] + timestamps[11]) / 2;
    expect(secondsToFrame(midpoint, timestamps)).toBe(10);
    expect(secondsToFrame(1, [0, 1, 1, 2])).toBe(1);
  });
});

describe("shouldLoopBack", () => {
  const range = { startSec: 10, endSec: 20 };

  test("終端を内側から通過したら巻き戻す", () => {
    expect(shouldLoopBack(true, range, 19.9, 20.05)).toBe(true);
  });

  test("区間外へのシーク（prevSec が既に終端以降）では巻き戻さない", () => {
    // seeking イベントで prevSec を追従させた後の状態
    expect(shouldLoopBack(true, range, 25, 25.1)).toBe(false);
  });

  test("区間内の通常再生では巻き戻さない", () => {
    expect(shouldLoopBack(true, range, 15, 15.2)).toBe(false);
  });

  test("ループ無効なら巻き戻さない", () => {
    expect(shouldLoopBack(false, range, 19.9, 20.05)).toBe(false);
  });

  test("範囲未設定なら巻き戻さない", () => {
    expect(shouldLoopBack(true, null, 19.9, 20.05)).toBe(false);
  });

  test("ちょうど終端に到達した場合も巻き戻す（t >= end）", () => {
    expect(shouldLoopBack(true, range, 19.99, 20)).toBe(true);
    expect(shouldLoopBack(true, range, 20, 20.1)).toBe(false);
  });
});
