import {
  DEMUX_METADATA_CHUNK_BYTES,
  ENCODED_QUEUE_BYTE_LOW_WATERMARK,
  ENCODED_QUEUE_SAMPLE_LOW_WATERMARK,
  MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES,
  MAX_DEMUX_METADATA_BYTES,
  MAX_DEMUX_METADATA_MP4_BUFFER_BYTES,
  MAX_DEMUX_MP4_SAMPLE_BYTES,
  MAX_DEMUX_RETAINED_BYTES,
  MAX_ENCODED_BATCH_BYTES,
  MAX_ENCODED_BATCH_SAMPLES,
  MAX_ENCODED_QUEUE_BYTES,
  MAX_ENCODED_QUEUE_SAMPLES,
  MAX_ENCODED_SAMPLE_BYTES,
} from "../client/src/modules/analysis/domain/encoded-video-limits";
import { summarizeTimings } from "./local-video-e2e-lib";

const STABLE_FIELDS = [
  "videoBytes",
  "preflightReadBytes",
  "demuxReadBytes",
  "demuxReadCount",
  "demuxChunkBytes",
  "demuxMetadataReadBytes",
  "demuxMediaReadBytes",
  "metadataSparseRangeCount",
  "metadataSparseRangeOperations",
  "maxEncodedSampleBytes",
  "observedMaxSampleBytes",
  "maxBatchSamples",
  "maxBatchBytes",
  "maxMetadataBytes",
  "maxMetadataMp4BufferBytes",
  "maxMediaMp4BufferBytes",
  "maxMp4SampleBytes",
  "maxDemuxRetainedBytes",
  "deliveredSamples",
  "releasedSamples",
  "encodedSamplesHighWatermark",
  "encodedSamplesLowWatermark",
  "encodedBytesHighWatermark",
  "encodedBytesLowWatermark",
] as const;

const DEMUX_PEAK_FIELDS = [
  "peakBlobBytes",
  "peakMetadataBlobBytes",
  "peakMediaBlobBytes",
  "peakMp4BufferBytes",
  "peakMetadataMp4BufferBytes",
  "peakMediaMp4BufferBytes",
  "peakMp4SampleBytes",
  "peakDemuxRetainedBytes",
  "peakBatchSamples",
  "peakBatchBytes",
] as const;

const VARIABLE_MAX_FIELDS = [
  "spatialReadCount",
  "spatialReadBytes",
  "spatialCacheHits",
  "spatialCacheMisses",
  "peakEncodedSamples",
  "peakEncodedBytes",
  "peakSpatialBlobBytes",
  "peakSpatialCacheBytes",
  "peakSpatialRetainedBytes",
] as const;

const SUMMARIZED_MAX_FIELDS = [
  ...DEMUX_PEAK_FIELDS,
  ...VARIABLE_MAX_FIELDS,
] as const;

export interface StreamingRunStats {
  readonly firstSampleMs: number;
  readonly demuxFirstSampleMs: number;
  readonly processTreePeakRssBytes: number | null;
  readonly videoBytes: number;
  readonly preflightReadBytes: number;
  readonly demuxReadBytes: number;
  readonly demuxReadCount: number;
  readonly demuxChunkBytes: number;
  readonly demuxMetadataReadBytes: number;
  readonly demuxMediaReadBytes: number;
  readonly metadataSparseRangeCount: number;
  readonly metadataSparseRangeOperations: number;
  readonly maxEncodedSampleBytes: number;
  readonly observedMaxSampleBytes: number;
  readonly maxBatchSamples: number;
  readonly maxBatchBytes: number;
  readonly maxMetadataBytes: number;
  readonly maxMetadataMp4BufferBytes: number;
  readonly maxMediaMp4BufferBytes: number;
  readonly maxMp4SampleBytes: number;
  readonly maxDemuxRetainedBytes: number;
  readonly peakBlobBytes: number;
  readonly peakMetadataBlobBytes: number;
  readonly peakMediaBlobBytes: number;
  readonly peakMp4BufferBytes: number;
  readonly peakMetadataMp4BufferBytes: number;
  readonly peakMediaMp4BufferBytes: number;
  readonly peakMp4SampleBytes: number;
  readonly peakDemuxRetainedBytes: number;
  readonly peakBatchSamples: number;
  readonly peakBatchBytes: number;
  readonly deliveredSamples: number;
  readonly releasedSamples: number;
  readonly peakEncodedSamples: number;
  readonly peakEncodedBytes: number;
  readonly encodedSamplesHighWatermark: number;
  readonly encodedSamplesLowWatermark: number;
  readonly encodedBytesHighWatermark: number;
  readonly encodedBytesLowWatermark: number;
  readonly spatialReadCount: number;
  readonly spatialReadBytes: number;
  readonly peakSpatialBlobBytes: number;
  readonly peakSpatialCacheBytes: number;
  readonly peakSpatialRetainedBytes: number;
  readonly spatialCacheHits: number;
  readonly spatialCacheMisses: number;
}

