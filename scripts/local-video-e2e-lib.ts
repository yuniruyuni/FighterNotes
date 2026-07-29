export type NumericExpectation =
  | number
  | {
      readonly equals?: number;
      readonly min?: number;
      readonly max?: number;
    };

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
}

export interface LocalVideoCase {
  readonly id: string;
  /** Path visible to the process running this script. */
  readonly videoPath: string;
  /** Optional alternate path visible to a browser running outside this OS. */
  readonly browserVideoPath?: string;
  readonly side: "p1" | "p2";
  readonly ownCharacter: string;
  readonly opponentCharacter: string;
  readonly timeoutSeconds?: number;
  readonly expect?: LocalVideoExpectation;
}

export interface LocalVideoManifest {
  readonly schemaVersion: 1;
  readonly cases: readonly LocalVideoCase[];
}

interface ReportLike {
  readonly rounds_detected?: unknown;
  readonly round_summaries?: unknown;
  readonly cards?: unknown;
  readonly input_stats?: unknown;
  readonly tactic_stats?: unknown;
  readonly coverage?: unknown;
}

export function parseLocalVideoManifest(value: unknown): LocalVideoManifest {
  if (!isRecord(value) || value.schemaVersion !== 1) {
    throw new Error("manifest.schemaVersion must be 1");
  }
  if (!Array.isArray(value.cases) || value.cases.length === 0) {
    throw new Error("manifest.cases must contain at least one case");
  }

  const ids = new Set<string>();
  const cases = value.cases.map((entry, index) => {
    if (!isRecord(entry)) {
      throw new Error(`manifest.cases[${index}] must be an object`);
    }
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
    return {
      id,
      videoPath: requiredString(entry, "videoPath", index),
      ...(typeof entry.browserVideoPath === "string"
        ? { browserVideoPath: entry.browserVideoPath }
        : {}),
      side,
      ownCharacter: requiredString(entry, "ownCharacter", index),
      opponentCharacter: requiredString(entry, "opponentCharacter", index),
      ...(timeoutSeconds === undefined ? {} : { timeoutSeconds }),
      ...(entry.expect === undefined
        ? {}
        : {
            expect: entry.expect as LocalVideoExpectation,
          }),
    } satisfies LocalVideoCase;
  });

  return { schemaVersion: 1, cases };
}

export function evaluateExpectations(
  report: ReportLike,
  expectation: LocalVideoExpectation | undefined,
): string[] {
  if (!expectation) return [];

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
    if (!cardIds.includes(id)) {
      failures.push(`report.cards: expected card "${id}"`);
    }
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

function requiredString(
  entry: Record<string, unknown>,
  field: string,
  index: number,
): string {
  const value = entry[field];
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`manifest.cases[${index}].${field} must be a string`);
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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
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
