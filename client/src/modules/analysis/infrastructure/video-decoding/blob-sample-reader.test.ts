import { describe, expect, test } from "bun:test";
import { MAX_ENCODED_SAMPLE_BYTES } from "../../domain/encoded-video-limits.js";
import type { FrameSample } from "../../domain/result.js";
import { BlobRangeReader } from "./blob-range-reader.js";
import { BlobSampleReader } from "./blob-sample-reader.js";

const SAMPLES: FrameSample[] = [
  { isSync: true, timestampUs: 0, offset: 10, size: 3 },
  { isSync: false, timestampUs: 1, offset: 13, size: 2 },
  { isSync: false, timestampUs: 2, offset: 20, size: 4 },
  { isSync: false, timestampUs: 3, offset: 80, size: 2 },
];

describe("BlobSampleReader", () => {
  test("coalesces nearby samples into one bounded cache range", async () => {
    const bytes = Uint8Array.from({ length: 100 }, (_, index) => index);
    const source = new Blob([bytes]);
    const reader = new BlobSampleReader(source, undefined, {
      maxCacheBytes: 32,
      maxSamplesPerRead: 3,
      maxGapBytes: 8,
    });

    expect(await reader.readSample(SAMPLES, 0)).toEqual(bytes.slice(10, 13));
    expect(await reader.readSample(SAMPLES, 1)).toEqual(bytes.slice(13, 15));
    expect(await reader.readSample(SAMPLES, 2)).toEqual(bytes.slice(20, 24));
    expect(reader.statistics).toEqual({
      readCount: 1,
      totalBytesRead: 14,
      peakReadBytes: 14,
      cacheHits: 2,
      cacheMisses: 1,
      peakCacheBytes: 14,
      peakRetainedBytes: 14,
    });

    expect(await reader.readSample(SAMPLES, 3)).toEqual(bytes.slice(80, 82));
    expect(reader.statistics.readCount).toBe(2);
    expect(reader.statistics.peakCacheBytes).toBeLessThanOrEqual(32);
  });

  test("bounds a sample larger than the preferred cache by the shared sample cap", async () => {
    const source = new Blob([new Uint8Array(1024)]);
    const reader = new BlobSampleReader(source, undefined, {
      maxCacheBytes: 16,
    });
    const value = await reader.readSample(
      [{ isSync: true, timestampUs: 123, offset: 100, size: 64 }],
      0,
    );

    expect(value.byteLength).toBe(64);
    expect(reader.statistics).toMatchObject({
      totalBytesRead: 64,
      peakCacheBytes: 64,
    });
  });

  test("rejects an oversized sample before issuing a Blob read", async () => {
    const source = {
      size: MAX_ENCODED_SAMPLE_BYTES + 1,
      slice(): Blob {
        throw new Error("oversized sample must not be read");
      },
    };
    const reader = new BlobSampleReader(source);

    expect(() =>
      reader.readSample(
        [
          {
            isSync: true,
            timestampUs: 0,
            offset: 0,
            size: MAX_ENCODED_SAMPLE_BYTES + 1,
          },
        ],
        0,
      ),
    ).toThrow("re-encode");
    expect(reader.statistics.readCount).toBe(0);
  });

  test("does not cache a range whose read completes after abort", async () => {
    const source = new Blob([new Uint8Array(32)]);
    const controller = new AbortController();
    let finishRead!: (buffer: ArrayBuffer) => void;
    const rangeReader = new BlobRangeReader(source, controller.signal, {
      readSlice: () =>
        new Promise<ArrayBuffer>((resolve) => {
          finishRead = resolve;
        }),
    });
    const reader = new BlobSampleReader(source, controller.signal, {
      reader: rangeReader,
    });
    const reason = new Error("spatial read cancelled");
    const reading = Promise.resolve(
      reader.readSample(
        [{ isSync: true, timestampUs: 0, offset: 0, size: 8 }],
        0,
      ),
    );

    controller.abort(reason);
    finishRead(new ArrayBuffer(8));

    expect(await reading.catch((error) => error)).toBe(reason);
    expect(reader.statistics).toMatchObject({
      readCount: 0,
      cacheHits: 0,
      cacheMisses: 1,
      peakCacheBytes: 0,
      peakRetainedBytes: 8,
    });
  });
});
