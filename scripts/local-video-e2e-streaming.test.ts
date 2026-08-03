import { describe, expect, test } from "bun:test";
import {
  compareStreamingPerformance,
  parseStreamingRunStats,
  summarizeStreamingPerformanceStats,
} from "./local-video-e2e-streaming";

const STREAMING_LOG =
  "[perf] streaming" +
  " video=1000000B preflight_read=1000B demux_read=900000B demux_reads=10" +
  " demux_chunk=1048576B demux_metadata_read=2000B demux_media_read=898000B" +
  " metadata_ranges=2 metadata_range_ops=20 sample_max=16777216B" +
  " observed_sample_max=2048B batch_samples_max=8 batch_bytes_max=16777216B" +
  " metadata_bytes_max=33554432B metadata_mp4_buffer_max=33554432B" +
  " media_mp4_buffer_max=50331648B mp4_sample_bytes_max=16777216B" +
  " demux_retained_max=100663296B" +
  " peak_blob=1048576B peak_metadata_blob=1048576B peak_media_blob=2048B" +
  " peak_mp4_buffer=1048576B peak_metadata_mp4_buffer=1048576B" +
  " peak_media_mp4_buffer=2048B peak_mp4_sample=16384B peak_demux_retained=2097152B" +
  " peak_batch_samples=8 peak_batch=16384B delivered_samples=100 released_samples=100" +
  " peak_encoded_samples=8 peak_encoded_queue=32768B encoded_samples_high=16" +
  " encoded_samples_low=8 encoded_bytes_high=33554432B encoded_bytes_low=16777216B" +
  " demux_first_sample=12ms";

const SPATIAL_LOG =
  "[perf] spatial-blob reads=2 read=8192 peak_blob=4096 peak_cache=4096" +
  " peak_spatial_retained=8192 cache_hits=6 cache_misses=2";

