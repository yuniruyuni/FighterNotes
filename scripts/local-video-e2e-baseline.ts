import {
  semanticSnapshot,
  summarizeTimings,
  type TimingSummary,
} from "./local-video-e2e-lib";
import {
  parseStreamingPerformanceStats,
  type StreamingPerformanceStats,
} from "./local-video-e2e-streaming";

export const RUNNER_VERSION = 5;
export const LEGACY_RUNNER_VERSION = 4;
export const REQUIRED_PERFORMANCE_STAGES = [
  "firstPass",
  "spatialPass",
  "frameExtraction",
  "workerCopy",
  "meterWasm",
  "hudWasm",
] as const;
export const CAPTURE_HASH_FIELDS = [
  "report",
  "timeline",
  "features",
  "trackedInputs",
  "fightMarkers",
  "attackInfo",
  "regressionEvents",
  "spatialWindows",
  "spatialObservations",
] as const;
export type CaptureHashField = (typeof CAPTURE_HASH_FIELDS)[number];

const CAPTURE_ARTIFACT_FIELDS: Readonly<Record<CaptureHashField, string>> = {
  report: "report",
  timeline: "timeline",
  features: "hpFeatures",
  trackedInputs: "trackedInputs",
  fightMarkers: "fightMarkers",
  attackInfo: "attackInfo",
  regressionEvents: "regressionEvents",
  spatialWindows: "spatialWindows",
  spatialObservations: "spatialObservations",
};

export interface FixtureSettings {
  readonly side: "p1" | "p2";
  readonly ownCharacter: string;
  readonly opponentCharacter: string;
}

export interface ArtifactIdentity {
  readonly hashes: Readonly<Record<CaptureHashField, string>>;
  readonly semanticHash: string;
}

export interface BaselineCaseArtifact extends Record<string, unknown> {
  readonly schemaVersion: 2;
  readonly runnerVersion: number;
  readonly caseId: string;
  readonly videoName: string;
  readonly fixtureContract: {
    readonly fixtureFingerprint: string;
    readonly settings: FixtureSettings;
    readonly expectationHash: string;
  };
  readonly analysisMs: number;
  readonly performance: TimingSummary;
  readonly spatialPerformance: SpatialPerformanceStats;
  readonly decodeMapping?: DecodeMappingIdentity;
  /** Absent from runner v4 artifacts; semantic and overall timing remain usable. */
  readonly streamingPerformance: StreamingPerformanceStats | null;
}

export interface DecodeMappingIdentity {
  readonly frameCount: number;
  readonly sampleCount: number;
  readonly sha256: string;
}

export interface SpatialPerformanceStats {
  readonly frameCount: number;
  readonly decoderQueueHighWatermark: number;
  readonly decoderQueueLowWatermark: number;
  readonly decoderOutstandingHighWatermark: number;
  readonly decoderOutstandingLowWatermark: number;
  readonly workerPendingHighWatermark: number;
  readonly workerPendingLowWatermark: number;
  readonly peakDecoderQueueSize: number;
  readonly peakDecoderOutstandingFrames: number;
  readonly peakWorkerPendingFrames: number;
}

export function compareFixtureIdSets(
  currentIds: readonly string[],
  baselineIds: readonly string[],
): string[] {
  const failures: string[] = [];
  const current = new Set(currentIds);
  const baseline = new Set(baselineIds);
  for (const id of duplicateValues(currentIds)) {
    failures.push(`current manifest contains duplicate fixture id ${id}`);
  }
  for (const id of duplicateValues(baselineIds)) {
    failures.push(`baseline contains duplicate fixture id ${id}`);
  }
  for (const id of [...baseline].sort()) {
    if (!current.has(id)) {
      failures.push(
        `baseline fixture ${id} is missing from the current manifest`,
      );
    }
  }
  for (const id of [...current].sort()) {
    if (!baseline.has(id)) {
      failures.push(`current fixture ${id} is missing from the baseline`);
    }
  }
  return failures;
}

export function computeArtifactIdentity(
  artifact: Readonly<Record<string, unknown>>,
): ArtifactIdentity {
  const hashes = Object.fromEntries(
    CAPTURE_HASH_FIELDS.map((field) => [
      field,
      sha256(serializedJson(artifact[CAPTURE_ARTIFACT_FIELDS[field]] ?? null)),
    ]),
  ) as Record<CaptureHashField, string>;
  return {
    hashes,
    semanticHash: sha256(serializedJson(semanticSnapshot({ ...artifact }))),
  };
}

