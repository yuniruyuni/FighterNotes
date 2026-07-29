import { describe, expect, test } from "bun:test";
import {
  compareTimings,
  evaluateExpectations,
  parseLocalVideoManifest,
} from "./local-video-e2e-lib";

describe("local video E2E manifest", () => {
  test("parses a valid local-only case", () => {
    expect(
      parseLocalVideoManifest({
        schemaVersion: 1,
        cases: [
          {
            id: "sample-1",
            videoPath: "/private/replay.mp4",
            side: "p2",
            ownCharacter: "KEN",
            opponentCharacter: "JURI",
          },
        ],
      }),
    ).toEqual({
      schemaVersion: 1,
      cases: [
        {
          id: "sample-1",
          videoPath: "/private/replay.mp4",
          side: "p2",
          ownCharacter: "KEN",
          opponentCharacter: "JURI",
        },
      ],
    });
  });

  test("rejects duplicate and unsafe output ids", () => {
    expect(() =>
      parseLocalVideoManifest({
        schemaVersion: 1,
        cases: [
          {
            id: "../escape",
            videoPath: "/private/replay.mp4",
            side: "p1",
            ownCharacter: "KEN",
            opponentCharacter: "JURI",
          },
        ],
      }),
    ).toThrow("safe for a file name");
  });
});

describe("local video E2E expectations", () => {
  const report = {
    rounds_detected: 2,
    round_summaries: [{ won: true }, { won: false }],
    cards: [{ id: "anti_air" }],
    input_stats: { jumps: 0 },
    tactic_stats: { di_faced: 2, anti_air_successes: 1 },
    coverage: { match_frames: 100, analyzed_match_frames: 90 },
  };

  test("accepts exact, range, list and coverage checks", () => {
    expect(
      evaluateExpectations(report, {
        roundsDetected: 2,
        roundWinners: [true, false],
        cardIds: { include: ["anti_air"], exclude: ["mashing"] },
        inputStats: { jumps: 0 },
        tacticStats: {
          di_faced: { min: 1 },
          anti_air_successes: { equals: 1, max: 1 },
        },
        coverageRatio: { min: 0.85 },
      }),
    ).toEqual([]);
  });

  test("reports every failed invariant", () => {
    const failures = evaluateExpectations(report, {
      roundsDetected: 3,
      roundWinners: [false],
      cardIds: { include: ["mashing"], exclude: ["anti_air"] },
      inputStats: { jumps: { min: 1 } },
      tacticStats: { di_faced: { max: 1 } },
      coverageRatio: { min: 0.95 },
    });

    expect(failures).toHaveLength(7);
    expect(failures.join("\n")).toContain("rounds_detected");
    expect(failures.join("\n")).toContain("coverage ratio");
  });
});

test("timing comparison only includes matching baselines", () => {
  expect(
    compareTimings({ first: 800, second: 200 }, { first: 1_000, third: 500 }),
  ).toEqual([
    {
      id: "first",
      currentMs: 800,
      baselineMs: 1_000,
      ratio: 0.8,
    },
  ]);
});
