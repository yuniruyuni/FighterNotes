import { afterEach, describe, expect, test } from "bun:test";
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  compareArtifactIdentity,
  compareFixtureIdSets,
  computeArtifactIdentity,
  createDecodeMappingIdentity,
  parseBaselineArtifact,
  REQUIRED_PERFORMANCE_STAGES,
  RUNNER_VERSION,
  summarizeSpatialPerformanceStats,
} from "./local-video-e2e-baseline";
import {
  assertBaselineAnalyzedFileBinding,
  beginOutputTransaction,
  prepareOutputDirectories,
} from "./local-video-e2e-output";

const SHA = "a".repeat(64);
const temporaryDirectories: string[] = [];

afterEach(async () => {
  await Promise.all(
    temporaryDirectories
      .splice(0)
      .map((directory) => rm(directory, { recursive: true, force: true })),
  );
});

async function temporaryDirectory(): Promise<string> {
  const directory = await mkdtemp(join(tmpdir(), "fighter-notes-e2e-test-"));
  temporaryDirectories.push(directory);
  return directory;
}

function performance() {
  return {
    runsMs: [100, 100, 100],
    medianMs: 100,
    p90Ms: 100,
    stages: Object.fromEntries(
      REQUIRED_PERFORMANCE_STAGES.map((stage) => [
        stage,
        { medianMs: 10, p90Ms: 10 },
      ]),
    ),
  };
}

function artifact(): Record<string, unknown> {
  return {
    schemaVersion: 2,
    runnerVersion: RUNNER_VERSION,
    caseId: "fixture-a",
    videoName: "replay.mp4",
    fixtureContract: {
      fixtureFingerprint: SHA,
      settings: {
        side: "p1",
        ownCharacter: "KEN",
        opponentCharacter: "JURI",
      },
      expectationHash: SHA,
    },
    analysisMs: 100,
    performance: performance(),
    spatialPerformance: {
      frameCount: 1_001,
      decoderQueueHighWatermark: 12,
      decoderQueueLowWatermark: 6,
      decoderOutstandingHighWatermark: 12,
      decoderOutstandingLowWatermark: 6,
      workerPendingHighWatermark: 12,
      workerPendingLowWatermark: 6,
      peakDecoderQueueSize: 12,
      peakDecoderOutstandingFrames: 12,
      peakWorkerPendingFrames: 12,
    },
    decodeMapping: {
      frameCount: 1,
      sampleCount: 1,
      sha256: SHA,
    },
    streamingPerformance: streamingPerformance(),
    report: { analyzer_build_id: "build-a", rounds_detected: 2 },
    timeline: { left: [], right: [] },
    hpFeatures: [],
    trackedInputs: { p1: [], p2: [] },
    fightMarkers: [],
    attackInfo: [],
    regressionEvents: {
      rounds: [],
      damage: [],
      super_arts: [],
      attack_evidence: { sequences: [], damage: [], super_arts: [] },
    },
    spatialWindows: [],
    spatialObservations: [],
    perfLogs: [],
  };
}

function streamingPerformance() {
  return {
    videoBytes: 1_000_000,
    preflightReadBytes: 1000,
    demuxReadBytes: 900_000,
    demuxReadCount: 10,
    demuxChunkBytes: 1024 * 1024,
    demuxMetadataReadBytes: 2000,
    demuxMediaReadBytes: 898_000,
    metadataSparseRangeCount: 2,
    metadataSparseRangeOperations: 20,
    maxEncodedSampleBytes: 16 * 1024 * 1024,
    observedMaxSampleBytes: 2048,
    maxBatchSamples: 8,
    maxBatchBytes: 16 * 1024 * 1024,
    maxMetadataBytes: 32 * 1024 * 1024,
    maxMetadataMp4BufferBytes: 32 * 1024 * 1024,
    maxMediaMp4BufferBytes: 48 * 1024 * 1024,
    maxMp4SampleBytes: 16 * 1024 * 1024,
    maxDemuxRetainedBytes: 96 * 1024 * 1024,
    peakBlobBytes: 1024 * 1024,
    peakMetadataBlobBytes: 1024 * 1024,
    peakMediaBlobBytes: 2048,
    peakMp4BufferBytes: 1024 * 1024,
    peakMetadataMp4BufferBytes: 1024 * 1024,
    peakMediaMp4BufferBytes: 2048,
    peakMp4SampleBytes: 16_384,
    peakDemuxRetainedBytes: 2 * 1024 * 1024,
    peakBatchSamples: 8,
    peakBatchBytes: 16_384,
    deliveredSamples: 100,
    releasedSamples: 100,
    peakEncodedSamples: 8,
    peakEncodedBytes: 32_768,
    encodedSamplesHighWatermark: 16,
    encodedSamplesLowWatermark: 8,
    encodedBytesHighWatermark: 32 * 1024 * 1024,
    encodedBytesLowWatermark: 16 * 1024 * 1024,
    spatialReadCount: 2,
    spatialReadBytes: 8192,
    peakSpatialBlobBytes: 4096,
    peakSpatialCacheBytes: 4096,
    peakSpatialRetainedBytes: 8192,
    spatialCacheHits: 6,
    spatialCacheMisses: 2,
    firstSampleRunsMs: [10, 10, 10],
    firstSampleMedianMs: 10,
    firstSampleP90Ms: 10,
    demuxFirstSampleRunsMs: [8, 8, 8],
    demuxFirstSampleMedianMs: 8,
    demuxFirstSampleP90Ms: 8,
    processTreePeakRssBytes: 200 * 1024 * 1024,
  };
}