export function compareArtifactIdentity(
  artifact: Readonly<Record<string, unknown>>,
  expected: ArtifactIdentity,
): string[] {
  const actual = computeArtifactIdentity(artifact);
  const failures = CAPTURE_HASH_FIELDS.flatMap((field) =>
    actual.hashes[field] === expected.hashes[field]
      ? []
      : [`capture hash ${field} does not match summary`],
  );
  if (actual.semanticHash !== expected.semanticHash) {
    failures.push("semantic hash does not match summary");
  }
  return failures;
}

export function createDecodeMappingIdentity(
  value: unknown,
  label: string,
): DecodeMappingIdentity {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(
    value,
    ["frameTimestamps", "frameToSampleIdx", "sampleData"],
    label,
  );
  if (
    !Array.isArray(value.frameTimestamps) ||
    !Array.isArray(value.frameToSampleIdx) ||
    !Array.isArray(value.sampleData)
  ) {
    throw new Error(`${label} arrays are required`);
  }
  if (value.frameTimestamps.length !== value.frameToSampleIdx.length) {
    throw new Error(`${label} frame timestamp and sample mapping counts differ`);
  }
  const frameTimestamps = value.frameTimestamps.map((timestamp, index) =>
    requiredUnboundedFiniteNumber(
      timestamp,
      `${label}.frameTimestamps[${index}]`,
    ),
  );
  const sampleData = value.sampleData.map((sample, index) =>
    parseMappingSample(sample, `${label}.sampleData[${index}]`),
  );
  const frameToSampleIdx = value.frameToSampleIdx.map((sampleIndex, index) => {
    const parsed = requiredInteger(
      sampleIndex,
      `${label}.frameToSampleIdx[${index}]`,
      -1,
    );
    if (parsed >= sampleData.length) {
      throw new Error(`${label}.frameToSampleIdx[${index}] is out of range`);
    }
    if (
      parsed >= 0 &&
      Math.round(frameTimestamps[index] * 1_000_000) !==
        sampleData[parsed].timestampUs
    ) {
      throw new Error(`${label}.frameToSampleIdx[${index}] timestamp differs`);
    }
    return parsed;
  });
  return {
    frameCount: frameTimestamps.length,
    sampleCount: sampleData.length,
    sha256: sha256(
      canonicalJson({ frameTimestamps, frameToSampleIdx, sampleData }),
    ),
  };
}

