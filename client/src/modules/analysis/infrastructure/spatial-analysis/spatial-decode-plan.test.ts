import { describe, expect, test } from "bun:test";
import type {
  FrameSample,
  SpatialCandidateWindow,
} from "../../domain/result.js";
import { SpatialDecodePlan, spatialHintsAt } from "./spatial-decode-plan.js";

const samples: FrameSample[] = [
  { isSync: true, timestampUs: 0, offset: 0, size: 10 },
  { isSync: false, timestampUs: 30, offset: 10, size: 10 },
  { isSync: false, timestampUs: 10, offset: 20, size: 10 },
  { isSync: true, timestampUs: 40, offset: 30, size: 10 },
];

const window: SpatialCandidateWindow = {
  start_frame: 1,
  end_frame: 2,
  teleport_hints: [{ side: 2, start_frame: 1, end_frame: 1 }],
  airborne_hints: [{ side: 1, start_frame: 2, end_frame: 3 }],
};

describe("SpatialDecodePlan", () => {
  test("候補frameだけを選び、decodeは直前keyframeから始める", () => {
    expect(SpatialDecodePlan.create(window, samples, [0, 2, 1, 3])).toEqual({
      firstSampleIndex: 0,
      lastSampleIndex: 2,
      targets: [
        { timestampUs: 10, frameIndex: 1 },
        { timestampUs: 30, frameIndex: 2 },
      ],
    });
  });

  test("teleportとairborne hintをside別に投影する", () => {
    expect(spatialHintsAt(window, 1)).toEqual({
      p1Teleport: false,
      p2Teleport: true,
      p1Airborne: false,
      p2Airborne: false,
    });
    expect(spatialHintsAt(window, 2).p1Airborne).toBe(true);
  });
});