export interface StreamingPerformanceStats
  extends Omit<
    StreamingRunStats,
    "firstSampleMs" | "demuxFirstSampleMs" | "processTreePeakRssBytes"
  > {
  readonly firstSampleRunsMs: readonly number[];
  readonly firstSampleMedianMs: number;
  readonly firstSampleP90Ms: number;
  readonly demuxFirstSampleRunsMs: readonly number[];
  readonly demuxFirstSampleMedianMs: number;
  readonly demuxFirstSampleP90Ms: number;
  readonly processTreePeakRssBytes: number | null;
}

const STREAMING_LOG_FIELDS = {
  videoBytes: "video",
  preflightReadBytes: "preflight_read",
  demuxReadBytes: "demux_read",
  demuxReadCount: "demux_reads",
  demuxChunkBytes: "demux_chunk",
  demuxMetadataReadBytes: "demux_metadata_read",
  demuxMediaReadBytes: "demux_media_read",
  metadataSparseRangeCount: "metadata_ranges",
  metadataSparseRangeOperations: "metadata_range_ops",
  maxEncodedSampleBytes: "sample_max",
  observedMaxSampleBytes: "observed_sample_max",
  maxBatchSamples: "batch_samples_max",
  maxBatchBytes: "batch_bytes_max",
  maxMetadataBytes: "metadata_bytes_max",
  maxMetadataMp4BufferBytes: "metadata_mp4_buffer_max",
  maxMediaMp4BufferBytes: "media_mp4_buffer_max",
  maxMp4SampleBytes: "mp4_sample_bytes_max",
  maxDemuxRetainedBytes: "demux_retained_max",
  peakBlobBytes: "peak_blob",
  peakMetadataBlobBytes: "peak_metadata_blob",
  peakMediaBlobBytes: "peak_media_blob",
  peakMp4BufferBytes: "peak_mp4_buffer",
  peakMetadataMp4BufferBytes: "peak_metadata_mp4_buffer",
  peakMediaMp4BufferBytes: "peak_media_mp4_buffer",
  peakMp4SampleBytes: "peak_mp4_sample",
  peakDemuxRetainedBytes: "peak_demux_retained",
  peakBatchSamples: "peak_batch_samples",
  peakBatchBytes: "peak_batch",
  deliveredSamples: "delivered_samples",
  releasedSamples: "released_samples",
  peakEncodedSamples: "peak_encoded_samples",
  peakEncodedBytes: "peak_encoded_queue",
  encodedSamplesHighWatermark: "encoded_samples_high",
  encodedSamplesLowWatermark: "encoded_samples_low",
  encodedBytesHighWatermark: "encoded_bytes_high",
  encodedBytesLowWatermark: "encoded_bytes_low",
  demuxFirstSampleMs: "demux_first_sample",
} as const;

const SPATIAL_LOG_FIELDS = {
  spatialReadCount: "reads",
  spatialReadBytes: "read",
  peakSpatialBlobBytes: "peak_blob",
  peakSpatialCacheBytes: "peak_cache",
  peakSpatialRetainedBytes: "peak_spatial_retained",
  spatialCacheHits: "cache_hits",
  spatialCacheMisses: "cache_misses",
} as const;

