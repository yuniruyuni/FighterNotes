export type NumericExpectation =
  | number
  | {
      readonly equals?: number;
      readonly min?: number;
      readonly max?: number;
    };

export type DetectorId =
  | "round"
  | "fight"
  | "damage"
  | "super"
  | "attackInfo"
  | "attackInfoAttribution"
  | "adviceEvidence";

export type PlayerSide = "p1" | "p2";

export type FrameExpectation =
  | number
  | {
      readonly min: number;
      readonly max: number;
    };

export interface SemanticEventExpectation {
  /** Stable local annotation id used in failure output. */
  readonly id: string;
  readonly detector: DetectorId;
  readonly frame: FrameExpectation;
  readonly side?: PlayerSide;
  readonly roundNo?: NumericExpectation;
  readonly winner?: PlayerSide | null;
  readonly hpDrop?: NumericExpectation;
  readonly reportedDamage?: NumericExpectation;
  readonly attribute?: "upper" | "middle" | "lower" | "throw" | "unknown";
  readonly superLevel?: 1 | 2 | 3;
  readonly criticalArt?: boolean;
  readonly cardId?: string;
  readonly endFrame?: NumericExpectation;
  /** Test path/name once this real-video regression has a synthetic equivalent. */
  readonly syntheticTest?: string;
}

export interface DetectorGate {
  readonly maxFalsePositives?: number;
  readonly maxFalseNegatives?: number;
  readonly minPrecision?: number;
  readonly minRecall?: number;
  readonly maxMeanFrameError?: number;
  readonly maxPrecisionDrop?: number;
  readonly maxRecallDrop?: number;
  readonly maxMeanFrameErrorIncrease?: number;
}

export interface LocalVideoExpectation {
  readonly roundsDetected?: NumericExpectation;
  readonly roundWinners?: readonly (boolean | null)[];
  readonly cardIds?: {
    readonly include?: readonly string[];
    readonly exclude?: readonly string[];
  };
  readonly inputStats?: Readonly<Record<string, NumericExpectation>>;
  readonly tacticStats?: Readonly<Record<string, NumericExpectation>>;
  readonly coverageRatio?: NumericExpectation;
  readonly semanticEvents?: readonly SemanticEventExpectation[];
  readonly detectorGates?: Partial<Readonly<Record<DetectorId, DetectorGate>>>;
}

export interface LocalVideoPerformancePolicy {
  readonly measuredRuns?: number;
  readonly warmupRuns?: number;
  readonly maxMedianRegressionRatio?: number;
  readonly maxP90RegressionRatio?: number;
  readonly maxStageMedianRegressionRatio?: number;
}

export interface LocalVideoCase {
  readonly id: string;
  /** Path visible to the process running this script. */
  readonly videoPath: string;
  /** Optional alternate path visible to a browser running outside this OS. */
  readonly browserVideoPath?: string;
  readonly side: PlayerSide;
  readonly ownCharacter: string;
  readonly opponentCharacter: string;
  readonly timeoutSeconds?: number;
  readonly expect?: LocalVideoExpectation;
}

export interface LocalVideoManifest {
  readonly schemaVersion: 1;
  readonly performance?: LocalVideoPerformancePolicy;
  readonly cases: readonly LocalVideoCase[];
}

export interface RegressionArtifacts {
  readonly report: unknown;
  readonly fightMarkers: unknown;
  readonly regressionEvents: unknown;
}

export interface DetectorMetrics {
  readonly expected: number;
  readonly actual: number;
  readonly matched: number;
  readonly falsePositives: number;
  readonly falseNegatives: number;
  readonly precision: number;
  readonly recall: number;
  readonly meanAbsoluteFrameError: number;
  readonly maxAbsoluteFrameError: number;
}

export interface RegressionEvaluation {
  readonly failures: readonly string[];
  readonly metrics: Partial<Readonly<Record<DetectorId, DetectorMetrics>>>;
  readonly syntheticCoverage: {
    readonly ported: number;
    readonly pending: number;
    readonly pendingIds: readonly string[];
  };
}

export interface TimingSummary {
  readonly runsMs: readonly number[];
  readonly medianMs: number;
  readonly p90Ms: number;
  readonly stages: Readonly<
    Record<
      string,
      {
        readonly medianMs: number;
        readonly p90Ms: number;
      }
    >
  >;
}

interface ActualSemanticEvent {
  readonly detector: DetectorId;
  readonly frame: number;
  readonly side?: PlayerSide;
  readonly roundNo?: number;
  readonly winner?: PlayerSide | null;
  readonly hpDrop?: number;
  readonly reportedDamage?: number;
  readonly attribute?: string;
  readonly superLevel?: number;
  readonly criticalArt?: boolean;
  readonly cardId?: string;
  readonly endFrame?: number;
}

const DETECTORS: readonly DetectorId[] = [
  "round",
  "fight",
  "damage",
  "super",
  "attackInfo",
  "attackInfoAttribution",
  "adviceEvidence",
];

const SEMANTIC_EVENT_FIELDS: Readonly<Record<DetectorId, readonly string[]>> = {
  round: ["roundNo", "winner", "endFrame"],
  fight: ["roundNo"],
  damage: [
    "side",
    "roundNo",
    "hpDrop",
    "reportedDamage",
    "attribute",
    "endFrame",
  ],
  super: [
    "side",
    "roundNo",
    "hpDrop",
    "reportedDamage",
    "superLevel",
    "criticalArt",
    "endFrame",
  ],
  attackInfo: ["side", "roundNo", "reportedDamage", "attribute", "endFrame"],
  attackInfoAttribution: [
    "side",
    "roundNo",
    "reportedDamage",
    "attribute",
    "endFrame",
  ],
  adviceEvidence: ["cardId", "endFrame"],
};

