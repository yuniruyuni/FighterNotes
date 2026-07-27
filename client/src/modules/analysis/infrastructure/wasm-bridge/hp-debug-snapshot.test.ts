import { describe, expect, test } from "bun:test";
import { buildHpDebugSnapshot } from "./hp-debug-snapshot.js";

describe("buildHpDebugSnapshot", () => {
  test("概要と500frameごとのHP読取値を表示用に整形する", () => {
    const features = Array.from({ length: 501 }, (_, index) => ({
      frame_index: index,
      is_match_screen: index < 2,
      left_hp_score: 0.12345,
      right_hp_score: "unknown",
      left_hp_raw: 0.5,
      right_hp_raw: 0.4,
      own_hp: index === 0 ? 0.75 : -1,
      opponent_hp: 0.25,
      left_hp_raw_quality: 0.987,
    }));

    expect(buildHpDebugSnapshot(JSON.stringify(features))).toEqual([
      { summary: "total=501 match_frames=2" },
      {
        fi: 0,
        match: true,
        lscore: "0.123",
        rscore: "?",
        lraw: "0.500",
        rraw: "0.400",
        own_hp: "0.750",
        opp_hp: "0.250",
        lqual: "0.99",
      },
      {
        fi: 500,
        match: false,
        lscore: "0.123",
        rscore: "?",
        lraw: "0.500",
        rraw: "0.400",
        own_hp: "?",
        opp_hp: "0.250",
        lqual: "0.99",
      },
    ]);
  });

  test("壊れたfeatures JSONはdebug情報なしとして扱う", () => {
    expect(buildHpDebugSnapshot("not-json")).toEqual([]);
  });
});