describe("local video E2E baseline identity", () => {
  test("requires the current and baseline fixture sets to be identical", () => {
    expect(compareFixtureIdSets(["a", "b"], ["a", "b"])).toEqual([]);
    expect(compareFixtureIdSets(["a"], ["a", "b"])).toEqual([
      "baseline fixture b is missing from the current manifest",
    ]);
    expect(compareFixtureIdSets(["a", "b"], ["a"])).toEqual([
      "current fixture b is missing from the baseline",
    ]);
    expect(compareFixtureIdSets(["a", "a"], ["a", "a"])).toEqual([
      "current manifest contains duplicate fixture id a",
      "baseline contains duplicate fixture id a",
    ]);
  });

  test("strictly parses a baseline artifact contract", () => {
    expect(parseBaselineArtifact(artifact(), "fixture-a.json", 3).caseId).toBe(
      "fixture-a",
    );
    expect(() =>
      parseBaselineArtifact(
        { ...artifact(), unexpected: true },
        "fixture-a.json",
        3,
      ),
    ).toThrow("fields must be");
    expect(() =>
      parseBaselineArtifact(
        { ...artifact(), regressionEvents: [] },
        "fixture-a.json",
        3,
      ),
    ).toThrow("regressionEvents must be an object");
    expect(() =>
      parseBaselineArtifact(
        { ...artifact(), runnerVersion: 3 },
        "fixture-a.json",
        3,
      ),
    ).toThrow("compatible runner");
    expect(() =>
      parseBaselineArtifact(
        {
          ...artifact(),
          spatialPerformance: {
            ...(artifact().spatialPerformance as object),
            peakWorkerPendingFrames: 13,
          },
        },
        "fixture-a.json",
        3,
      ),
    ).toThrow("exceeds its high watermark");
    const missingStage = artifact();
    const timing = missingStage.performance as ReturnType<typeof performance>;
    const { meterWasm: _missing, ...stages } = timing.stages;
    expect(() =>
      parseBaselineArtifact(
        { ...missingStage, performance: { ...timing, stages } },
        "fixture-a.json",
        3,
      ),
    ).toThrow("stages.meterWasm is required");
  });

  test("accepts runner v4 artifacts for semantic and overall timing comparison", () => {
    const legacy = artifact();
    legacy.runnerVersion = 4;
    delete legacy.decodeMapping;
    delete legacy.streamingPerformance;

    const parsed = parseBaselineArtifact(legacy, "legacy-v4.json", 3);

    expect(parsed.runnerVersion).toBe(4);
    expect(parsed.streamingPerformance).toBeNull();
  });

  test("validates and hashes the exact frame/sample timestamp mapping", () => {
    const mapping = createDecodeMappingIdentity(
      {
        frameTimestamps: [0, 0.001],
        frameToSampleIdx: [0, 1],
        sampleData: [
          { isSync: true, timestampUs: 0, offset: 40, size: 3 },
          { isSync: false, timestampUs: 1000, offset: 43, size: 2 },
        ],
      },
      "mapping",
    );

    expect(mapping).toMatchObject({ frameCount: 2, sampleCount: 2 });
    expect(mapping.sha256).toMatch(/^[0-9a-f]{64}$/);
    expect(() =>
      createDecodeMappingIdentity(
        {
          frameTimestamps: [0],
          frameToSampleIdx: [1],
          sampleData: [{ isSync: true, timestampUs: 0, offset: 40, size: 3 }],
        },
        "invalid-mapping",
      ),
    ).toThrow("out of range");
  });

  test("rejects a stale or replaced full artifact against summary hashes", () => {
    const original = artifact();
    const identity = computeArtifactIdentity(original);
    expect(compareArtifactIdentity(original, identity)).toEqual([]);

    const changed = {
      ...original,
      report: { analyzer_build_id: "build-a", rounds_detected: 3 },
    };
    expect(compareArtifactIdentity(changed, identity)).toEqual([
      "capture hash report does not match summary",
      "semantic hash does not match summary",
    ]);

    const performanceOnly = {
      ...original,
      spatialPerformance: {
        ...(original.spatialPerformance as object),
        peakWorkerPendingFrames: 11,
      },
    };
    expect(compareArtifactIdentity(performanceOnly, identity)).toEqual([]);
  });

  test("keeps stable spatial work and the highest peak across measured runs", () => {
    const first = artifact().spatialPerformance as Parameters<
      typeof summarizeSpatialPerformanceStats
    >[0][number];
    expect(
      summarizeSpatialPerformanceStats(
        [first, { ...first, peakWorkerPendingFrames: 11 }],
        "spatial",
      ).peakWorkerPendingFrames,
    ).toBe(12);
    expect(() =>
      summarizeSpatialPerformanceStats(
        [first, { ...first, frameCount: first.frameCount - 1 }],
        "spatial",
      ),
    ).toThrow("frameCount changed");
  });
});