export function parseBaselineArtifact(
  value: unknown,
  label: string,
  measuredRuns: number,
): BaselineCaseArtifact {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  const runnerVersion = value.runnerVersion;
  if (
    value.schemaVersion !== 2 ||
    (runnerVersion !== RUNNER_VERSION &&
      runnerVersion !== LEGACY_RUNNER_VERSION)
  ) {
    throw new Error(`${label} was not generated by a compatible runner`);
  }
  assertExactKeys(
    value,
    [
      "schemaVersion",
      "runnerVersion",
      "caseId",
      "videoName",
      "fixtureContract",
      "analysisMs",
      "performance",
      "spatialPerformance",
      ...(runnerVersion === RUNNER_VERSION ? ["decodeMapping"] : []),
      ...(runnerVersion === RUNNER_VERSION ? ["streamingPerformance"] : []),
      "report",
      "timeline",
      "hpFeatures",
      "trackedInputs",
      "fightMarkers",
      "attackInfo",
      "regressionEvents",
      "spatialWindows",
      "spatialObservations",
      "perfLogs",
    ],
    label,
  );
  if (!isRecord(value.fixtureContract)) {
    throw new Error(`${label}.fixtureContract must be an object`);
  }
  assertExactKeys(
    value.fixtureContract,
    ["fixtureFingerprint", "settings", "expectationHash"],
    `${label}.fixtureContract`,
  );
  const performance = parseTimingSummary(
    value.performance,
    `${label}.performance`,
    measuredRuns,
  );
  const analysisMs = requiredFiniteNumber(
    value.analysisMs,
    `${label}.analysisMs`,
    0,
  );
  if (Math.abs(analysisMs - performance.medianMs) > 1e-6) {
    throw new Error(`${label}.analysisMs must equal performance.medianMs`);
  }
  const spatialPerformance = parseSpatialPerformanceStats(
    value.spatialPerformance,
    `${label}.spatialPerformance`,
  );
  const streamingPerformance =
    runnerVersion === RUNNER_VERSION
      ? parseStreamingPerformanceStats(
          value.streamingPerformance,
          `${label}.streamingPerformance`,
          measuredRuns,
        )
      : null;
  const decodeMapping =
    runnerVersion === RUNNER_VERSION
      ? parseDecodeMappingIdentity(
          value.decodeMapping,
          `${label}.decodeMapping`,
        )
      : undefined;
  requireRecordPayload(value.report, `${label}.report`);
  requireRecordPayload(value.timeline, `${label}.timeline`);
  requireArrayPayload(value.hpFeatures, `${label}.hpFeatures`);
  requireNullableRecordPayload(value.trackedInputs, `${label}.trackedInputs`);
  requireNullableArrayPayload(value.fightMarkers, `${label}.fightMarkers`);
  requireNullableArrayPayload(value.attackInfo, `${label}.attackInfo`);
  requireRecordPayload(value.regressionEvents, `${label}.regressionEvents`);
  requireNullableArrayPayload(value.spatialWindows, `${label}.spatialWindows`);
  requireNullableArrayPayload(
    value.spatialObservations,
    `${label}.spatialObservations`,
  );
  if (
    !Array.isArray(value.perfLogs) ||
    value.perfLogs.some((line) => typeof line !== "string")
  ) {
    throw new Error(`${label}.perfLogs must be a string array`);
  }
  return {
    ...value,
    schemaVersion: 2,
    runnerVersion,
    caseId: requiredString(value.caseId, `${label}.caseId`),
    videoName: requiredString(value.videoName, `${label}.videoName`),
    fixtureContract: {
      fixtureFingerprint: requiredSha256(
        value.fixtureContract.fixtureFingerprint,
        `${label}.fixtureContract.fixtureFingerprint`,
      ),
      settings: parseFixtureSettings(
        value.fixtureContract.settings,
        `${label}.fixtureContract.settings`,
      ),
      expectationHash: requiredSha256(
        value.fixtureContract.expectationHash,
        `${label}.fixtureContract.expectationHash`,
      ),
    },
    analysisMs,
    performance,
    spatialPerformance,
    ...(decodeMapping ? { decodeMapping } : {}),
    streamingPerformance,
  };
}

function parseDecodeMappingIdentity(
  value: unknown,
  label: string,
): DecodeMappingIdentity {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(value, ["frameCount", "sampleCount", "sha256"], label);
  return {
    frameCount: requiredInteger(value.frameCount, `${label}.frameCount`, 0),
    sampleCount: requiredInteger(value.sampleCount, `${label}.sampleCount`, 0),
    sha256: requiredSha256(value.sha256, `${label}.sha256`),
  };
}

function parseMappingSample(
  value: unknown,
  label: string,
): {
  readonly isSync: boolean;
  readonly timestampUs: number;
  readonly offset: number;
  readonly size: number;
} {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(value, ["isSync", "timestampUs", "offset", "size"], label);
  if (typeof value.isSync !== "boolean") {
    throw new Error(`${label}.isSync must be boolean`);
  }
  return {
    isSync: value.isSync,
    timestampUs: requiredSafeInteger(
      value.timestampUs,
      `${label}.timestampUs`,
    ),
    offset: requiredSafeInteger(value.offset, `${label}.offset`, 0),
    size: requiredSafeInteger(value.size, `${label}.size`, 1),
  };
}

