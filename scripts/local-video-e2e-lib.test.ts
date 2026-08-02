import { describe, expect, test } from "bun:test";
import {
  compareDetectorMetrics,
  comparePerformance,
  compareTimings,
  diffSemanticValues,
  evaluateExpectations,
  evaluateRegressionEvents,
  parseLocalVideoManifest,
  semanticSnapshot,
  summarizeTimings,
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

  test("validates semantic annotations and performance policy", () => {
    const manifest = parseLocalVideoManifest({
      schemaVersion: 1,
      performance: {
        measuredRuns: 3,
        warmupRuns: 1,
        maxMedianRegressionRatio: 1.08,
      },
      cases: [
        {
          id: "annotated",
          videoPath: "/private/replay.mp4",
          side: "p1",
          ownCharacter: "KEN",
          opponentCharacter: "JURI",
          expect: {
            semanticEvents: [
              {
                id: "round-1-fight",
                detector: "fight",
                frame: { min: 950, max: 990 },
                roundNo: 1,
              },
            ],
            detectorGates: {
              fight: { maxFalsePositives: 0, minRecall: 1 },
            },
          },
        },
      ],
    });

    expect(manifest.performance?.measuredRuns).toBe(3);
    expect(manifest.cases[0].expect?.semanticEvents?.[0].id).toBe(
      "round-1-fight",
    );
    expect(() =>
      parseLocalVideoManifest({
        schemaVersion: 1,
        cases: [
          {
            id: "bad-range",
            videoPath: "/private/replay.mp4",
            side: "p1",
            ownCharacter: "KEN",
            opponentCharacter: "JURI",
            expect: {
              semanticEvents: [
                {
                  id: "bad",
                  detector: "damage",
                  frame: { min: 20, max: 10 },
                },
              ],
            },
          },
        ],
      }),
    ).toThrow("0 <= min <= max");
  });

  test("rejects unknown fields and mistyped optional contracts", () => {
    const baseCase = {
      id: "strict",
      videoPath: "/private/replay.mp4",
      side: "p1",
      ownCharacter: "KEN",
      opponentCharacter: "JURI",
    } as const;
    expect(() =>
      parseLocalVideoManifest({
        schemaVersion: 1,
        cases: [{ ...baseCase, expect: { semanticEvent: [] } }],
      }),
    ).toThrow("unknown field");
    expect(() =>
      parseLocalVideoManifest({
        schemaVersion: 1,
        cases: [{ ...baseCase, browserVideoPath: 42 }],
      }),
    ).toThrow("browserVideoPath must be a string");
    expect(() =>
      parseLocalVideoManifest({
        schemaVersion: 1,
        cases: [{ ...baseCase, expect: { cardIds: [] } }],
      }),
    ).toThrow("cardIds must be an object");
    expect(() =>
      parseLocalVideoManifest({
        schemaVersion: 1,
        cases: [
          {
            ...baseCase,
            expect: {
              semanticEvents: [
                {
                  id: "bad-metadata",
                  detector: "fight",
                  frame: 100,
                  hpDrop: 0.2,
                },
              ],
            },
          },
        ],
      }),
    ).toThrow("unknown field");
    expect(() =>
      parseLocalVideoManifest({
        schemaVersion: 1,
        cases: [
          {
            ...baseCase,
            expect: {
              detectorGates: { damage: { minRecell: 1 } },
            },
          },
        ],
      }),
    ).toThrow("unknown field");
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

describe("semantic event regression", () => {
  const artifacts = {
    fightMarkers: [{ peak_frame: 976 }],
    regressionEvents: {
      rounds: [{ round_no: 1, start_frame: 1024, end_frame: 4500, winner: 1 }],
      damage: [
        {
          victim: 2,
          start_frame: 1767,
          end_frame: 1810,
          drop: 0.24,
          round_no: 1,
        },
      ],
      super_arts: [
        {
          side: 1,
          frame: 2000,
          level: 3,
          critical_art: false,
          damage: 0.2,
          round_no: 1,
        },
      ],
      attack_evidence: {
        sequences: [
          {
            attacker: 1,
            start_frame: 1770,
            end_frame: 1810,
            combo_damage: 2400,
            starter_attribute: "lower",
            final_attribute: "lower",
            complete: true,
            recovered_from_max: false,
          },
        ],
        damage: [
          {
            victim: 2,
            attacker: 1,
            damage_start_frame: 1767,
            sequence_start_frame: 1770,
            sequence_end_frame: 1810,
            combo_damage: 2400,
            starter_attribute: "lower",
          },
        ],
        super_arts: [{ side: 1, super_frame: 2000, combo_damage: 3000 }],
      },
    },
    report: {
      cards: [
        {
          id: "big_hits",
          evidence: [{ frame: 1767, end_frame: 1810 }],
        },
      ],
    },
  };

  test("matches round, FIGHT, damage, SA, attack info and advice evidence", () => {
    const semanticEvents = [
      {
        id: "round",
        detector: "round" as const,
        frame: { min: 1020, max: 1030 },
        roundNo: 1,
        winner: "p1" as const,
        syntheticTest: "pipeline_contract::round",
      },
      {
        id: "fight",
        detector: "fight" as const,
        frame: { min: 970, max: 980 },
      },
      {
        id: "damage",
        detector: "damage" as const,
        frame: { min: 1765, max: 1770 },
        side: "p2" as const,
        roundNo: 1,
        hpDrop: { min: 0.23, max: 0.25 },
        reportedDamage: 2400,
        attribute: "lower" as const,
      },
      {
        id: "super",
        detector: "super" as const,
        frame: 2000,
        side: "p1" as const,
        superLevel: 3 as const,
        criticalArt: false,
        reportedDamage: 3000,
      },
      {
        id: "attack-info",
        detector: "attackInfo" as const,
        frame: { min: 1768, max: 1772 },
        side: "p1" as const,
        reportedDamage: 2400,
        attribute: "lower" as const,
      },
      {
        id: "attack-info-attribution",
        detector: "attackInfoAttribution" as const,
        frame: { min: 1768, max: 1772 },
        side: "p1" as const,
        reportedDamage: 2400,
        attribute: "lower" as const,
      },
      {
        id: "advice",
        detector: "adviceEvidence" as const,
        frame: 1767,
        cardId: "big_hits",
      },
    ];
    const result = evaluateRegressionEvents(artifacts, {
      semanticEvents,
      detectorGates: Object.fromEntries(
        semanticEvents.map((event) => [
          event.detector,
          { maxFalsePositives: 0, maxFalseNegatives: 0 },
        ]),
      ),
    });

    expect(result.failures).toEqual([]);
    expect(result.metrics.damage?.precision).toBe(1);
    expect(result.metrics.damage?.recall).toBe(1);
    expect(result.syntheticCoverage).toEqual({
      ported: 1,
      pending: 6,
      pendingIds: [
        "fight",
        "damage",
        "super",
        "attack-info",
        "attack-info-attribution",
        "advice",
      ],
    });
  });

  test("uses maximum-cardinality matching for overlapping annotations", () => {
    const result = evaluateRegressionEvents(
      {
        report: {},
        fightMarkers: [],
        regressionEvents: {
          rounds: [],
          damage: [
            { victim: 1, start_frame: 100 },
            { victim: 2, start_frame: 110 },
          ],
          super_arts: [],
          attack_evidence: { sequences: [], damage: [], super_arts: [] },
        },
      },
      {
        semanticEvents: [
          {
            id: "broad",
            detector: "damage",
            frame: { min: 100, max: 110 },
          },
          {
            id: "specific-p1",
            detector: "damage",
            frame: { min: 100, max: 110 },
            side: "p1",
          },
        ],
        detectorGates: {
          damage: { maxFalsePositives: 0, maxFalseNegatives: 0 },
        },
      },
    );

    expect(result.failures).toEqual([]);
    expect(result.metrics.damage?.matched).toBe(2);
  });

  test("fails on a known missed event and an unexpected detection", () => {
    const result = evaluateRegressionEvents(artifacts, {
      semanticEvents: [{ id: "known-miss", detector: "damage", frame: 9999 }],
      detectorGates: {
        damage: { maxFalsePositives: 0, maxFalseNegatives: 0 },
      },
    });

    expect(result.metrics.damage).toMatchObject({
      matched: 0,
      falsePositives: 1,
      falseNegatives: 1,
    });
    expect(result.failures.join("\n")).toContain("known-miss");
    expect(result.failures.join("\n")).toContain("false positive");
  });
});

describe("statistical regression gates", () => {
  test("summarizes median, p90 and stage timings", () => {
    const summary = summarizeTimings([
      { analysisMs: 100, stages: { wasm: 40 } },
      { analysisMs: 80, stages: { wasm: 30 } },
      { analysisMs: 120, stages: { wasm: 50 } },
    ]);
    expect(summary.medianMs).toBe(100);
    expect(summary.p90Ms).toBe(116);
    expect(summary.stages.wasm.medianMs).toBe(40);
  });

  test("fails configured performance and detector regressions", () => {
    const baseline = summarizeTimings([
      { analysisMs: 100, stages: { wasm: 40 } },
      { analysisMs: 100, stages: { wasm: 40 } },
      { analysisMs: 100, stages: { wasm: 40 } },
    ]);
    const current = summarizeTimings([
      { analysisMs: 120, stages: { wasm: 50 } },
      { analysisMs: 121, stages: { wasm: 51 } },
      { analysisMs: 119, stages: { wasm: 49 } },
    ]);
    expect(
      comparePerformance(current, baseline, {
        maxMedianRegressionRatio: 1.1,
        maxP90RegressionRatio: 1.1,
        maxStageMedianRegressionRatio: 1.1,
      }).join("\n"),
    ).toContain("median regressed");

    const metric = {
      expected: 2,
      actual: 2,
      matched: 2,
      falsePositives: 0,
      falseNegatives: 0,
      precision: 1,
      recall: 1,
      meanAbsoluteFrameError: 1,
      maxAbsoluteFrameError: 1,
    };
    expect(
      compareDetectorMetrics(
        { damage: { ...metric, matched: 1, precision: 0.5, recall: 0.5 } },
        { damage: metric },
        { damage: { maxPrecisionDrop: 0, maxRecallDrop: 0 } },
      ).join("\n"),
    ).toContain("precision regressed");
    expect(
      compareDetectorMetrics({}, { damage: metric }, {}).join("\n"),
    ).toContain("metric disappeared");
    expect(
      comparePerformance(
        summarizeTimings([
          { analysisMs: 100, stages: {} },
          { analysisMs: 100, stages: {} },
          { analysisMs: 100, stages: {} },
        ]),
        baseline,
        {},
      ).join("\n"),
    ).toContain("stage wasm is missing");
  });
});

test("semantic snapshot ignores build ids and produces structural paths", () => {
  const baseline = semanticSnapshot({
    report: { analyzer_build_id: "old", rounds_detected: 2 },
    fightMarkers: [{ peak_frame: 100 }],
  });
  const current = semanticSnapshot({
    report: { analyzer_build_id: "new", rounds_detected: 3 },
    fightMarkers: [{ peak_frame: 105 }],
  });
  expect(diffSemanticValues(baseline, current)).toEqual([
    "$.fightMarkers[0].peak_frame: 100 -> 105",
    "$.report.rounds_detected: 2 -> 3",
  ]);
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