describe("local video E2E output integrity", () => {
  test("rejects an unverified browser-side path for baseline comparison", () => {
    expect(() =>
      assertBaselineAnalyzedFileBinding("fixture-a", "D:\\replays\\a.mp4"),
    ).toThrow(
      "fixture-a: baseline comparison cannot verify browserVideoPath content against videoPath",
    );
    expect(() =>
      assertBaselineAnalyzedFileBinding("fixture-a", undefined),
    ).not.toThrow();
  });

  test("rejects a symlink alias between baseline and output", async () => {
    const root = await temporaryDirectory();
    const baseline = join(root, "baseline");
    const outputAlias = join(root, "current");
    await mkdir(baseline);
    await symlink(baseline, outputAlias, "dir");

    await expect(
      prepareOutputDirectories(outputAlias, baseline),
    ).rejects.toThrow(
      "--baseline and --output must be different physical directories",
    );
  });

  test("rejects nested baseline and output directories", async () => {
    const root = await temporaryDirectory();
    const output = join(root, "current");
    const nestedBaseline = join(output, "baseline");
    await mkdir(nestedBaseline, { recursive: true });

    await expect(
      prepareOutputDirectories(output, nestedBaseline),
    ).rejects.toThrow(
      "--baseline and --output must not contain one another physically",
    );
  });

  test("keeps the previous output until a complete staged run is published", async () => {
    const root = await temporaryDirectory();
    const output = join(root, "current");
    await prepareOutputDirectories(output);
    await writeFile(join(output, "summary.json"), "old summary");
    await writeFile(join(output, "old-case.json"), "old case");

    const incomplete = await beginOutputTransaction(output);
    await writeFile(join(incomplete.directory, "new-case.json"), "new case");
    await expect(incomplete.publish()).rejects.toThrow(
      "staged E2E output is incomplete",
    );
    expect(await readFile(join(output, "summary.json"), "utf8")).toBe(
      "old summary",
    );
    await incomplete.discard();

    const complete = await beginOutputTransaction(output);
    await writeFile(join(complete.directory, "new-case.json"), "new case");
    await writeFile(join(complete.directory, "summary.json"), "new summary");
    expect(await readFile(join(output, "summary.json"), "utf8")).toBe(
      "old summary",
    );
    await complete.publish();

    expect(await readFile(join(output, "summary.json"), "utf8")).toBe(
      "new summary",
    );
    expect(await readFile(join(output, "new-case.json"), "utf8")).toBe(
      "new case",
    );
    await expect(
      readFile(join(output, "old-case.json"), "utf8"),
    ).rejects.toThrow();
  });
});