export function parseLocalVideoManifest(value: unknown): LocalVideoManifest {
  if (!isRecord(value) || value.schemaVersion !== 1) {
    throw new Error("manifest.schemaVersion must be 1");
  }
  assertKnownKeys(value, ["schemaVersion", "performance", "cases"], "manifest");
  if (!Array.isArray(value.cases) || value.cases.length === 0) {
    throw new Error("manifest.cases must contain at least one case");
  }

  const ids = new Set<string>();
  const cases = value.cases.map((entry, index) => {
    if (!isRecord(entry)) {
      throw new Error(`manifest.cases[${index}] must be an object`);
    }
    assertKnownKeys(
      entry,
      [
        "id",
        "videoPath",
        "browserVideoPath",
        "side",
        "ownCharacter",
        "opponentCharacter",
        "timeoutSeconds",
        "expect",
      ],
      `manifest.cases[${index}]`,
    );
    const id = requiredString(entry, "id", index);
    if (!/^[a-zA-Z0-9][a-zA-Z0-9._-]*$/.test(id)) {
      throw new Error(
        `manifest.cases[${index}].id must be safe for a file name`,
      );
    }
    if (ids.has(id)) {
      throw new Error(`manifest case id is duplicated: ${id}`);
    }
    ids.add(id);

    const side = entry.side;
    if (side !== "p1" && side !== "p2") {
      throw new Error(`manifest.cases[${index}].side must be p1 or p2`);
    }
    const timeoutSeconds = optionalPositiveNumber(
      entry.timeoutSeconds,
      `manifest.cases[${index}].timeoutSeconds`,
    );
    const expectation = parseExpectation(
      entry.expect,
      `manifest.cases[${index}].expect`,
    );
    return {
      id,
      videoPath: requiredString(entry, "videoPath", index),
      ...(entry.browserVideoPath === undefined
        ? {}
        : {
            browserVideoPath: requiredNonEmptyString(
              entry.browserVideoPath,
              `manifest.cases[${index}].browserVideoPath`,
            ),
          }),
      side,
      ownCharacter: requiredString(entry, "ownCharacter", index),
      opponentCharacter: requiredString(entry, "opponentCharacter", index),
      ...(timeoutSeconds === undefined ? {} : { timeoutSeconds }),
      ...(expectation === undefined ? {} : { expect: expectation }),
    } satisfies LocalVideoCase;
  });

  const performance = parsePerformancePolicy(
    value.performance,
    "manifest.performance",
  );
  return {
    schemaVersion: 1,
    ...(performance ? { performance } : {}),
    cases,
  };
}

export function evaluateExpectations(
  reportValue: unknown,
  expectation: LocalVideoExpectation | undefined,
): string[] {
  if (!expectation) return [];
  const report = isRecord(reportValue) ? reportValue : {};

  const failures: string[] = [];
  if (expectation.roundsDetected !== undefined) {
    checkNumber(
      "report.rounds_detected",
      report.rounds_detected,
      expectation.roundsDetected,
      failures,
    );
  }

  if (expectation.roundWinners) {
    const summaries = Array.isArray(report.round_summaries)
      ? report.round_summaries
      : [];
    const actual = summaries.map((summary) =>
      isRecord(summary) &&
      (typeof summary.won === "boolean" || summary.won === null)
        ? summary.won
        : undefined,
    );
    if (!sameArray(actual, expectation.roundWinners)) {
      failures.push(
        `report.round_summaries[].won: expected ${JSON.stringify(
          expectation.roundWinners,
        )}, got ${JSON.stringify(actual)}`,
      );
    }
  }

  const cardIds = Array.isArray(report.cards)
    ? report.cards.flatMap((card) =>
        isRecord(card) && typeof card.id === "string" ? [card.id] : [],
      )
    : [];
  for (const id of expectation.cardIds?.include ?? []) {
    if (!cardIds.includes(id))
      failures.push(`report.cards: expected card "${id}"`);
  }
  for (const id of expectation.cardIds?.exclude ?? []) {
    if (cardIds.includes(id)) {
      failures.push(`report.cards: did not expect card "${id}"`);
    }
  }

  checkNumericRecord(
    "report.input_stats",
    report.input_stats,
    expectation.inputStats,
    failures,
  );
  checkNumericRecord(
    "report.tactic_stats",
    report.tactic_stats,
    expectation.tacticStats,
    failures,
  );

  if (expectation.coverageRatio !== undefined) {
    const coverage = report.coverage;
    const matchFrames =
      isRecord(coverage) && typeof coverage.match_frames === "number"
        ? coverage.match_frames
        : 0;
    const analyzedFrames =
      isRecord(coverage) && typeof coverage.analyzed_match_frames === "number"
        ? coverage.analyzed_match_frames
        : Number.NaN;
    const ratio = matchFrames > 0 ? analyzedFrames / matchFrames : Number.NaN;
    checkNumber(
      "report.coverage ratio",
      ratio,
      expectation.coverageRatio,
      failures,
    );
  }
  return failures;
}

export function evaluateRegressionEvents(
  artifacts: RegressionArtifacts,
  expectation: LocalVideoExpectation | undefined,
): RegressionEvaluation {
  const expected = expectation?.semanticEvents ?? [];
  const actual = extractSemanticEvents(artifacts);
  const failures: string[] = [];
  const metrics: Partial<Record<DetectorId, DetectorMetrics>> = {};
  const pendingIds = expected
    .filter((event) => !event.syntheticTest)
    .map((event) => event.id);

  for (const detector of DETECTORS) {
    const expectedForDetector = expected.filter(
      (event) => event.detector === detector,
    );
    const gate = expectation?.detectorGates?.[detector];
    if (expectedForDetector.length === 0 && !gate) continue;
    const actualForDetector = actual.filter(
      (event) => event.detector === detector,
    );
    const result = matchEvents(expectedForDetector, actualForDetector);
    metrics[detector] = result.metrics;
    for (const unmatched of result.unmatchedExpected) {
      failures.push(
        `${detector}.${unmatched.id}: expected event was not found in ${frameLabel(
          unmatched.frame,
        )}`,
      );
    }
    applyDetectorGate(detector, result.metrics, gate, failures);
  }

  return {
    failures,
    metrics,
    syntheticCoverage: {
      ported: expected.length - pendingIds.length,
      pending: pendingIds.length,
      pendingIds,
    },
  };
}