export function parseSpatialPerformanceStats(
  value: unknown,
  label: string,
): SpatialPerformanceStats {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  const fields = [
    "frameCount",
    "decoderQueueHighWatermark",
    "decoderQueueLowWatermark",
    "decoderOutstandingHighWatermark",
    "decoderOutstandingLowWatermark",
    "workerPendingHighWatermark",
    "workerPendingLowWatermark",
    "peakDecoderQueueSize",
    "peakDecoderOutstandingFrames",
    "peakWorkerPendingFrames",
  ] as const;
  assertExactKeys(value, fields, label);
  const parsed = Object.fromEntries(
    fields.map((field) => [
      field,
      requiredInteger(value[field], `${label}.${field}`, 0),
    ]),
  ) as unknown as SpatialPerformanceStats;
  validatePeak(
    parsed.peakDecoderQueueSize,
    parsed.decoderQueueHighWatermark,
    `${label}.peakDecoderQueueSize`,
  );
  validatePeak(
    parsed.peakDecoderOutstandingFrames,
    parsed.decoderOutstandingHighWatermark,
    `${label}.peakDecoderOutstandingFrames`,
  );
  validatePeak(
    parsed.peakWorkerPendingFrames,
    parsed.workerPendingHighWatermark,
    `${label}.peakWorkerPendingFrames`,
  );
  validateWatermarkPair(
    parsed.decoderQueueHighWatermark,
    parsed.decoderQueueLowWatermark,
    `${label}.decoderQueue`,
  );
  validateWatermarkPair(
    parsed.decoderOutstandingHighWatermark,
    parsed.decoderOutstandingLowWatermark,
    `${label}.decoderOutstanding`,
  );
  validateWatermarkPair(
    parsed.workerPendingHighWatermark,
    parsed.workerPendingLowWatermark,
    `${label}.workerPending`,
  );
  return parsed;
}

export function summarizeSpatialPerformanceStats(
  runs: readonly SpatialPerformanceStats[],
  label: string,
): SpatialPerformanceStats {
  const first = runs[0];
  if (!first) throw new Error(`${label} requires at least one run`);
  const stableFields = [
    "frameCount",
    "decoderQueueHighWatermark",
    "decoderQueueLowWatermark",
    "decoderOutstandingHighWatermark",
    "decoderOutstandingLowWatermark",
    "workerPendingHighWatermark",
    "workerPendingLowWatermark",
  ] as const;
  for (const [index, run] of runs.entries()) {
    for (const field of stableFields) {
      if (run[field] !== first[field]) {
        throw new Error(
          `${label}[${index}].${field} changed from ${first[field]} to ${run[field]}`,
        );
      }
    }
  }
  return {
    ...first,
    peakDecoderQueueSize: Math.max(
      ...runs.map((run) => run.peakDecoderQueueSize),
    ),
    peakDecoderOutstandingFrames: Math.max(
      ...runs.map((run) => run.peakDecoderOutstandingFrames),
    ),
    peakWorkerPendingFrames: Math.max(
      ...runs.map((run) => run.peakWorkerPendingFrames),
    ),
  };
}

export function canonicalJson(value: unknown): string {
  if (Array.isArray(value)) {
    return `[${value.map(canonicalJson).join(",")}]`;
  }
  if (isRecord(value)) {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${canonicalJson(value[key])}`)
      .join(",")}}`;
  }
  const serialized = JSON.stringify(value);
  if (serialized === undefined)
    throw new Error("cannot canonicalize undefined");
  return serialized;
}

function serializedJson(value: unknown): string {
  const serialized = JSON.stringify(value);
  if (serialized === undefined) throw new Error("cannot serialize undefined");
  return serialized;
}

function parseTimingSummary(
  value: unknown,
  label: string,
  measuredRuns: number,
): TimingSummary {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(value, ["runsMs", "medianMs", "p90Ms", "stages"], label);
  if (
    !Array.isArray(value.runsMs) ||
    value.runsMs.length !== measuredRuns ||
    value.runsMs.some((run) => !isFiniteNumberAtLeast(run, 0))
  ) {
    throw new Error(
      `${label}.runsMs must contain ${measuredRuns} finite values`,
    );
  }
  const runsMs = value.runsMs as number[];
  const derived = summarizeTimings(
    runsMs.map((analysisMs) => ({ analysisMs, stages: {} })),
  );
  const medianMs = requiredFiniteNumber(value.medianMs, `${label}.medianMs`, 0);
  const p90Ms = requiredFiniteNumber(value.p90Ms, `${label}.p90Ms`, 0);
  if (
    Math.abs(medianMs - derived.medianMs) > 1e-6 ||
    Math.abs(p90Ms - derived.p90Ms) > 1e-6
  ) {
    throw new Error(`${label} median/p90 must agree with runsMs`);
  }
  if (!isRecord(value.stages)) {
    throw new Error(`${label}.stages must be an object`);
  }
  const stages: Record<string, { medianMs: number; p90Ms: number }> = {};
  for (const [stage, timing] of Object.entries(value.stages)) {
    if (!isRecord(timing)) {
      throw new Error(`${label}.stages.${stage} must be an object`);
    }
    assertExactKeys(timing, ["medianMs", "p90Ms"], `${label}.stages.${stage}`);
    stages[stage] = {
      medianMs: requiredFiniteNumber(
        timing.medianMs,
        `${label}.stages.${stage}.medianMs`,
        0,
      ),
      p90Ms: requiredFiniteNumber(
        timing.p90Ms,
        `${label}.stages.${stage}.p90Ms`,
        0,
      ),
    };
  }
  for (const stage of REQUIRED_PERFORMANCE_STAGES) {
    if (!stages[stage]) throw new Error(`${label}.stages.${stage} is required`);
  }
  return { runsMs, medianMs, p90Ms, stages };
}