export function parseStreamingRunStats(
  perfLogs: readonly string[],
  label: string,
  firstSampleMs: number,
  processTreePeakRssBytes: number | null,
): StreamingRunStats {
  const streaming = lastLog(perfLogs, "[perf] streaming", label);
  const spatial = optionalLastLog(perfLogs, "[perf] spatial-blob");
  const result = {
    ...parseMappedFields(streaming, STREAMING_LOG_FIELDS, label),
    ...(spatial
      ? parseMappedFields(spatial, SPATIAL_LOG_FIELDS, label)
      : zeroMappedFields(SPATIAL_LOG_FIELDS)),
    firstSampleMs: requiredFiniteNumber(
      firstSampleMs,
      `${label}.firstSampleMs`,
    ),
    processTreePeakRssBytes,
  } as StreamingRunStats;
  assertStreamingBounds(result, label);
  return result;
}

export function summarizeStreamingPerformanceStats(
  runs: readonly StreamingRunStats[],
  label: string,
): StreamingPerformanceStats {
  if (runs.length === 0) throw new Error(`${label} must not be empty`);
  const stable = Object.fromEntries(
    STABLE_FIELDS.map((field) => [field, stableValue(runs, field, label)]),
  );
  const peaks = Object.fromEntries(
    SUMMARIZED_MAX_FIELDS.map((field) => [
      field,
      Math.max(...runs.map((run) => run[field])),
    ]),
  );
  const startup = summarizeTimings(
    runs.map((run) => ({ analysisMs: run.firstSampleMs, stages: {} })),
  );
  const demuxStartup = summarizeTimings(
    runs.map((run) => ({ analysisMs: run.demuxFirstSampleMs, stages: {} })),
  );
  const rssSupported = runs[0].processTreePeakRssBytes !== null;
  if (
    runs.some((run) => (run.processTreePeakRssBytes !== null) !== rssSupported)
  ) {
    throw new Error(
      `${label}.processTreePeakRssBytes support changed between runs`,
    );
  }
  const rss = runs.flatMap((run) =>
    run.processTreePeakRssBytes === null ? [] : [run.processTreePeakRssBytes],
  );
  const result = {
    ...stable,
    ...peaks,
    firstSampleRunsMs: startup.runsMs,
    firstSampleMedianMs: startup.medianMs,
    firstSampleP90Ms: startup.p90Ms,
    demuxFirstSampleRunsMs: demuxStartup.runsMs,
    demuxFirstSampleMedianMs: demuxStartup.medianMs,
    demuxFirstSampleP90Ms: demuxStartup.p90Ms,
    processTreePeakRssBytes: rss.length > 0 ? Math.max(...rss) : null,
  } as StreamingPerformanceStats;
  assertStreamingBounds(
    {
      ...result,
      firstSampleMs: result.firstSampleMedianMs,
      demuxFirstSampleMs: result.demuxFirstSampleMedianMs,
    },
    label,
  );
  return result;
}