export function compareDetectorMetrics(
  current: Partial<Readonly<Record<DetectorId, DetectorMetrics>>>,
  baseline: Partial<Readonly<Record<DetectorId, DetectorMetrics>>>,
  gates: LocalVideoExpectation["detectorGates"],
): string[] {
  const failures: string[] = [];
  for (const detector of DETECTORS) {
    const actual = current[detector];
    const previous = baseline[detector];
    if (previous && !actual) {
      failures.push(
        `${detector}: detector metric disappeared from the current run`,
      );
      continue;
    }
    if (actual && !previous) {
      failures.push(
        `${detector}: detector metric is missing from the baseline`,
      );
      continue;
    }
    if (!actual || !previous) continue;
    const gate = gates?.[detector];
    const maxPrecisionDrop = gate?.maxPrecisionDrop ?? 0;
    const maxRecallDrop = gate?.maxRecallDrop ?? 0;
    const maxFrameIncrease = gate?.maxMeanFrameErrorIncrease ?? 0;
    if (previous.precision - actual.precision > maxPrecisionDrop + 1e-9) {
      failures.push(
        `${detector}: precision regressed from ${formatRatio(
          previous.precision,
        )} to ${formatRatio(actual.precision)}`,
      );
    }
    if (previous.recall - actual.recall > maxRecallDrop + 1e-9) {
      failures.push(
        `${detector}: recall regressed from ${formatRatio(
          previous.recall,
        )} to ${formatRatio(actual.recall)}`,
      );
    }
    if (
      actual.meanAbsoluteFrameError - previous.meanAbsoluteFrameError >
      maxFrameIncrease + 1e-9
    ) {
      failures.push(
        `${detector}: mean frame error regressed from ${previous.meanAbsoluteFrameError.toFixed(
          2,
        )} to ${actual.meanAbsoluteFrameError.toFixed(2)}`,
      );
    }
  }
  return failures;
}

export function summarizeTimings(
  runs: readonly {
    readonly analysisMs: number;
    readonly stages: Readonly<Record<string, number>>;
  }[],
): TimingSummary {
  if (runs.length === 0) throw new Error("at least one timing run is required");
  const runsMs = runs.map((run) => run.analysisMs);
  const stageNames = new Set(runs.flatMap((run) => Object.keys(run.stages)));
  const stages: Record<string, { medianMs: number; p90Ms: number }> = {};
  for (const stage of stageNames) {
    const values = runs.flatMap((run) => {
      const value = run.stages[stage];
      return Number.isFinite(value) ? [value] : [];
    });
    if (values.length === runs.length) {
      stages[stage] = {
        medianMs: median(values),
        p90Ms: percentile(values, 0.9),
      };
    }
  }
  return {
    runsMs,
    medianMs: median(runsMs),
    p90Ms: percentile(runsMs, 0.9),
    stages,
  };
}

export function comparePerformance(
  current: TimingSummary,
  baseline: TimingSummary,
  policy: LocalVideoPerformancePolicy,
): string[] {
  const failures: string[] = [];
  const medianRatio = current.medianMs / baseline.medianMs;
  const p90Ratio = current.p90Ms / baseline.p90Ms;
  const medianLimit = policy.maxMedianRegressionRatio ?? 1.1;
  const p90Limit = policy.maxP90RegressionRatio ?? 1.15;
  const stageLimit = policy.maxStageMedianRegressionRatio ?? 1.15;
  if (medianRatio > medianLimit) {
    failures.push(
      `performance median regressed by ${formatPercent(medianRatio - 1)} (limit ${formatPercent(
        medianLimit - 1,
      )})`,
    );
  }
  if (p90Ratio > p90Limit) {
    failures.push(
      `performance p90 regressed by ${formatPercent(p90Ratio - 1)} (limit ${formatPercent(
        p90Limit - 1,
      )})`,
    );
  }
  const stageNames = new Set([
    ...Object.keys(baseline.stages),
    ...Object.keys(current.stages),
  ]);
  for (const stage of stageNames) {
    const timing = current.stages[stage];
    const previous = baseline.stages[stage];
    if (previous && !timing) {
      failures.push(
        `performance stage ${stage} is missing from the current run`,
      );
      continue;
    }
    if (timing && !previous) {
      failures.push(`performance stage ${stage} is missing from the baseline`);
      continue;
    }
    if (!timing || !previous || previous.medianMs <= 0) continue;
    const ratio = timing.medianMs / previous.medianMs;
    if (ratio > stageLimit) {
      failures.push(
        `performance stage ${stage} median regressed by ${formatPercent(
          ratio - 1,
        )} (limit ${formatPercent(stageLimit - 1)})`,
      );
    }
  }
  return failures;
}

export function compareTimings(
  current: Readonly<Record<string, number>>,
  baseline: Readonly<Record<string, number>>,
): Array<{
  readonly id: string;
  readonly currentMs: number;
  readonly baselineMs: number;
  readonly ratio: number;
}> {
  return Object.entries(current).flatMap(([id, currentMs]) => {
    const baselineMs = baseline[id];
    return Number.isFinite(baselineMs) && baselineMs > 0
      ? [{ id, currentMs, baselineMs, ratio: currentMs / baselineMs }]
      : [];
  });
}

export function semanticSnapshot(artifact: Record<string, unknown>): unknown {
  return {
    report: omitBuildId(artifact.report),
    timeline: artifact.timeline ?? null,
    hpFeatures: artifact.hpFeatures ?? null,
    trackedInputs: artifact.trackedInputs ?? null,
    fightMarkers: artifact.fightMarkers ?? null,
    attackInfo: artifact.attackInfo ?? null,
    regressionEvents: artifact.regressionEvents ?? null,
    spatialWindows: artifact.spatialWindows ?? null,
    spatialObservations: artifact.spatialObservations ?? null,
    ...(artifact.decodeMapping === undefined
      ? {}
      : { decodeMapping: artifact.decodeMapping }),
  };
}

export function diffSemanticValues(
  baseline: unknown,
  current: unknown,
  limit = 50,
): string[] {
  const differences: string[] = [];
  collectDiff(baseline, current, "$", differences, limit);
  return differences;
}

function parseExpectation(
  value: unknown,
  label: string,
): LocalVideoExpectation | undefined {
  if (value === undefined) return undefined;
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertKnownKeys(
    value,
    [
      "roundsDetected",
      "roundWinners",
      "cardIds",
      "inputStats",
      "tacticStats",
      "coverageRatio",
      "semanticEvents",
      "detectorGates",
    ],
    label,
  );
  const semanticEvents = parseSemanticEvents(value.semanticEvents, label);
  const detectorGates = parseDetectorGates(value.detectorGates, label);
  const cardIds = parseCardIds(value.cardIds, label);
  const inputStats = parseOptionalNumericRecord(
    value.inputStats,
    `${label}.inputStats`,
  );
  const tacticStats = parseOptionalNumericRecord(
    value.tacticStats,
    `${label}.tacticStats`,
  );
  return {
    ...(value.roundsDetected === undefined
      ? {}
      : {
          roundsDetected: parseNumericExpectation(
            value.roundsDetected,
            `${label}.roundsDetected`,
          ),
        }),
    ...parseRoundWinners(value.roundWinners, label),
    ...(cardIds ? { cardIds } : {}),
    ...(inputStats ? { inputStats } : {}),
    ...(tacticStats ? { tacticStats } : {}),
    ...(value.coverageRatio === undefined
      ? {}
      : {
          coverageRatio: parseNumericExpectation(
            value.coverageRatio,
            `${label}.coverageRatio`,
          ),
        }),
    ...(semanticEvents ? { semanticEvents } : {}),
    ...(detectorGates ? { detectorGates } : {}),
  };
}