function parseFixtureSettings(value: unknown, label: string): FixtureSettings {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  assertExactKeys(value, ["side", "ownCharacter", "opponentCharacter"], label);
  if (value.side !== "p1" && value.side !== "p2") {
    throw new Error(`${label}.side must be p1 or p2`);
  }
  return {
    side: value.side,
    ownCharacter: requiredString(value.ownCharacter, `${label}.ownCharacter`),
    opponentCharacter: requiredString(
      value.opponentCharacter,
      `${label}.opponentCharacter`,
    ),
  };
}

function duplicateValues(values: readonly string[]): string[] {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const value of values) {
    if (seen.has(value)) duplicates.add(value);
    else seen.add(value);
  }
  return [...duplicates].sort();
}

function sha256(value: string): string {
  return new Bun.CryptoHasher("sha256").update(value).digest("hex");
}

function assertExactKeys(
  value: Readonly<Record<string, unknown>>,
  expected: readonly string[],
  label: string,
): void {
  const actual = Object.keys(value).sort();
  const required = [...expected].sort();
  if (
    actual.length !== required.length ||
    actual.some((key, index) => key !== required[index])
  ) {
    throw new Error(
      `${label} fields must be ${required.join(", ")}; got ${actual.join(", ")}`,
    );
  }
}

function requiredString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.trim() === "") {
    throw new Error(`${label} must be a non-empty string`);
  }
  return value;
}

function requiredSha256(value: unknown, label: string): string {
  const hash = requiredString(value, label);
  if (!/^[0-9a-f]{64}$/.test(hash)) {
    throw new Error(`${label} must be a lowercase SHA-256 digest`);
  }
  return hash;
}

function isFiniteNumberAtLeast(
  value: unknown,
  minimum: number,
): value is number {
  return (
    typeof value === "number" && Number.isFinite(value) && value >= minimum
  );
}

function requiredFiniteNumber(
  value: unknown,
  label: string,
  minimum: number,
): number {
  if (!isFiniteNumberAtLeast(value, minimum)) {
    throw new Error(`${label} must be a finite number >= ${minimum}`);
  }
  return value;
}

function requiredUnboundedFiniteNumber(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value)) {
    throw new Error(`${label} must be finite`);
  }
  return value;
}

function requiredInteger(
  value: unknown,
  label: string,
  minimum: number,
): number {
  const parsed = requiredFiniteNumber(value, label, minimum);
  if (!Number.isInteger(parsed)) {
    throw new Error(`${label} must be an integer`);
  }
  return parsed;
}

function requiredSafeInteger(
  value: unknown,
  label: string,
  minimum = Number.MIN_SAFE_INTEGER,
): number {
  if (
    typeof value !== "number" ||
    !Number.isSafeInteger(value) ||
    value < minimum
  ) {
    throw new Error(`${label} must be a safe integer >= ${minimum}`);
  }
  return value;
}

function validatePeak(peak: number, high: number, label: string): void {
  if (peak > high) {
    throw new Error(`${label} exceeds its high watermark ${high}`);
  }
}

function validateWatermarkPair(high: number, low: number, label: string): void {
  if (high <= 0 || low >= high) {
    throw new Error(`${label} watermarks must satisfy 0 <= low < high`);
  }
}

function requireRecordPayload(value: unknown, label: string): void {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
}

function requireArrayPayload(value: unknown, label: string): void {
  if (!Array.isArray(value)) throw new Error(`${label} must be an array`);
}

function requireNullableRecordPayload(value: unknown, label: string): void {
  if (value !== null) requireRecordPayload(value, label);
}

function requireNullableArrayPayload(value: unknown, label: string): void {
  if (value !== null) requireArrayPayload(value, label);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