describe("local video E2E streaming stats", () => {
  test("parses configured limits, observed peaks, Blob cache, and RSS", () => {
    const parsed = parseStreamingRunStats(
      [STREAMING_LOG, SPATIAL_LOG],
      "fixture.streaming",
      15,
      200 * 1024 * 1024,
    );

    expect(parsed).toMatchObject({
      firstSampleMs: 15,
      demuxFirstSampleMs: 12,
      maxEncodedSampleBytes: 16 * 1024 * 1024,
      maxBatchSamples: 8,
      encodedBytesHighWatermark: 32 * 1024 * 1024,
      peakDemuxRetainedBytes: 2 * 1024 * 1024,
      peakSpatialRetainedBytes: 8192,
      processTreePeakRssBytes: 200 * 1024 * 1024,
    });
  });

  test("summarizes startup and highest component/RSS peaks", () => {
    const first = parseStreamingRunStats(
      [STREAMING_LOG, SPATIAL_LOG],
      "first",
      15,
      100,
    );
    const second = {
      ...first,
      firstSampleMs: 20,
      peakEncodedBytes: first.peakEncodedBytes + 1,
      spatialReadCount: first.spatialReadCount + 1,
      processTreePeakRssBytes: 120,
    };
    const summary = summarizeStreamingPerformanceStats(
      [first, second],
      "streaming",
    );

    expect(summary.firstSampleRunsMs).toEqual([15, 20]);
    expect(summary.firstSampleMedianMs).toBe(17.5);
    expect(summary.demuxFirstSampleRunsMs).toEqual([12, 12]);
    expect(summary.demuxFirstSampleMedianMs).toBe(12);
    expect(summary.peakEncodedBytes).toBe(second.peakEncodedBytes);
    expect(summary.spatialReadCount).toBe(second.spatialReadCount);
    expect(summary.processTreePeakRssBytes).toBe(120);
  });

  test("rejects hard-limit and baseline component regressions", () => {
    const baseline = summarizeStreamingPerformanceStats(
      [parseStreamingRunStats([STREAMING_LOG, SPATIAL_LOG], "base", 15, null)],
      "base",
    );
    expect(() =>
      parseStreamingRunStats(
        [
          STREAMING_LOG.replace(
            "peak_encoded_samples=8",
            "peak_encoded_samples=17",
          ),
          SPATIAL_LOG,
        ],
        "overflow",
        15,
        null,
      ),
    ).toThrow("exceeds limit");
    expect(
      compareStreamingPerformance(
        {
          ...baseline,
          peakDemuxRetainedBytes: baseline.peakDemuxRetainedBytes + 1,
        },
        baseline,
      ),
    ).toContain(
      `peakDemuxRetainedBytes increased from ${baseline.peakDemuxRetainedBytes} to ${baseline.peakDemuxRetainedBytes + 1}`,
    );
  });

  test("rejects tampered configured limits and absolute demux retention overflow", () => {
    expect(() =>
      parseStreamingRunStats(
        [
          STREAMING_LOG.replace(
            "demux_retained_max=100663296B",
            "demux_retained_max=209715200B",
          ),
          SPATIAL_LOG,
        ],
        "tampered-limit",
        15,
        null,
      ),
    ).toThrow("does not match fixed limit");
    expect(() =>
      parseStreamingRunStats(
        [
          STREAMING_LOG.replace(
            "peak_demux_retained=2097152B",
            "peak_demux_retained=209715200B",
          ),
          SPATIAL_LOG,
        ],
        "absolute-overflow",
        15,
        null,
      ),
    ).toThrow("demux retained bytes 209715200 exceeds limit 100663296");
    expect(() =>
      parseStreamingRunStats(
        [
          STREAMING_LOG.replace("batch_samples_max=8", "batch_samples_max=0"),
          SPATIAL_LOG,
        ],
        "zero-batch",
        15,
        null,
      ),
    ).toThrow("batch sample limit must be positive");
  });

  test("keeps timing-dependent queue peaks informational and separates p90 policy", () => {
    const baseline = summarizeStreamingPerformanceStats(
      [parseStreamingRunStats([STREAMING_LOG, SPATIAL_LOG], "base", 15, 100)],
      "base",
    );
    const current = {
      ...baseline,
      peakEncodedSamples: baseline.peakEncodedSamples + 1,
      peakEncodedBytes: baseline.peakEncodedBytes + 1,
      firstSampleP90Ms: baseline.firstSampleP90Ms * 1.02,
    };

    expect(compareStreamingPerformance(current, baseline, 2, 1.01)).toEqual([
      `first sample p90 regressed from ${baseline.firstSampleP90Ms}ms to ${current.firstSampleP90Ms}ms`,
    ]);
  });

  test("fails RSS support mismatch and mixed support across measured runs", () => {
    const supported = parseStreamingRunStats(
      [STREAMING_LOG, SPATIAL_LOG],
      "supported",
      15,
      100,
    );
    const unsupported = { ...supported, processTreePeakRssBytes: null };
    expect(
      compareStreamingPerformance(supported, {
        ...summarizeStreamingPerformanceStats([supported], "base"),
        processTreePeakRssBytes: null,
      }),
    ).toContain(
      "Chrome process-tree peak RSS support changed from unsupported to 100 bytes",
    );
    expect(() =>
      summarizeStreamingPerformanceStats([supported, unsupported], "mixed-rss"),
    ).toThrow("support changed between runs");
  });

  test("rejects unsafe integer logs and delivered/released mismatches", () => {
    expect(() =>
      parseStreamingRunStats(
        [
          STREAMING_LOG.replace("video=1000000B", "video=9007199254740992B"),
          SPATIAL_LOG,
        ],
        "unsafe",
        15,
        null,
      ),
    ).toThrow("missing video");
    expect(() =>
      parseStreamingRunStats(
        [
          STREAMING_LOG.replace("released_samples=100", "released_samples=99"),
          SPATIAL_LOG,
        ],
        "unreleased",
        15,
        null,
      ),
    ).toThrow("do not equal delivered samples");
  });
});
