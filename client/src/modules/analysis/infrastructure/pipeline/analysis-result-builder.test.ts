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
  left_super_value: 0,
  right_super_value: 0,
  left_super_uncertain: true,
  right_super_uncertain: true,
  left_ca_ready: false,
  right_ca_ready: false,
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
        attackInfo: JSON.stringify([
          {
            frame_index: 3,
            p1: {
              last_damage: 600,
              scaling_percent: 100,
              combo_damage: 600,
              max_combo_damage: 600,
              attribute: "lower",
            },
            p2: {
              last_damage: 0,
              scaling_percent: 100,
              combo_damage: 0,
              max_combo_damage: 0,
              attribute: "upper",
            },
          },
        ]),
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
    expect(result.attackInfo).toHaveLength(1);
    expect(result.attackInfo[0].p1.attribute).toBe("lower");
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
    expect(result.attackInfo).toEqual([]);
    expect(result.hpFeatures).toEqual([]);
    expect(result.spatialObservations).toEqual([]);
  });
});