export function parseStreamingPerformanceStats(
  value: unknown,
  label: string,
  measuredRuns: number,
): StreamingPerformanceStats {
  if (!isRecord(value)) throw new Error(`${label} must be an object`);
  const fields = [
    ...STABLE_FIELDS,
    ...SUMMARIZED_MAX_FIELDS,
    "firstSampleRunsMs",
    "firstSampleMedianMs",
    "firstSampleP90Ms",
    "demuxFirstSampleRunsMs",
    "demuxFirstSampleMedianMs",
    "demuxFirstSampleP90Ms",
    "processTreePeakRssBytes",
  ] as const;
  assertExactKeys(value, fields, label);
  const numeric = Object.fromEntries(
    [...STABLE_FIELDS, ...SUMMARIZED_MAX_FIELDS].map((field) => [
      field,
      requiredNonNegativeInteger(value[field], `${label}.${field}`),
    ]),
  );
  if (
    !Array.isArray(value.firstSampleRunsMs) ||
    value.firstSampleRunsMs.length !== measuredRuns
  ) {
    throw new Error(
      `${label}.firstSampleRunsMs must contain ${measuredRuns} runs`,
    );
  }
  const firstSampleRunsMs = value.firstSampleRunsMs.map((run, index) =>
    requiredFiniteNumber(run, `${label}.firstSampleRunsMs[${index}]`),
  );
  const expected = summarizeTimings(
    firstSampleRunsMs.map((analysisMs) => ({ analysisMs, stages: {} })),
  );
  const firstSampleMedianMs = requiredFiniteNumber(
    value.firstSampleMedianMs,
    `${label}.firstSampleMedianMs`,
  );
  const firstSampleP90Ms = requiredFiniteNumber(
    value.firstSampleP90Ms,
    `${label}.firstSampleP90Ms`,
  );
  if (
    firstSampleMedianMs !== expected.medianMs ||
    firstSampleP90Ms !== expected.p90Ms
  ) {
    throw new Error(`${label} first-sample summary does not match its runs`);
  }
  if (
    !Array.isArray(value.demuxFirstSampleRunsMs) ||
    value.demuxFirstSampleRunsMs.length !== measuredRuns
  ) {
    throw new Error(
      `${label}.demuxFirstSampleRunsMs must contain ${measuredRuns} runs`,
    );
  }
  const demuxFirstSampleRunsMs = value.demuxFirstSampleRunsMs.map(
    (run, index) =>
      requiredFiniteNumber(run, `${label}.demuxFirstSampleRunsMs[${index}]`),
  );
  const expectedDemux = summarizeTimings(
    demuxFirstSampleRunsMs.map((analysisMs) => ({ analysisMs, stages: {} })),
  );
  const demuxFirstSampleMedianMs = requiredFiniteNumber(
    value.demuxFirstSampleMedianMs,
    `${label}.demuxFirstSampleMedianMs`,
  );
  const demuxFirstSampleP90Ms = requiredFiniteNumber(
    value.demuxFirstSampleP90Ms,
    `${label}.demuxFirstSampleP90Ms`,
  );
  if (
    demuxFirstSampleMedianMs !== expectedDemux.medianMs ||
    demuxFirstSampleP90Ms !== expectedDemux.p90Ms
  ) {
    throw new Error(
      `${label} demux-first-sample summary does not match its runs`,
    );
  }
  const processTreePeakRssBytes =
    value.processTreePeakRssBytes === null
      ? null
      : requiredNonNegativeInteger(
          value.processTreePeakRssBytes,
          `${label}.processTreePeakRssBytes`,
        );
  const result = {
    ...numeric,
    firstSampleRunsMs,
    firstSampleMedianMs,
    firstSampleP90Ms,
    demuxFirstSampleRunsMs,
    demuxFirstSampleMedianMs,
    demuxFirstSampleP90Ms,
    processTreePeakRssBytes,
  } as StreamingPerformanceStats;
  assertStreamingBounds(
    {
      ...result,
      firstSampleMs: result.firstSampleMedianMs,
      demuxFirstSampleMs: result.demuxFirstSampleMedianMs,
    },
    label,
  );
  return result;
}

export function compareStreamingPerformance(
  current: StreamingPerformanceStats,
  baseline: StreamingPerformanceStats,
  medianRegressionRatio = 1.1,
  p90RegressionRatio = 1.15,
): string[] {
  const failures: string[] = [];
  if (
    current.firstSampleMedianMs >
    baseline.firstSampleMedianMs * medianRegressionRatio
  ) {
    failures.push(
      `first sample median regressed from ${baseline.firstSampleMedianMs}ms to ${current.firstSampleMedianMs}ms`,
    );
  }
  if (
    current.firstSampleP90Ms >
    baseline.firstSampleP90Ms * p90RegressionRatio
  ) {
    failures.push(
      `first sample p90 regressed from ${baseline.firstSampleP90Ms}ms to ${current.firstSampleP90Ms}ms`,
    );
  }
  for (const field of STABLE_FIELDS) {
    if (current[field] !== baseline[field]) {
      failures.push(
        `${field} changed from ${baseline[field]} to ${current[field]}`,
      );
    }
  }
  for (const field of DEMUX_PEAK_FIELDS) {
    if (current[field] > baseline[field]) {
      failures.push(
        `${field} increased from ${baseline[field]} to ${current[field]}`,
      );
    }
  }
  for (const field of [
    "spatialReadBytes",
    "spatialReadCount",
    "spatialCacheMisses",
    "peakSpatialBlobBytes",
    "peakSpatialCacheBytes",
    "peakSpatialRetainedBytes",
  ] as const) {
    if (current[field] > Math.max(baseline[field] + 1, baseline[field] * 1.1)) {
      failures.push(
        `${field} increased beyond tolerance from ${baseline[field]} to ${current[field]}`,
      );
    }
  }
  const currentRss = current.processTreePeakRssBytes;
  const baselineRss = baseline.processTreePeakRssBytes;
  if ((currentRss === null) !== (baselineRss === null)) {
    failures.push(
      `Chrome process-tree peak RSS support changed from ${formatOptionalBytes(baselineRss)} to ${formatOptionalBytes(currentRss)}`,
    );
  } else if (
    currentRss !== null &&
    baselineRss !== null &&
    currentRss > baselineRss * 1.1
  ) {
    failures.push(
      `Chrome process-tree peak RSS increased from ${baselineRss} to ${currentRss} bytes`,
    );
  }
  return failures;
}

