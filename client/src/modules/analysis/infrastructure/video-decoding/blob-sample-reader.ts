import { MAX_ENCODED_SAMPLE_BYTES } from "../../domain/encoded-video-limits.js";
import type { FrameSample } from "../../domain/result.js";
import {
  BlobRangeReader,
  type BlobRangeReaderStats,
  type BlobSliceSource,
} from "./blob-range-reader.js";

const DEFAULT_CACHE_BYTES = 4 * 1024 * 1024;
const DEFAULT_MAX_SAMPLES = 16;
const DEFAULT_MAX_GAP_BYTES = 64 * 1024;

export interface BlobSampleReaderStats extends BlobRangeReaderStats {
  readonly cacheHits: number;
  readonly cacheMisses: number;
  readonly peakCacheBytes: number;
  /** Cache plus an in-flight replacement read; this is not process RSS. */
  readonly peakRetainedBytes: number;
}

interface BlobSampleReaderDependencies {
  readonly reader?: BlobRangeReader;
  readonly maxCacheBytes?: number;
  readonly maxSamplesPerRead?: number;
  readonly maxGapBytes?: number;
}

/**
 * Maintains one bounded cache for sample offset/size reads.
 *
 * The returned view is valid until the caller releases it. The analysis path
 * passes it immediately to EncodedVideoChunk, whose constructor copies data
 * when no transfer list is supplied, before requesting another sample.
 */
export class BlobSampleReader {
  readonly #source: BlobSliceSource;
  readonly #reader: BlobRangeReader;
  readonly #maxCacheBytes: number;
  readonly #maxSamplesPerRead: number;
  readonly #maxGapBytes: number;
  #cache: { readonly offset: number; readonly buffer: ArrayBuffer } | undefined;
  #cacheHits = 0;
  #cacheMisses = 0;
  #peakCacheBytes = 0;
  #peakRetainedBytes = 0;

  constructor(
    source: BlobSliceSource,
    signal?: AbortSignal,
    dependencies: BlobSampleReaderDependencies = {},
  ) {
    this.#source = source;
    this.#reader = dependencies.reader ?? new BlobRangeReader(source, signal);
    this.#maxCacheBytes = dependencies.maxCacheBytes ?? DEFAULT_CACHE_BYTES;
    this.#maxSamplesPerRead =
      dependencies.maxSamplesPerRead ?? DEFAULT_MAX_SAMPLES;
    this.#maxGapBytes = dependencies.maxGapBytes ?? DEFAULT_MAX_GAP_BYTES;
    assertPositiveInteger(this.#maxCacheBytes, "sample cache bytes");
    assertPositiveInteger(
      this.#maxSamplesPerRead,
      "samples per Blob range read",
    );
    if (!Number.isSafeInteger(this.#maxGapBytes) || this.#maxGapBytes < 0) {
      throw new Error("sample cache gap bytes must be non-negative");
    }
  }

  get statistics(): BlobSampleReaderStats {
    return {
      ...this.#reader.statistics,
      cacheHits: this.#cacheHits,
      cacheMisses: this.#cacheMisses,
      peakCacheBytes: this.#peakCacheBytes,
      peakRetainedBytes: this.#peakRetainedBytes,
    };
  }

  readSample(
    samples: readonly FrameSample[],
    sampleIndex: number,
  ): Uint8Array<ArrayBuffer> | Promise<Uint8Array<ArrayBuffer>> {
    const sample = samples[sampleIndex];
    if (!sample) throw new Error(`Missing encoded sample ${sampleIndex}`);
    assertSampleRange(sample, this.#source.size);

    const cached = this.#viewFromCache(sample);
    if (cached) {
      this.#cacheHits += 1;
      return cached;
    }

    this.#cacheMisses += 1;
    return this.#loadSample(samples, sampleIndex, sample);
  }

  async #loadSample(
    samples: readonly FrameSample[],
    sampleIndex: number,
    sample: FrameSample,
  ): Promise<Uint8Array<ArrayBuffer>> {
    const range = coalescedRange(
      samples,
      sampleIndex,
      this.#source.size,
      this.#maxCacheBytes,
      this.#maxSamplesPerRead,
      this.#maxGapBytes,
    );
    this.#peakRetainedBytes = Math.max(
      this.#peakRetainedBytes,
      (this.#cache?.buffer.byteLength ?? 0) + range.size,
    );
    const buffer = await this.#reader.read(range.offset, range.size);
    this.#cache = { offset: range.offset, buffer };
    this.#peakCacheBytes = Math.max(this.#peakCacheBytes, buffer.byteLength);
    const loaded = this.#viewFromCache(sample);
    if (!loaded)
      throw new Error(`Encoded sample ${sampleIndex} was not loaded`);
    return loaded;
  }

  stop(reason?: unknown): void {
    this.#cache = undefined;
    this.#reader.stop(reason);
  }

  #viewFromCache(sample: FrameSample): Uint8Array<ArrayBuffer> | undefined {
    const cache = this.#cache;
    if (
      !cache ||
      sample.offset < cache.offset ||
      sample.offset + sample.size > cache.offset + cache.buffer.byteLength
    ) {
      return undefined;
    }
    return new Uint8Array(
      cache.buffer,
      sample.offset - cache.offset,
      sample.size,
    );
  }
}

function coalescedRange(
  samples: readonly FrameSample[],
  firstIndex: number,
  sourceSize: number,
  maxCacheBytes: number,
  maxSamples: number,
  maxGapBytes: number,
): { readonly offset: number; readonly size: number } {
  const first = samples[firstIndex]!;
  const offset = first.offset;
  let end = first.offset + first.size;
  const byteLimit = Math.max(maxCacheBytes, first.size);
  for (
    let index = firstIndex + 1;
    index < samples.length && index < firstIndex + maxSamples;
    index += 1
  ) {
    const next = samples[index];
    assertSampleRange(next, sourceSize);
    if (next.offset < end || next.offset - end > maxGapBytes) break;
    const nextEnd = next.offset + next.size;
    if (nextEnd - offset > byteLimit) break;
    end = nextEnd;
  }
  return { offset, size: end - offset };
}

function assertSampleRange(sample: FrameSample, sourceSize: number): void {
  if (sample.size > MAX_ENCODED_SAMPLE_BYTES) {
    throw new Error(
      `Encoded sample exceeds ${MAX_ENCODED_SAMPLE_BYTES} bytes; re-encode the video at a lower bitrate`,
    );
  }
  if (
    !Number.isSafeInteger(sample.offset) ||
    sample.offset < 0 ||
    !Number.isSafeInteger(sample.size) ||
    sample.size <= 0 ||
    sample.offset > sourceSize ||
    sample.size > sourceSize - sample.offset
  ) {
    throw new Error("Encoded sample range is outside the video Blob");
  }
}

function assertPositiveInteger(value: number, label: string): void {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`${label} must be a positive safe integer`);
  }
}