function parseSemanticEvents(
  value: unknown,
  label: string,
): SemanticEventExpectation[] | undefined {
  if (value === undefined) return undefined;
  if (!Array.isArray(value))
    throw new Error(`${label}.semanticEvents must be an array`);
  const ids = new Set<string>();
  return value.map((event, index) => {
    const eventLabel = `${label}.semanticEvents[${index}]`;
    if (!isRecord(event)) throw new Error(`${eventLabel} must be an object`);
    const id = requiredNonEmptyString(event.id, `${eventLabel}.id`);
    if (ids.has(id)) throw new Error(`${eventLabel}.id is duplicated: ${id}`);
    ids.add(id);
    if (!isDetector(event.detector)) {
      throw new Error(`${eventLabel}.detector is not supported`);
    }
    assertKnownKeys(
      event,
      [
        "id",
        "detector",
        "frame",
        "syntheticTest",
        ...SEMANTIC_EVENT_FIELDS[event.detector],
      ],
      eventLabel,
    );
    const frame = parseFrameExpectation(event.frame, `${eventLabel}.frame`);
    const side = event.side;
    if (side !== undefined && side !== "p1" && side !== "p2") {
      throw new Error(`${eventLabel}.side must be p1 or p2`);
    }
    const winner = event.winner;
    if (
      winner !== undefined &&
      winner !== null &&
      winner !== "p1" &&
      winner !== "p2"
    ) {
      throw new Error(`${eventLabel}.winner must be p1, p2, or null`);
    }
    const superLevel = event.superLevel;
    if (
      superLevel !== undefined &&
      superLevel !== 1 &&
      superLevel !== 2 &&
      superLevel !== 3
    ) {
      throw new Error(`${eventLabel}.superLevel must be 1, 2, or 3`);
    }
    const criticalArt = event.criticalArt;
    if (criticalArt !== undefined && typeof criticalArt !== "boolean") {
      throw new Error(`${eventLabel}.criticalArt must be boolean`);
    }
    const cardId =
      event.cardId === undefined
        ? undefined
        : requiredNonEmptyString(event.cardId, `${eventLabel}.cardId`);
    const syntheticTest =
      event.syntheticTest === undefined
        ? undefined
        : requiredNonEmptyString(
            event.syntheticTest,
            `${eventLabel}.syntheticTest`,
          );
    return {
      id,
      detector: event.detector,
      frame,
      ...(side ? { side } : {}),
      ...(event.roundNo === undefined
        ? {}
        : {
            roundNo: parseNumericExpectation(
              event.roundNo,
              `${eventLabel}.roundNo`,
            ),
          }),
      ...(winner === undefined ? {} : { winner }),
      ...(event.hpDrop === undefined
        ? {}
        : {
            hpDrop: parseNumericExpectation(
              event.hpDrop,
              `${eventLabel}.hpDrop`,
            ),
          }),
      ...(event.reportedDamage === undefined
        ? {}
        : {
            reportedDamage: parseNumericExpectation(
              event.reportedDamage,
              `${eventLabel}.reportedDamage`,
            ),
          }),
      ...parseAttribute(event.attribute, eventLabel),
      ...(superLevel === undefined ? {} : { superLevel }),
      ...(criticalArt === undefined ? {} : { criticalArt }),
      ...(cardId === undefined ? {} : { cardId }),
      ...(event.endFrame === undefined
        ? {}
        : {
            endFrame: parseNumericExpectation(
              event.endFrame,
              `${eventLabel}.endFrame`,
            ),
          }),
      ...(syntheticTest === undefined ? {} : { syntheticTest }),
    } satisfies SemanticEventExpectation;
  });
}

function parseDetectorGates(
  value: unknown,
  label: string,
): Partial<Record<DetectorId, DetectorGate>> | undefined {
  if (value === undefined) return undefined;
  if (!isRecord(value))
    throw new Error(`${label}.detectorGates must be an object`);
  const gates: Partial<Record<DetectorId, DetectorGate>> = {};
  for (const [detector, rawGate] of Object.entries(value)) {
    if (!isDetector(detector))
      throw new Error(`${label}.detectorGates.${detector} is not supported`);
    if (!isRecord(rawGate))
      throw new Error(`${label}.detectorGates.${detector} must be an object`);
    assertKnownKeys(
      rawGate,
      [
        "maxFalsePositives",
        "maxFalseNegatives",
        "minPrecision",
        "minRecall",
        "maxMeanFrameError",
        "maxPrecisionDrop",
        "maxRecallDrop",
        "maxMeanFrameErrorIncrease",
      ],
      `${label}.detectorGates.${detector}`,
    );
    const gate: Record<string, number> = {};
    for (const field of ["maxFalsePositives", "maxFalseNegatives"] as const) {
      const raw = rawGate[field];
      if (raw === undefined) continue;
      if (typeof raw !== "number" || !Number.isInteger(raw) || raw < 0) {
        throw new Error(
          `${label}.detectorGates.${detector}.${field} must be a non-negative integer`,
        );
      }
      gate[field] = raw;
    }
    for (const field of [
      "minPrecision",
      "minRecall",
      "maxPrecisionDrop",
      "maxRecallDrop",
    ] as const) {
      const raw = rawGate[field];
      if (raw === undefined) continue;
      if (
        typeof raw !== "number" ||
        !Number.isFinite(raw) ||
        raw < 0 ||
        raw > 1
      ) {
        throw new Error(
          `${label}.detectorGates.${detector}.${field} must be between 0 and 1`,
        );
      }
      gate[field] = raw;
    }
    for (const field of [
      "maxMeanFrameError",
      "maxMeanFrameErrorIncrease",
    ] as const) {
      const raw = rawGate[field];
      if (raw === undefined) continue;
      if (typeof raw !== "number" || !Number.isFinite(raw) || raw < 0) {
        throw new Error(
          `${label}.detectorGates.${detector}.${field} must be non-negative`,
        );
      }
      gate[field] = raw;
    }
    if (Object.keys(gate).length === 0) {
      throw new Error(
        `${label}.detectorGates.${detector} must contain at least one threshold`,
      );
    }
    gates[detector] = gate;
  }
  return gates;
}