function formatOptionalBytes(value: number | null): string {
  return value === null ? "unsupported" : `${value} bytes`;
}

function assertStreamingBounds(
  value: Omit<StreamingRunStats, "processTreePeakRssBytes">,
  label: string,
): void {
  const fixedConfiguration: ReadonlyArray<readonly [number, number, string]> = [
    [value.demuxChunkBytes, DEMUX_METADATA_CHUNK_BYTES, "demux chunk bytes"],
    [
      value.maxEncodedSampleBytes,
      MAX_ENCODED_SAMPLE_BYTES,
      "encoded sample bytes",
    ],
    [value.maxBatchBytes, MAX_ENCODED_BATCH_BYTES, "batch bytes"],
    [value.maxMetadataBytes, MAX_DEMUX_METADATA_BYTES, "metadata bytes"],
    [
      value.maxMetadataMp4BufferBytes,
      MAX_DEMUX_METADATA_MP4_BUFFER_BYTES,
      "metadata MP4 buffer bytes",
    ],
    [
      value.maxMediaMp4BufferBytes,
      MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES,
      "media MP4 buffer bytes",
    ],
    [value.maxMp4SampleBytes, MAX_DEMUX_MP4_SAMPLE_BYTES, "MP4 sample bytes"],
    [
      value.maxDemuxRetainedBytes,
      MAX_DEMUX_RETAINED_BYTES,
      "demux retained bytes",
    ],
    [
      value.encodedSamplesHighWatermark,
      MAX_ENCODED_QUEUE_SAMPLES,
      "encoded sample high watermark",
    ],
    [
      value.encodedSamplesLowWatermark,
      ENCODED_QUEUE_SAMPLE_LOW_WATERMARK,
      "encoded sample low watermark",
    ],
    [
      value.encodedBytesHighWatermark,
      MAX_ENCODED_QUEUE_BYTES,
      "encoded byte high watermark",
    ],
    [
      value.encodedBytesLowWatermark,
      ENCODED_QUEUE_BYTE_LOW_WATERMARK,
      "encoded byte low watermark",
    ],
  ];
  for (const [actual, expected, field] of fixedConfiguration) {
    if (actual !== expected) {
      throw new Error(
        `${label} ${field} ${actual} does not match fixed limit ${expected}`,
      );
    }
  }
  if (value.maxBatchSamples <= 0) {
    throw new Error(`${label} batch sample limit must be positive`);
  }
  const bounds: ReadonlyArray<readonly [number, number, string]> = [
    [value.maxBatchSamples, MAX_ENCODED_BATCH_SAMPLES, "batch sample limit"],
    [value.observedMaxSampleBytes, value.maxEncodedSampleBytes, "sample"],
    [
      value.preflightReadBytes,
      value.maxMetadataBytes,
      "preflight metadata bytes",
    ],
    [
      value.demuxMetadataReadBytes,
      value.maxMetadataBytes,
      "demux metadata bytes",
    ],
    [value.peakMetadataBlobBytes, value.demuxChunkBytes, "metadata Blob read"],
    [value.peakMediaBlobBytes, value.maxBatchBytes, "media Blob read"],
    [
      value.peakBlobBytes,
      Math.max(value.demuxChunkBytes, value.maxBatchBytes),
      "Blob read",
    ],
    [
      value.peakMetadataMp4BufferBytes,
      value.maxMetadataMp4BufferBytes,
      "metadata MP4 buffer",
    ],
    [
      value.peakMediaMp4BufferBytes,
      value.maxMediaMp4BufferBytes,
      "media MP4 buffer",
    ],
    [
      value.peakMp4BufferBytes,
      Math.max(value.maxMetadataMp4BufferBytes, value.maxMediaMp4BufferBytes),
      "MP4 buffer",
    ],
    [value.peakMp4SampleBytes, value.maxMp4SampleBytes, "MP4 sample data"],
    [
      value.peakDemuxRetainedBytes,
      value.maxDemuxRetainedBytes,
      "demux retained bytes",
    ],
    [value.peakBatchSamples, value.maxBatchSamples, "batch samples"],
    [value.peakBatchBytes, value.maxBatchBytes, "batch bytes"],
    [
      value.peakEncodedSamples,
      value.encodedSamplesHighWatermark,
      "encoded queue samples",
    ],
    [
      value.peakEncodedBytes,
      value.encodedBytesHighWatermark,
      "encoded queue bytes",
    ],
    [
      value.peakSpatialBlobBytes,
      value.maxEncodedSampleBytes,
      "spatial Blob read",
    ],
    [
      value.peakSpatialCacheBytes,
      value.maxEncodedSampleBytes,
      "spatial cache",
    ],
    [
      value.peakSpatialRetainedBytes,
      value.maxEncodedSampleBytes * 2,
      "spatial retained bytes",
    ],
  ];
  for (const [observed, limit, field] of bounds) {
    if (observed > limit) {
      throw new Error(`${label} ${field} ${observed} exceeds limit ${limit}`);
    }
  }
  if (value.releasedSamples !== value.deliveredSamples) {
    throw new Error(
      `${label} released samples ${value.releasedSamples} do not equal delivered samples ${value.deliveredSamples}`,
    );
  }
}

