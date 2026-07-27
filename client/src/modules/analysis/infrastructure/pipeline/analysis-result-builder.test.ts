import { describe, expect, test } from "bun:test";
import { buildAnalysisResult } from "./analysis-result-builder.js";

const context = { ownSide: "p1" as const, p1: {}, p2: {} };
const hpFrame = {
  frame_index: 3,
  fps: 60,
  own_hp: 1,
  opponent_hp: 1,
  is_match_screen: true,
  left_hp_score: 1,
  right_hp_score: 1,
  left_drive_ratio: 1,
  right_drive_ratio: 1,
  left_burnout: false,
  right_burnout: false,
  left_drive_uncertain: false,
  right_drive_uncertain: false,
  left_hp_raw: 1,
  right_hp_raw: 1,
};

describe("buildAnalysisResult", () => {
  test("parses worker payloads and retains decode artifacts", () => {
    const videoArrayBuffer = new ArrayBuffer(4);
    const result = buildAnalysisResult(
      {
        type: "done",
        report: JSON.stringify({ ruleset_version: 1 }),
        timeline: JSON.stringify({ entries: [] }),
        trackedInputs: JSON.stringify({ p1: [], p2: [] }),
        features: JSON.stringify([hpFrame]),
        spatialObservations: JSON.stringify([{ frame: 3 }]),
      },
      {
        analysisContext: context,
        frameTimestamps: [0, 1 / 60],
        sampleData: [],
        videoArrayBuffer,
        codecConfig: null,
        frameToSampleIdx: [0, 1],
      },
    );

    expect(result.analysisContext).toBe(context);
    expect(result.videoArrayBuffer).toBe(videoArrayBuffer);
    expect(result.frameCount).toBe(2);
    expect(result.trackedInputs).toEqual({ p1: [], p2: [] });
    expect(result.hpFeatures).toEqual([hpFrame]);
    expect(result.spatialObservations).toEqual([{ frame: 3 }]);
  });

  test("uses empty optional collections when the worker omits them", () => {
    const result = buildAnalysisResult(
      {
        type: "done",
        report: "{}",
        timeline: "{}",
        features: "",
      },
      {
        analysisContext: context,
        frameTimestamps: [],
        sampleData: [],
        videoArrayBuffer: new ArrayBuffer(0),
        codecConfig: null,
        frameToSampleIdx: [],
      },
    );

    expect(result.trackedInputs).toBeNull();
    expect(result.hpFeatures).toEqual([]);
    expect(result.spatialObservations).toEqual([]);
  });
});