function parsePerformancePolicy(
  value: unknown,
  label: string,
): LocalVideoPerformancePolicy | undefined {
  if (value === undefined) return undefined;
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertKnownKeys(
    value,
    [
      "measuredRuns",
      "warmupRuns",
      "maxMedianRegressionRatio",
      "maxP90RegressionRatio",
      "maxStageMedianRegressionRatio",
    ],
    label,
  );
  const measuredRuns = optionalInteger(
    value.measuredRuns,
    `${label}.measuredRuns`,
    1,
  );
  const warmupRuns = optionalInteger(
    value.warmupRuns,
    `${label}.warmupRuns`,
    0,
  );
  return {
    ...(measuredRuns === undefined ? {} : { measuredRuns }),
    ...(warmupRuns === undefined ? {} : { warmupRuns }),
    ...positivePolicyRatios(value, label),
  };
}

function positivePolicyRatios(
  value: Record<string, unknown>,
  label: string,
): Pick<
  LocalVideoPerformancePolicy,
  | "maxMedianRegressionRatio"
  | "maxP90RegressionRatio"
  | "maxStageMedianRegressionRatio"
> {
  const result: Record<string, number> = {};
  for (const field of [
    "maxMedianRegressionRatio",
    "maxP90RegressionRatio",
    "maxStageMedianRegressionRatio",
  ] as const) {
    const raw = value[field];
    if (raw === undefined) continue;
    if (typeof raw !== "number" || !Number.isFinite(raw) || raw <= 0) {
      throw new Error(`${label}.${field} must be a positive number`);
    }
    result[field] = raw;
  }
  return result;
}

function extractSemanticEvents(
  artifacts: RegressionArtifacts,
): ActualSemanticEvent[] {
  const actual: ActualSemanticEvent[] = [];
  const regression = isRecord(artifacts.regressionEvents)
    ? artifacts.regressionEvents
    : {};
  const rounds = arrayOfRecords(regression.rounds);
  for (const round of rounds) {
    const frame = finiteNumber(round.start_frame);
    if (frame === undefined) continue;
    actual.push({
      detector: "round",
      frame,
      ...optionalNumberField("roundNo", round.round_no),
      winner: playerSide(round.winner) ?? null,
      ...optionalNumberField("endFrame", round.end_frame),
    });
  }

  const fightMarkers = Array.isArray(artifacts.fightMarkers)
    ? artifacts.fightMarkers
    : [];
  for (const [index, marker] of fightMarkers.entries()) {
    if (!isRecord(marker)) continue;
    const frame = finiteNumber(marker.peak_frame);
    if (frame === undefined) continue;
    actual.push({ detector: "fight", frame, roundNo: index + 1 });
  }

  const attackEvidence = isRecord(regression.attack_evidence)
    ? regression.attack_evidence
    : {};
  const attackSequences = arrayOfRecords(attackEvidence.sequences);
  const damageEvidence = arrayOfRecords(attackEvidence.damage);
  const damage = arrayOfRecords(regression.damage);
  for (const event of damage) {
    const frame = finiteNumber(event.start_frame);
    if (frame === undefined) continue;
    const victim = finiteNumber(event.victim);
    const linked = damageEvidence.find(
      (candidate) =>
        finiteNumber(candidate.victim) === victim &&
        finiteNumber(candidate.damage_start_frame) === frame,
    );
    actual.push({
      detector: "damage",
      frame,
      ...(playerSide(victim) ? { side: playerSide(victim) } : {}),
      ...optionalNumberField("roundNo", event.round_no),
      ...optionalNumberField("hpDrop", event.drop),
      ...optionalNumberField("endFrame", event.end_frame),
      ...optionalNumberField("reportedDamage", linked?.combo_damage),
      ...(typeof linked?.starter_attribute === "string"
        ? { attribute: linked.starter_attribute }
        : {}),
    });
  }

  const superEvidence = arrayOfRecords(attackEvidence.super_arts);
  for (const event of arrayOfRecords(regression.super_arts)) {
    const frame = finiteNumber(event.frame);
    if (frame === undefined) continue;
    const side = finiteNumber(event.side);
    const linked = superEvidence.find(
      (candidate) =>
        finiteNumber(candidate.side) === side &&
        finiteNumber(candidate.super_frame) === frame,
    );
    actual.push({
      detector: "super",
      frame,
      ...(playerSide(side) ? { side: playerSide(side) } : {}),
      ...optionalNumberField("roundNo", event.round_no),
      ...optionalNumberField("superLevel", event.level),
      ...(typeof event.critical_art === "boolean"
        ? { criticalArt: event.critical_art }
        : {}),
      ...optionalNumberField("hpDrop", event.damage),
      ...optionalNumberField("reportedDamage", linked?.combo_damage),
    });
  }

  for (const sequence of attackSequences) {
    const frame = finiteNumber(sequence.start_frame);
    if (frame === undefined) continue;
    const round = rounds.find((candidate) => {
      const start = finiteNumber(candidate.start_frame);
      const end = finiteNumber(candidate.end_frame);
      return (
        start !== undefined &&
        end !== undefined &&
        frame >= start &&
        frame <= end
      );
    });
    actual.push({
      detector: "attackInfo",
      frame,
      ...(playerSide(sequence.attacker)
        ? { side: playerSide(sequence.attacker) }
        : {}),
      ...optionalNumberField("roundNo", round?.round_no),
      ...optionalNumberField("reportedDamage", sequence.combo_damage),
      ...(typeof sequence.starter_attribute === "string"
        ? { attribute: sequence.starter_attribute }
        : {}),
      ...optionalNumberField("endFrame", sequence.end_frame),
    });
  }

  for (const evidence of damageEvidence) {
    const frame = finiteNumber(evidence.sequence_start_frame);
    if (frame === undefined) continue;
    const damageFrame = finiteNumber(evidence.damage_start_frame);
    const damageEvent = damage.find(
      (candidate) => finiteNumber(candidate.start_frame) === damageFrame,
    );
    actual.push({
      detector: "attackInfoAttribution",
      frame,
      ...(playerSide(evidence.attacker)
        ? { side: playerSide(evidence.attacker) }
        : {}),
      ...optionalNumberField("roundNo", damageEvent?.round_no),
      ...optionalNumberField("reportedDamage", evidence.combo_damage),
      ...(typeof evidence.starter_attribute === "string"
        ? { attribute: evidence.starter_attribute }
        : {}),
      ...optionalNumberField("endFrame", evidence.sequence_end_frame),
    });
  }

  const report = isRecord(artifacts.report) ? artifacts.report : {};
  for (const card of arrayOfRecords(report.cards)) {
    if (typeof card.id !== "string") continue;
    for (const evidence of arrayOfRecords(card.evidence)) {
      const frame = finiteNumber(evidence.frame);
      if (frame === undefined) continue;
      actual.push({
        detector: "adviceEvidence",
        frame,
        cardId: card.id,
        ...optionalNumberField("endFrame", evidence.end_frame),
      });
    }
  }
  return actual;
}