function stableValue<K extends (typeof STABLE_FIELDS)[number]>(
  runs: readonly StreamingRunStats[],
  field: K,
  label: string,
): StreamingRunStats[K] {
  const expected = runs[0][field];
  if (runs.some((run) => run[field] !== expected)) {
    throw new Error(`${label}.${field} changed between measured runs`);
  }
  return expected;
}

function lastLog(
  logs: readonly string[],
  prefix: string,
  label: string,
): string {
  const line = optionalLastLog(logs, prefix);
  if (!line) throw new Error(`${label} is missing ${prefix}`);
  return line;
}

function optionalLastLog(
  logs: readonly string[],
  prefix: string,
): string | undefined {
  return [...logs].reverse().find((line) => line.startsWith(prefix));
}

function parseMappedFields<T extends Readonly<Record<string, string>>>(
  line: string,
  mapping: T,
  label: string,
): { readonly [K in keyof T]: number } {
  const tokens = new Map(
    [...line.matchAll(/([a-z0-9_]+)=([0-9]+(?:\.[0-9]+)?)(?:B|ms)?/g)].map(
      (match) => [match[1], Number(match[2])],
    ),
  );
  return Object.fromEntries(
    Object.entries(mapping).map(([field, token]) => {
      const value = tokens.get(token);
      const timing = token === "demux_first_sample";
      if (
        typeof value !== "number" ||
        !Number.isFinite(value) ||
        value < 0 ||
        (!timing && !Number.isSafeInteger(value))
      ) {
        throw new Error(`${label} is missing ${token}`);
      }
      return [field, value];
    }),
  ) as { readonly [K in keyof T]: number };
}

function zeroMappedFields<T extends Readonly<Record<string, string>>>(
  mapping: T,
): { readonly [K in keyof T]: number } {
  return Object.fromEntries(
    Object.keys(mapping).map((field) => [field, 0]),
  ) as {
    readonly [K in keyof T]: number;
  };
}

function requiredNonNegativeInteger(value: unknown, label: string): number {
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw new Error(`${label} must be a non-negative safe integer`);
  }
  return value as number;
}

function requiredFiniteNumber(value: unknown, label: string): number {
  if (typeof value !== "number" || !Number.isFinite(value) || value < 0) {
    throw new Error(`${label} must be a non-negative finite number`);
  }
  return value;
}

function assertExactKeys(
  value: Readonly<Record<string, unknown>>,
  fields: readonly string[],
  label: string,
): void {
  const expected = [...fields].sort();
  const actual = Object.keys(value).sort();
  if (JSON.stringify(expected) !== JSON.stringify(actual)) {
    throw new Error(`${label} fields must be ${expected.join(", ")}`);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
