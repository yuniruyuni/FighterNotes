import { describe, expect, test } from "bun:test";
import type { FrameSample } from "../../domain/result.js";
import {
  FrameDecodePlan,
  precedingKeyframeIndex,
} from "./frame-decode-plan.js";

const samples: FrameSample[] = [
  { isSync: true, timestampUs: 0, offset: 0, size: 10 },
  { isSync: false, timestampUs: 30, offset: 10, size: 10 },
  { isSync: false, timestampUs: 10, offset: 20, size: 10 },
  { isSync: true, timestampUs: 40, offset: 30, size: 10 },
];

describe("FrameDecodePlan", () => {
  test("表示順frameをdecode順sampleと直前keyframeへ対応付ける", () => {
    expect(FrameDecodePlan.create(samples, [0, 2, 1, 3], 1)).toEqual({
      firstSampleIndex: 0,
      lastSampleIndex: 2,
      targetTimestampUs: 10,
    });
    expect(precedingKeyframeIndex(samples, 3)).toBe(3);
  });

  test("sample対応がないframeはdecodeしない", () => {
    expect(FrameDecodePlan.create(samples, [-1], 0)).toBeNull();
  });
});