function matchEvents(
  expected: readonly SemanticEventExpectation[],
  actual: readonly ActualSemanticEvent[],
): {
  readonly metrics: DetectorMetrics;
  readonly unmatchedExpected: readonly SemanticEventExpectation[];
} {
  const matching = minimumCostMaximumMatching(expected, actual);
  const usedExpected = new Set(matching.map((match) => match.expectedIndex));
  const frameErrors = matching.map((match) => match.frameError);
  const matched = usedExpected.size;
  const falsePositives = actual.length - matched;
  const falseNegatives = expected.length - matched;
  return {
    metrics: {
      expected: expected.length,
      actual: actual.length,
      matched,
      falsePositives,
      falseNegatives,
      precision:
        actual.length === 0
          ? expected.length === 0
            ? 1
            : 0
          : matched / actual.length,
      recall: expected.length === 0 ? 1 : matched / expected.length,
      meanAbsoluteFrameError:
        frameErrors.length === 0
          ? 0
          : frameErrors.reduce((sum, value) => sum + value, 0) /
            frameErrors.length,
      maxAbsoluteFrameError:
        frameErrors.length === 0 ? 0 : Math.max(...frameErrors),
    },
    unmatchedExpected: expected.filter((_, index) => !usedExpected.has(index)),
  };
}

interface FlowEdge {
  readonly to: number;
  readonly reverse: number;
  capacity: number;
  readonly cost: number;
}

function minimumCostMaximumMatching(
  expected: readonly SemanticEventExpectation[],
  actual: readonly ActualSemanticEvent[],
): readonly {
  readonly expectedIndex: number;
  readonly actualIndex: number;
  readonly frameError: number;
}[] {
  const source = 0;
  const expectedOffset = 1;
  const actualOffset = expectedOffset + expected.length;
  const sink = actualOffset + actual.length;
  const graph: FlowEdge[][] = Array.from({ length: sink + 1 }, () => []);
  const candidates: Array<{
    readonly edge: FlowEdge;
    readonly expectedIndex: number;
    readonly actualIndex: number;
    readonly frameError: number;
  }> = [];

  for (
    let expectedIndex = 0;
    expectedIndex < expected.length;
    expectedIndex += 1
  ) {
    addFlowEdge(graph, source, expectedOffset + expectedIndex, 0);
  }
  for (let actualIndex = 0; actualIndex < actual.length; actualIndex += 1) {
    addFlowEdge(graph, actualOffset + actualIndex, sink, 0);
  }
  for (
    let expectedIndex = 0;
    expectedIndex < expected.length;
    expectedIndex += 1
  ) {
    const annotation = expected[expectedIndex];
    for (let actualIndex = 0; actualIndex < actual.length; actualIndex += 1) {
      const event = actual[actualIndex];
      if (!matchesAnnotation(annotation, event)) continue;
      const frameError = Math.abs(
        event.frame - expectedFrameCenter(annotation.frame),
      );
      const edge = addFlowEdge(
        graph,
        expectedOffset + expectedIndex,
        actualOffset + actualIndex,
        frameError,
      );
      candidates.push({ edge, expectedIndex, actualIndex, frameError });
    }
  }

  while (augmentShortestPath(graph, source, sink)) {
    // Each path adds one match. Continue until no augmenting path remains so
    // cardinality is maximal; shortest residual paths minimize total error.
  }

  return candidates
    .filter((candidate) => candidate.edge.capacity === 0)
    .map(({ expectedIndex, actualIndex, frameError }) => ({
      expectedIndex,
      actualIndex,
      frameError,
    }));
}

function addFlowEdge(
  graph: FlowEdge[][],
  from: number,
  to: number,
  cost: number,
): FlowEdge {
  const forward: FlowEdge = {
    to,
    reverse: graph[to].length,
    capacity: 1,
    cost,
  };
  const reverse: FlowEdge = {
    to: from,
    reverse: graph[from].length,
    capacity: 0,
    cost: -cost,
  };
  graph[from].push(forward);
  graph[to].push(reverse);
  return forward;
}

function augmentShortestPath(
  graph: FlowEdge[][],
  source: number,
  sink: number,
): boolean {
  const distances = Array.from(
    { length: graph.length },
    () => Number.POSITIVE_INFINITY,
  );
  const previousNode = Array.from({ length: graph.length }, () => -1);
  const previousEdge = Array.from({ length: graph.length }, () => -1);
  distances[source] = 0;

  for (let pass = 0; pass < graph.length - 1; pass += 1) {
    let changed = false;
    for (let node = 0; node < graph.length; node += 1) {
      if (!Number.isFinite(distances[node])) continue;
      for (let edgeIndex = 0; edgeIndex < graph[node].length; edgeIndex += 1) {
        const edge = graph[node][edgeIndex];
        if (edge.capacity === 0) continue;
        const candidate = distances[node] + edge.cost;
        if (candidate + 1e-9 >= distances[edge.to]) continue;
        distances[edge.to] = candidate;
        previousNode[edge.to] = node;
        previousEdge[edge.to] = edgeIndex;
        changed = true;
      }
    }
    if (!changed) break;
  }
  if (!Number.isFinite(distances[sink])) return false;

  for (let node = sink; node !== source; node = previousNode[node]) {
    const from = previousNode[node];
    const edgeIndex = previousEdge[node];
    if (from < 0 || edgeIndex < 0) return false;
    const edge = graph[from][edgeIndex];
    edge.capacity -= 1;
    graph[node][edge.reverse].capacity += 1;
  }
  return true;
}

