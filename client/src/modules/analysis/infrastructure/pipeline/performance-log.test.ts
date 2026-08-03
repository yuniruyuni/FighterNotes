import { expect, spyOn, test } from "bun:test";
import { logPerformance } from "./performance-log.js";

test("logs preflight reads separately from first-pass demux retention", () => {
  const lines: string[] = [];
  const log = spyOn(console, "log").mockImplementation((...values) => {
    lines.push(values.map(String).join(" "));
  });
  try {
    logPerformance({
      frameIndex: 10,
      tDraw: 1,
      tCopy: 2,
      tMeter: 3,
      tHud: 4,
      streaming: {
        videoBytes: 1_000_000,
        preflightMetadataBytes: 1000,
        demux: {
          readCount: 10,
          totalBytesRead: 900_000,
          peakReadBytes: 1024,
          chunkBytes: 1024,
          extractionBatchSamples: 8,
          maxEncodedSampleBytes: 16 * 1024 * 1024,
          observedMaxSampleBytes: 2048,
          maxExtractionBatchBytes: 16 * 1024 * 1024,
          maxMetadataBytes: 32 * 1024 * 1024,
          maxMetadataMp4BufferBytes: 32 * 1024 * 1024,
          maxMediaMp4BufferBytes: 48 * 1024 * 1024,
          maxMp4SampleBytes: 16 * 1024 * 1024,
          maxDemuxRetainedBytes: 96 * 1024 * 1024,
          metadataSparseRangeCount: 2,
          metadataSparseRangeOperations: 20,
          metadataReadCount: 2,
          metadataBytesRead: 2000,
          peakMetadataReadBytes: 1000,
          mediaReadCount: 8,
          mediaBytesRead: 898_000,
          peakMediaReadBytes: 1024,
          deliveredSamples: 100,
          releasedSamples: 100,
          peakBatchSamples: 8,
          peakBatchBytes: 2048,
          peakMp4BufferBytes: 4096,
          peakMetadataMp4BufferBytes: 4096,
          peakMediaMp4BufferBytes: 2048,
          peakMp4SampleBytes: 2048,
          peakDemuxRetainedBytes: 8192,
          timeToFirstSampleMs: 12,
        },
        encodedQueue: {
          maxQueuedSamples: 16,
          queuedSampleLowWatermark: 8,
          maxQueuedBytes: 32 * 1024 * 1024,
          queuedByteLowWatermark: 16 * 1024 * 1024,
          peakQueuedSamples: 10,
          peakQueuedBytes: 8192,
        },
      },
    });
  } finally {
    log.mockRestore();
  }

  expect(lines).toHaveLength(2);
  expect(lines[1]).toContain("preflight_read=1000B");
  expect(lines[1]).toContain("demux_read=900000B");
  expect(lines[1]).toContain("demux_metadata_read=2000B");
  expect(lines[1]).toContain("demux_media_read=898000B");
  expect(lines[1]).toContain("peak_mp4_buffer=4096B");
  expect(lines[1]).toContain("peak_encoded_queue=8192B");
  expect(lines[1]).toContain("sample_max=16777216B");
  expect(lines[1]).toContain("metadata_bytes_max=33554432B");
  expect(lines[1]).toContain("demux_retained_max=100663296B");
  expect(lines[1]).toContain("encoded_bytes_high=33554432B");
  expect(lines[1]).toContain("demux_first_sample=12ms");
});