function matchesAnnotation(
  expected: SemanticEventExpectation,
  actual: ActualSemanticEvent,
): boolean {
  if (!frameContains(expected.frame, actual.frame)) return false;
  if (expected.side !== undefined && expected.side !== actual.side)
    return false;
  if (expected.winner !== undefined && expected.winner !== actual.winner)
    return false;
  if (
    expected.attribute !== undefined &&
    expected.attribute !== actual.attribute
  )
    return false;
  if (
    expected.superLevel !== undefined &&
    expected.superLevel !== actual.superLevel
  )
    return false;
  if (
    expected.criticalArt !== undefined &&
    expected.criticalArt !== actual.criticalArt
  )
    return false;
  if (expected.cardId !== undefined && expected.cardId !== actual.cardId)
    return false;
  return (
    numberMatches(actual.roundNo, expected.roundNo) &&
    numberMatches(actual.hpDrop, expected.hpDrop) &&
    numberMatches(actual.reportedDamage, expected.reportedDamage) &&
    numberMatches(actual.endFrame, expected.endFrame)
  );
}

function applyDetectorGate(
  detector: DetectorId,
  metrics: DetectorMetrics,
  gate: DetectorGate | undefined,
  failures: string[],
): void {
  if (!gate) return;
  if (
    gate.maxFalsePositives !== undefined &&
    metrics.falsePositives > gate.maxFalsePositives
  ) {
    failures.push(
      `${detector}: ${metrics.falsePositives} false positive(s), maximum ${gate.maxFalsePositives}`,
    );
  }
  if (
    gate.maxFalseNegatives !== undefined &&
    metrics.falseNegatives > gate.maxFalseNegatives
  ) {
    failures.push(
      `${detector}: ${metrics.falseNegatives} false negative(s), maximum ${gate.maxFalseNegatives}`,
    );
  }
  if (
    gate.minPrecision !== undefined &&
    metrics.precision < gate.minPrecision
  ) {
    failures.push(
      `${detector}: precision ${formatRatio(metrics.precision)} is below ${formatRatio(
        gate.minPrecision,
      )}`,
    );
  }
  if (gate.minRecall !== undefined && metrics.recall < gate.minRecall) {
    failures.push(
      `${detector}: recall ${formatRatio(metrics.recall)} is below ${formatRatio(
        gate.minRecall,
      )}`,
    );
  }
  if (
    gate.maxMeanFrameError !== undefined &&
    metrics.meanAbsoluteFrameError > gate.maxMeanFrameError
  ) {
    failures.push(
      `${detector}: mean frame error ${metrics.meanAbsoluteFrameError.toFixed(
        2,
      )} exceeds ${gate.maxMeanFrameError}`,
    );
  }
}

function parseNumericRecord(
  value: Record<string, unknown>,
  label: string,
): Record<string, NumericExpectation> {
  return Object.fromEntries(
    Object.entries(value).map(([key, item]) => [
      key,
      parseNumericExpectation(item, `${label}.${key}`),
    ]),
  );
}

function parseOptionalNumericRecord(
  value: unknown,
  label: string,
): Record<string, NumericExpectation> | undefined {
  if (value === undefined) return undefined;
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  return parseNumericRecord(value, label);
}

function parseCardIds(
  value: unknown,
  label: string,
): LocalVideoExpectation["cardIds"] | undefined {
  if (value === undefined) return undefined;
  if (!isRecord(value)) throw new Error(`${label}.cardIds must be an object`);
  assertKnownKeys(value, ["include", "exclude"], `${label}.cardIds`);
  return {
    include: stringArray(value.include, `${label}.cardIds.include`),
    exclude: stringArray(value.exclude, `${label}.cardIds.exclude`),
  };
}

function parseRoundWinners(
  value: unknown,
  label: string,
): { readonly roundWinners?: readonly (boolean | null)[] } {
  if (value === undefined) return {};
  if (
    !Array.isArray(value) ||
    value.some((winner) => typeof winner !== "boolean" && winner !== null)
  ) {
    throw new Error(`${label}.roundWinners must contain booleans or null`);
  }
  return { roundWinners: value };
}

function parseAttribute(
  value: unknown,
  label: string,
): { readonly attribute?: SemanticEventExpectation["attribute"] } {
  if (value === undefined) return {};
  if (
    value !== "upper" &&
    value !== "middle" &&
    value !== "lower" &&
    value !== "throw" &&
    value !== "unknown"
  ) {
    throw new Error(`${label}.attribute is not supported`);
  }
  return { attribute: value };
}

function parseNumericExpectation(
  value: unknown,
  label: string,
): NumericExpectation {
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (!isRecord(value)) throw new Error(`${label} must be a number or range`);
  assertKnownKeys(value, ["equals", "min", "max"], label);
  const result: { equals?: number; min?: number; max?: number } = {};
  for (const field of ["equals", "min", "max"] as const) {
    const raw = value[field];
    if (raw === undefined) continue;
    if (typeof raw !== "number" || !Number.isFinite(raw)) {
      throw new Error(`${label}.${field} must be a finite number`);
    }
    result[field] = raw;
  }
  if (Object.keys(result).length === 0) {
    throw new Error(`${label} must contain equals, min, or max`);
  }
  if (
    result.min !== undefined &&
    result.max !== undefined &&
    result.min > result.max
  ) {
    throw new Error(`${label} must have min <= max`);
  }
  if (
    result.equals !== undefined &&
    ((result.min !== undefined && result.equals < result.min) ||
      (result.max !== undefined && result.equals > result.max))
  ) {
    throw new Error(`${label}.equals must be inside min/max`);
  }
  return result;
}

function parseFrameExpectation(
  value: unknown,
  label: string,
): FrameExpectation {
  if (typeof value === "number" && Number.isInteger(value) && value >= 0)
    return value;
  if (!isRecord(value)) throw new Error(`${label} must be a frame or range`);
  assertKnownKeys(value, ["min", "max"], label);
  const min = value.min;
  const max = value.max;
  if (
    typeof min !== "number" ||
    typeof max !== "number" ||
    !Number.isInteger(min) ||
    !Number.isInteger(max) ||
    min < 0 ||
    max < min
  ) {
    throw new Error(`${label} must have integer 0 <= min <= max`);
  }
  return { min, max };
}

function checkNumericRecord(
  label: string,
  value: unknown,
  expected: Readonly<Record<string, NumericExpectation>> | undefined,
  failures: string[],
): void {
  if (!expected) return;
  for (const [key, expectation] of Object.entries(expected)) {
    checkNumber(
      `${label}.${key}`,
      isRecord(value) ? value[key] : undefined,
      expectation,
      failures,
    );
  }
}

function checkNumber(
  label: string,
  actual: unknown,
  expectation: NumericExpectation,
  failures: string[],
): void {
  if (typeof actual !== "number" || !Number.isFinite(actual)) {
    failures.push(`${label}: expected a finite number, got ${String(actual)}`);
    return;
  }
  const normalized =
    typeof expectation === "number" ? { equals: expectation } : expectation;
  if (normalized.equals !== undefined && actual !== normalized.equals) {
    failures.push(`${label}: expected ${normalized.equals}, got ${actual}`);
  }
  if (normalized.min !== undefined && actual < normalized.min) {
    failures.push(`${label}: expected >= ${normalized.min}, got ${actual}`);
  }
  if (normalized.max !== undefined && actual > normalized.max) {
    failures.push(`${label}: expected <= ${normalized.max}, got ${actual}`);
  }
}

function numberMatches(
  actual: number | undefined,
  expectation: NumericExpectation | undefined,
): boolean {
  if (expectation === undefined) return true;
  if (actual === undefined || !Number.isFinite(actual)) return false;
  const normalized =
    typeof expectation === "number" ? { equals: expectation } : expectation;
  return !(
    (normalized.equals !== undefined && actual !== normalized.equals) ||
    (normalized.min !== undefined && actual < normalized.min) ||
    (normalized.max !== undefined && actual > normalized.max)
  );
}

function requiredString(
  entry: Record<string, unknown>,
  field: string,
  index: number,
): string {
  return requiredNonEmptyString(
    entry[field],
    `manifest.cases[${index}].${field}`,
  );
}

function requiredNonEmptyString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} must be a string`);
  }
  return value;
}

function optionalPositiveNumber(
  value: unknown,
  label: string,
): number | undefined {
  if (value === undefined) return undefined;
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    throw new Error(`${label} must be a positive number`);
  }
  return value;
}

function optionalInteger(
  value: unknown,
  label: string,
  minimum: number,
): number | undefined {
  if (value === undefined) return undefined;
  if (
    typeof value !== "number" ||
    !Number.isInteger(value) ||
    value < minimum
  ) {
    throw new Error(`${label} must be an integer >= ${minimum}`);
  }
  return value;
}

function stringArray(value: unknown, label: string): string[] {
  if (value === undefined) return [];
  if (
    !Array.isArray(value) ||
    value.some((item) => typeof item !== "string" || item.trim() === "")
  ) {
    throw new Error(`${label} must be a string array`);
  }
  return value;
}

function assertKnownKeys(
  value: Readonly<Record<string, unknown>>,
  allowed: readonly string[],
  label: string,
): void {
  const known = new Set(allowed);
  const unexpected = Object.keys(value).filter((key) => !known.has(key));
  if (unexpected.length > 0) {
    throw new Error(
      `${label} contains unknown field(s): ${unexpected.join(", ")}`,
    );
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isDetector(value: unknown): value is DetectorId {
  return typeof value === "string" && DETECTORS.includes(value as DetectorId);
}

function sameArray(
  actual: readonly unknown[],
  expected: readonly unknown[],
): boolean {
  return (
    actual.length === expected.length &&
    actual.every((value, index) => value === expected[index])
  );
}

function frameContains(expectation: FrameExpectation, actual: number): boolean {
  return typeof expectation === "number"
    ? expectation === actual
    : actual >= expectation.min && actual <= expectation.max;
}

function expectedFrameCenter(expectation: FrameExpectation): number {
  return typeof expectation === "number"
    ? expectation
    : (expectation.min + expectation.max) / 2;
}

function frameLabel(expectation: FrameExpectation): string {
  return typeof expectation === "number"
    ? `frame ${expectation}`
    : `frames ${expectation.min}..${expectation.max}`;
}

function finiteNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value)
    ? value
    : undefined;
}

function playerSide(value: unknown): PlayerSide | undefined {
  return value === 1 ? "p1" : value === 2 ? "p2" : undefined;
}

function optionalNumberField<Key extends string>(
  key: Key,
  value: unknown,
): Partial<Record<Key, number>> {
  const number = finiteNumber(value);
  return number === undefined ? {} : ({ [key]: number } as Record<Key, number>);
}

function arrayOfRecords(value: unknown): Record<string, unknown>[] {
  return Array.isArray(value) ? value.filter(isRecord) : [];
}

function median(values: readonly number[]): number {
  return percentile(values, 0.5);
}

function percentile(values: readonly number[], quantile: number): number {
  const sorted = [...values].sort((left, right) => left - right);
  if (sorted.length === 1) return sorted[0];
  const index = (sorted.length - 1) * quantile;
  const lower = Math.floor(index);
  const upper = Math.ceil(index);
  const fraction = index - lower;
  return sorted[lower] * (1 - fraction) + sorted[upper] * fraction;
}

function formatRatio(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

function formatPercent(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}

function omitBuildId(value: unknown): unknown {
  if (!isRecord(value)) return value;
  const { analyzer_build_id: _ignored, ...report } = value;
  return report;
}

function collectDiff(
  baseline: unknown,
  current: unknown,
  path: string,
  differences: string[],
  limit: number,
): void {
  if (
    differences.length >= limit ||
    Object.is(baseline, current) ||
    (baseline === 0 && current === 0)
  )
    return;
  if (Array.isArray(baseline) && Array.isArray(current)) {
    if (baseline.length !== current.length) {
      differences.push(
        `${path}.length: ${baseline.length} -> ${current.length}`,
      );
    }
    const length = Math.min(baseline.length, current.length);
    for (
      let index = 0;
      index < length && differences.length < limit;
      index += 1
    ) {
      collectDiff(
        baseline[index],
        current[index],
        `${path}[${index}]`,
        differences,
        limit,
      );
    }
    return;
  }
  if (isRecord(baseline) && isRecord(current)) {
    const keys = [
      ...new Set([...Object.keys(baseline), ...Object.keys(current)]),
    ].sort();
    for (const key of keys) {
      if (differences.length >= limit) break;
      if (!(key in baseline)) {
        differences.push(`${path}.${key}: added ${shortValue(current[key])}`);
      } else if (!(key in current)) {
        differences.push(
          `${path}.${key}: removed ${shortValue(baseline[key])}`,
        );
      } else {
        collectDiff(
          baseline[key],
          current[key],
          `${path}.${key}`,
          differences,
          limit,
        );
      }
    }
    return;
  }
  differences.push(
    `${path}: ${shortValue(baseline)} -> ${shortValue(current)}`,
  );
}

function shortValue(value: unknown): string {
  const serialized = JSON.stringify(value);
  if (serialized === undefined) return String(value);
  return serialized.length > 120
    ? `${serialized.slice(0, 117)}...`
    : serialized;
}
