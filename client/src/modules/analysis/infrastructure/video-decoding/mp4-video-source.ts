import {
  createFile,
  type ISOFile,
  type Movie,
  type MP4BoxBuffer,
  type Sample,
} from "mp4box";
import {
  DEMUX_METADATA_CHUNK_BYTES,
  MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES,
  MAX_DEMUX_METADATA_BYTES,
  MAX_DEMUX_METADATA_MP4_BUFFER_BYTES,
  MAX_DEMUX_MP4_SAMPLE_BYTES,
  MAX_DEMUX_RETAINED_BYTES,
  MAX_ENCODED_BATCH_BYTES,
  MAX_ENCODED_BATCH_SAMPLES,
  MAX_ENCODED_SAMPLE_BYTES,
} from "../../domain/encoded-video-limits.js";
import type { FrameSample, VideoCodecConfig } from "../../domain/result.js";
import type { InspectedVideoTrack } from "../../domain/video-preflight.js";
import {
  BlobRangeReader,
  type BlobRangeReaderStats,
  type BlobSliceSource,
} from "./blob-range-reader.js";
import { mp4TimestampUs } from "./sample-timestamp-index.js";

const DEFAULT_DEMUX_CHUNK_BYTES = DEMUX_METADATA_CHUNK_BYTES;
const DEFAULT_EXTRACTION_BATCH_SAMPLES = MAX_ENCODED_BATCH_SAMPLES;

export interface Mp4VideoTrack {
  readonly totalSamples: number;
  readonly decoderConfig: VideoDecoderConfig;
  readonly codecConfig: VideoCodecConfig;
}

export interface Mp4VideoSample {
  readonly metadata: FrameSample;
  readonly chunk: EncodedVideoChunk;
}

export interface Mp4VideoSourceStats extends BlobRangeReaderStats {
  readonly chunkBytes: number;
  readonly extractionBatchSamples: number;
  readonly maxEncodedSampleBytes: number;
  readonly observedMaxSampleBytes: number;
  readonly maxExtractionBatchBytes: number;
  readonly maxMetadataBytes: number;
  readonly maxMetadataMp4BufferBytes: number;
  readonly maxMediaMp4BufferBytes: number;
  readonly maxMp4SampleBytes: number;
  readonly maxDemuxRetainedBytes: number;
  readonly metadataSparseRangeCount: number;
  readonly metadataSparseRangeOperations: number;
  readonly metadataReadCount: number;
  readonly metadataBytesRead: number;
  readonly peakMetadataReadBytes: number;
  readonly mediaReadCount: number;
  readonly mediaBytesRead: number;
  readonly peakMediaReadBytes: number;
  readonly deliveredSamples: number;
  readonly releasedSamples: number;
  readonly peakBatchSamples: number;
  readonly peakBatchBytes: number;
  readonly peakMp4BufferBytes: number;
  readonly peakMetadataMp4BufferBytes: number;
  readonly peakMediaMp4BufferBytes: number;
  readonly peakMp4SampleBytes: number;
  /** Logical demux-owned bytes; this is not browser process RSS. */
  readonly peakDemuxRetainedBytes: number;
  readonly timeToFirstSampleMs: number | null;
}

interface Mp4VideoSourceCallbacks {
  readonly onTrack: (track: Mp4VideoTrack) => Promise<void>;
  readonly onSamples: (samples: readonly Mp4VideoSample[]) => void;
  readonly onError: (error: unknown) => void;
}

interface Mp4VideoSourceDependencies {
  readonly file?: ISOFile;
  readonly reader?: BlobRangeReader;
  readonly signal?: AbortSignal;
  readonly chunkBytes?: number;
  readonly extractionBatchSamples?: number;
  readonly createChunk?: (sample: Sample) => EncodedVideoChunk;
  readonly now?: () => number;
}

/**
 * Pull-driven MP4Box source.
 *
 * One pull may read multiple sparse Blob ranges, but it yields at most one
 * extraction batch. MP4Box is stopped synchronously in onSamples so neither
 * parsing nor extraction can run ahead of the downstream decode queue.
 */
export class Mp4VideoSource {
  readonly #source: BlobSliceSource;
  readonly #callbacks: Mp4VideoSourceCallbacks;
  readonly #file: ISOFile;
  readonly #reader: BlobRangeReader;
  readonly #validatedTrack: InspectedVideoTrack;
  readonly #chunkBytes: number;
  readonly #requestedExtractionBatchSamples: number;
  readonly #createChunk: (sample: Sample) => EncodedVideoChunk;
  readonly #now: () => number;
  readonly #readRanges = new SparseReadRanges();
  readonly #pullWaiters = new Set<() => void>();
  #started = false;
  #stopped = false;
  #failed = false;
  #pullRequested = false;
  #nextOffset = 0;
  #flushed = false;
  #trackId: number | undefined;
  #totalSamples: number | undefined;
  #nextSampleNumber = 0;
  #deliveredSamples = 0;
  #releasedSamples = 0;
  #batchSequence = 0;
  #decoderPrepared = false;
  #startedAt = 0;
  #timeToFirstSampleMs: number | null = null;
  #peakBatchSamples = 0;
  #peakBatchBytes = 0;
  #peakMp4BufferBytes = 0;
  #peakMetadataMp4BufferBytes = 0;
  #peakMediaMp4BufferBytes = 0;
  #peakMp4SampleBytes = 0;
  #peakDemuxRetainedBytes = 0;
  #metadataReadCount = 0;
  #metadataBytesRead = 0;
  #peakMetadataReadBytes = 0;
  #mediaReadCount = 0;
  #mediaBytesRead = 0;
  #peakMediaReadBytes = 0;
  #activeReadKind: "metadata" | "media" | undefined;
  #lastMediaRequest: string | undefined;
  #extractionBatchSamples: number;
  #maxSampleBytes = 0;

  constructor(
    source: BlobSliceSource,
    callbacks: Mp4VideoSourceCallbacks,
    validatedTrack: InspectedVideoTrack,
    dependencies: Mp4VideoSourceDependencies = {},
  ) {
    this.#source = source;
    this.#callbacks = callbacks;
    this.#validatedTrack = validatedTrack;
    this.#file = dependencies.file ?? createFile();
    this.#reader =
      dependencies.reader ?? new BlobRangeReader(source, dependencies.signal);
    this.#chunkBytes = dependencies.chunkBytes ?? DEFAULT_DEMUX_CHUNK_BYTES;
    this.#requestedExtractionBatchSamples =
      dependencies.extractionBatchSamples ?? DEFAULT_EXTRACTION_BATCH_SAMPLES;
    this.#extractionBatchSamples = this.#requestedExtractionBatchSamples;
    this.#createChunk = dependencies.createChunk ?? encodedVideoChunk;
    this.#now = dependencies.now ?? (() => performance.now());
    assertPositiveInteger(this.#chunkBytes, "demux chunk bytes");
    assertPositiveInteger(
      this.#requestedExtractionBatchSamples,
      "extraction batch samples",
    );
    if (this.#requestedExtractionBatchSamples > MAX_ENCODED_BATCH_SAMPLES) {
      throw new Error(
        `extraction batch samples must not exceed ${MAX_ENCODED_BATCH_SAMPLES}`,
      );
    }
  }

  get statistics(): Mp4VideoSourceStats {
    return {
      ...this.#reader.statistics,
      chunkBytes: this.#chunkBytes,
      extractionBatchSamples: this.#extractionBatchSamples,
      maxEncodedSampleBytes: MAX_ENCODED_SAMPLE_BYTES,
      observedMaxSampleBytes: this.#maxSampleBytes,
      maxExtractionBatchBytes: MAX_ENCODED_BATCH_BYTES,
      maxMetadataBytes: MAX_DEMUX_METADATA_BYTES,
      maxMetadataMp4BufferBytes: MAX_DEMUX_METADATA_MP4_BUFFER_BYTES,
      maxMediaMp4BufferBytes: MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES,
      maxMp4SampleBytes: MAX_DEMUX_MP4_SAMPLE_BYTES,
      maxDemuxRetainedBytes: MAX_DEMUX_RETAINED_BYTES,
      metadataSparseRangeCount: this.#readRanges.rangeCount,
      metadataSparseRangeOperations: this.#readRanges.operations,
      metadataReadCount: this.#metadataReadCount,
      metadataBytesRead: this.#metadataBytesRead,
      peakMetadataReadBytes: this.#peakMetadataReadBytes,
      mediaReadCount: this.#mediaReadCount,
      mediaBytesRead: this.#mediaBytesRead,
      peakMediaReadBytes: this.#peakMediaReadBytes,
      deliveredSamples: this.#deliveredSamples,
      releasedSamples: this.#releasedSamples,
      peakBatchSamples: this.#peakBatchSamples,
      peakBatchBytes: this.#peakBatchBytes,
      peakMp4BufferBytes: this.#peakMp4BufferBytes,
      peakMetadataMp4BufferBytes: this.#peakMetadataMp4BufferBytes,
      peakMediaMp4BufferBytes: this.#peakMediaMp4BufferBytes,
      peakMp4SampleBytes: this.#peakMp4SampleBytes,
      peakDemuxRetainedBytes: this.#peakDemuxRetainedBytes,
      timeToFirstSampleMs: this.#timeToFirstSampleMs,
    };
  }

  start(analysisStartedAtMs = this.#now()): void {
    if (this.#started) throw new Error("MP4 video source already started");
    if (this.#stopped) return;
    if (!Number.isFinite(analysisStartedAtMs)) {
      throw new Error("analysis start time must be finite");
    }
    this.#started = true;
    this.#startedAt = analysisStartedAtMs;
    this.#file.onReady = (info) => {
      if (this.#stopped || this.#trackId !== undefined) return;
      this.#startVideoTrack(info);
    };
    this.#file.onSamples = (id, _user, samples) => {
      if (this.#stopped || samples.length === 0) return;
      this.#file.stop();
      try {
        this.#observeMp4Memory();
        this.#deliverSamples(id, samples);
        this.#batchSequence += 1;
      } catch (error) {
        this.#reportError(error);
      }
    };
    this.#file.onError = (module, message) => {
      this.#reportError(new Error(`${module}: ${message}`));
    };

    this.pull();
    void this.#prepareDecoderAndRun().catch((error) =>
      this.#reportError(error),
    );
  }

  /** Grants permission to produce at most one more extraction batch. */
  pull(): void {
    if (this.#stopped) return;
    this.#pullRequested = true;
    this.#wakePullWaiters();
  }

  stop(reason: unknown = new Error("MP4 video source stopped")): void {
    if (this.#stopped) return;
    this.#stopped = true;
    this.#pullRequested = false;
    this.#reader.stop(reason);
    this.#file.onReady = undefined;
    this.#file.onSamples = undefined;
    this.#file.onError = undefined;
    this.#file.stop();
    this.#wakePullWaiters();
  }

  async #run(): Promise<void> {
    while (!this.#stopped && !this.#allSamplesDelivered()) {
      if (!(await this.#takePull())) return;
      const produced = await this.#produceOneBatch();
      if (!produced && !this.#allSamplesDelivered()) {
        throw new Error(
          `MP4 sample extraction ended after ${this.#deliveredSamples} of ${
            this.#totalSamples ?? "unknown"
          } samples`,
        );
      }
    }
  }

  async #produceOneBatch(): Promise<boolean> {
    const initialSequence = this.#batchSequence;
    while (!this.#stopped && this.#batchSequence === initialSequence) {
      if (this.#trackId !== undefined) {
        this.#file.start();
        this.#observeMp4Memory();
        if (this.#batchSequence !== initialSequence) {
          return true;
        }
        this.#nextOffset = chooseNextOffset(
          this.#nextOffset,
          this.#file.nextSeekPosition,
          this.#source.size,
          this.#readRanges,
        );
      }

      const mediaRange = this.#nextMediaRange();
      // Metadata reads are a parser-navigation history, not proof that MP4Box
      // still owns the bytes. With discardMdatData=true, an early mdat slice
      // is cleaned before a tail moov makes the sample table available. Media
      // reads therefore intentionally bypass the metadata range set.
      const offset = mediaRange
        ? mediaRange.offset
        : this.#readRanges.nextUnread(this.#nextOffset);
      if (offset >= this.#source.size) {
        if (this.#flushed) return false;
        this.#flushed = true;
        this.#file.flush();
        this.#observeMp4Memory();
        if (this.#batchSequence !== initialSequence) {
          return true;
        }
        return false;
      }

      const readKind = mediaRange ? "media" : "metadata";
      const requestedSize = mediaRange
        ? mediaRange.size
        : this.#readRanges.readableBytes(
            offset,
            this.#chunkBytes,
            this.#source.size,
          );
      const size =
        readKind === "metadata"
          ? this.#boundedMetadataReadSize(requestedSize)
          : requestedSize;
      if (size <= 0) {
        throw new Error(
          `MP4 sparse reader made no progress at offset ${offset}`,
        );
      }
      if (mediaRange) {
        const request = `${mediaRange.sampleIndex}:${offset}:${size}`;
        if (request === this.#lastMediaRequest) {
          throw new Error(
            `MP4 media reader made no progress at sample ${mediaRange.sampleIndex}`,
          );
        }
        this.#lastMediaRequest = request;
      }
      const raw = await this.#reader.read(offset, size);
      if (this.#stopped) return false;
      this.#recordRead(readKind, raw.byteLength);
      this.#observeMp4Memory(raw);
      const input = raw as MP4BoxBuffer;
      input.fileStart = offset;
      const end = offset + input.byteLength;
      if (readKind === "metadata") this.#readRanges.add(offset, end);
      // Sparse reads may visit a tail moov before earlier media ranges. Passing
      // `last=true` merely because this slice touches physical EOF would make
      // MP4Box flush an incomplete logical stream.
      this.#activeReadKind = readKind;
      let suggested: number | undefined;
      try {
        suggested = this.#file.appendBuffer(input, false);
        this.#observeMp4Memory(raw);
      } finally {
        this.#activeReadKind = undefined;
      }
      this.#nextOffset = chooseNextOffset(
        end,
        suggested,
        this.#source.size,
        this.#readRanges,
      );
      if (this.#batchSequence !== initialSequence) {
        return true;
      }
    }
    return this.#batchSequence !== initialSequence;
  }

  async #prepareDecoderAndRun(): Promise<void> {
    await this.#callbacks.onTrack({
      totalSamples: this.#validatedTrack.totalSamples,
      decoderConfig: this.#validatedTrack.decoderConfig,
      codecConfig: this.#validatedTrack.codecConfig,
    });
    if (this.#stopped) return;
    this.#decoderPrepared = true;
    await this.#run();
  }

  #startVideoTrack(info: Movie): void {
    const track = info.videoTracks[0];
    if (!track?.video) throw new Error("動画トラックが見つかりません");
    if (!this.#decoderPrepared) {
      throw new Error("MP4 extraction started before decoder preparation");
    }
    assertTrackMatchesPreflight(track, this.#validatedTrack);
    const samples = this.#file.getTrackSamplesInfo(track.id) ?? [];
    this.#maxSampleBytes = validateEncodedSamples(
      samples,
      this.#validatedTrack,
    );
    this.#extractionBatchSamples = Math.min(
      this.#requestedExtractionBatchSamples,
      Math.max(1, Math.floor(MAX_ENCODED_BATCH_BYTES / this.#maxSampleBytes)),
    );
    this.#trackId = track.id;
    this.#totalSamples = this.#validatedTrack.totalSamples;
    this.#file.setExtractionOptions(track.id, undefined, {
      nbSamples: this.#extractionBatchSamples,
    });
    this.#file.start();
    this.#observeMp4Memory();
  }

  #deliverSamples(id: number, samples: Sample[]): void {
    if (this.#stopped) return;
    if (id !== this.#trackId) {
      throw new Error(`Unexpected MP4 sample track ${id}`);
    }
    assertConsecutiveSamples(samples, this.#nextSampleNumber, id);
    const batchBytes = samples.reduce((sum, sample) => sum + sample.size, 0);
    if (
      samples.length > this.#extractionBatchSamples ||
      samples.length > MAX_ENCODED_BATCH_SAMPLES
    ) {
      throw new Error("MP4 extraction exceeded its fixed sample batch limit");
    }
    if (batchBytes > MAX_ENCODED_BATCH_BYTES) {
      throw new Error("MP4 extraction exceeded its fixed byte batch limit");
    }
    this.#peakBatchSamples = Math.max(this.#peakBatchSamples, samples.length);
    this.#peakBatchBytes = Math.max(this.#peakBatchBytes, batchBytes);

    // EncodedVideoChunk copies init.data when transfer is omitted. Only release
    // MP4Box-owned sample bytes after every constructor completed successfully.
    const converted = samples.map((sample) =>
      toVideoSample(sample, this.#createChunk),
    );
    if (this.#stopped) return;
    this.#observeMp4Memory(undefined, batchBytes);
    const releaseUntil = samples.at(-1)!.number + 1;
    this.#file.releaseUsedSamples(id, releaseUntil);
    this.#releasedSamples = releaseUntil;
    this.#nextSampleNumber = releaseUntil;
    this.#deliveredSamples += converted.length;
    if (this.#timeToFirstSampleMs === null && converted.length > 0) {
      this.#timeToFirstSampleMs = this.#now() - this.#startedAt;
    }
    this.#observeMp4Memory();
    this.#callbacks.onSamples(converted);
  }

  async #takePull(): Promise<boolean> {
    while (!this.#stopped && !this.#pullRequested) {
      await new Promise<void>((resolve) => this.#pullWaiters.add(resolve));
    }
    if (this.#stopped) return false;
    this.#pullRequested = false;
    return true;
  }

  #allSamplesDelivered(): boolean {
    return (
      this.#totalSamples !== undefined &&
      this.#deliveredSamples >= this.#totalSamples
    );
  }

  #observeMp4Memory(rawBuffer?: ArrayBuffer, convertedBatchBytes = 0): void {
    const sampleBytes = this.#file.getAllocatedSampleDataSize?.() ?? 0;
    const retained = retainedMp4Buffers(this.#file);
    const retainedBytes = retained.bytes;
    if (sampleBytes > MAX_DEMUX_MP4_SAMPLE_BYTES) {
      throw new Error("MP4 sample retention exceeded its fixed byte limit");
    }
    const mp4BufferLimit =
      this.#activeReadKind === "metadata"
        ? MAX_DEMUX_METADATA_MP4_BUFFER_BYTES
        : MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES;
    if (retainedBytes > mp4BufferLimit) {
      throw new Error(
        "MP4 input buffer retention exceeded its fixed byte limit",
      );
    }
    this.#peakMp4SampleBytes = Math.max(this.#peakMp4SampleBytes, sampleBytes);
    this.#peakMp4BufferBytes = Math.max(
      this.#peakMp4BufferBytes,
      retainedBytes,
    );
    const rawBytes =
      rawBuffer && !retained.identities.has(rawBuffer)
        ? rawBuffer.byteLength
        : 0;
    const demuxRetainedBytes =
      retainedBytes + sampleBytes + convertedBatchBytes + rawBytes;
    if (demuxRetainedBytes > MAX_DEMUX_RETAINED_BYTES) {
      throw new Error("MP4 demux retention exceeded its fixed byte limit");
    }
    this.#peakDemuxRetainedBytes = Math.max(
      this.#peakDemuxRetainedBytes,
      demuxRetainedBytes,
    );
    if (this.#activeReadKind === "metadata") {
      this.#peakMetadataMp4BufferBytes = Math.max(
        this.#peakMetadataMp4BufferBytes,
        retainedBytes,
      );
    } else if (this.#activeReadKind === "media") {
      this.#peakMediaMp4BufferBytes = Math.max(
        this.#peakMediaMp4BufferBytes,
        retainedBytes,
      );
    }
  }

  #recordRead(kind: "metadata" | "media", bytes: number): void {
    if (kind === "metadata") {
      this.#metadataReadCount += 1;
      this.#metadataBytesRead += bytes;
      this.#peakMetadataReadBytes = Math.max(
        this.#peakMetadataReadBytes,
        bytes,
      );
    } else {
      this.#mediaReadCount += 1;
      this.#mediaBytesRead += bytes;
      this.#peakMediaReadBytes = Math.max(this.#peakMediaReadBytes, bytes);
    }
  }

  #boundedMetadataReadSize(requested: number): number {
    const remaining = MAX_DEMUX_METADATA_BYTES - this.#metadataBytesRead;
    if (remaining <= 0) {
      throw new Error(
        `MP4の動画情報が${MAX_DEMUX_METADATA_BYTES / (1024 * 1024)}MiBを超えています。通常のMP4へ再多重化するか、動画を再エンコードしてください。`,
      );
    }
    return Math.min(requested, remaining);
  }

  #nextMediaRange(): {
    readonly sampleIndex: number;
    readonly offset: number;
    readonly size: number;
  } | null {
    if (this.#trackId === undefined) return null;
    const track = this.#file.getTrackById(this.#trackId);
    const first = track?.samples[track.nextSample];
    if (!first) return null;
    assertEncodedSampleSize(first.size);
    const alreadyRead = first.alreadyRead ?? 0;
    const offset = first.offset + alreadyRead;
    let end = first.offset + first.size;
    let included = 1;
    if (alreadyRead === 0) {
      for (
        let index = track.nextSample + 1;
        index < track.samples.length && included < this.#extractionBatchSamples;
        index += 1
      ) {
        const sample = track.samples[index];
        assertEncodedSampleSize(sample.size);
        const sampleEnd = sample.offset + sample.size;
        if (
          sample.offset !== end ||
          sampleEnd - offset >
            Math.min(
              MAX_ENCODED_BATCH_BYTES,
              Math.max(this.#chunkBytes, first.size),
            )
        ) {
          break;
        }
        end = sampleEnd;
        included += 1;
      }
    }
    if (end <= offset) return null;
    return { sampleIndex: track.nextSample, offset, size: end - offset };
  }

  #wakePullWaiters(): void {
    for (const resolve of this.#pullWaiters) resolve();
    this.#pullWaiters.clear();
  }

  #reportError(error: unknown): void {
    if (this.#stopped || this.#failed) return;
    this.#failed = true;
    this.stop(error);
    this.#callbacks.onError(error);
  }
}

class SparseReadRanges {
  readonly #ranges: Array<{ start: number; end: number }> = [];
  #operations = 0;

  get rangeCount(): number {
    return this.#ranges.length;
  }

  get operations(): number {
    return this.#operations;
  }

  add(start: number, end: number): void {
    let next = { start, end };
    const merged: Array<{ start: number; end: number }> = [];
    let inserted = false;
    for (const range of this.#ranges) {
      this.#operations += 1;
      if (range.end < next.start) {
        merged.push(range);
      } else if (next.end < range.start) {
        if (!inserted) {
          merged.push(next);
          inserted = true;
        }
        merged.push(range);
      } else {
        next = {
          start: Math.min(next.start, range.start),
          end: Math.max(next.end, range.end),
        };
      }
    }
    if (!inserted) merged.push(next);
    this.#ranges.splice(0, this.#ranges.length, ...merged);
  }

  nextUnread(offset: number): number {
    let next = offset;
    for (const range of this.#ranges) {
      this.#operations += 1;
      if (next < range.start) break;
      if (next < range.end) next = range.end;
    }
    return next;
  }

  readableBytes(offset: number, maximum: number, sourceSize: number): number {
    let end = Math.min(sourceSize, offset + maximum);
    for (const range of this.#ranges) {
      this.#operations += 1;
      if (range.start > offset) {
        end = Math.min(end, range.start);
        break;
      }
    }
    return end - offset;
  }
}

function chooseNextOffset(
  fallback: number,
  suggested: number | undefined,
  sourceSize: number,
  ranges: SparseReadRanges,
): number {
  const candidate =
    typeof suggested === "number" &&
    Number.isSafeInteger(suggested) &&
    suggested >= 0 &&
    suggested < sourceSize
      ? suggested
      : fallback;
  return Math.min(sourceSize, ranges.nextUnread(candidate));
}

function assertConsecutiveSamples(
  samples: readonly Sample[],
  expectedFirst: number,
  trackId: number,
): void {
  for (let index = 0; index < samples.length; index += 1) {
    const sample = samples[index];
    if (
      sample.track_id !== trackId ||
      sample.number !== expectedFirst + index
    ) {
      throw new Error(
        `MP4 samples must be consecutive from ${expectedFirst}; received track ${sample.track_id} sample ${sample.number}`,
      );
    }
  }
}

function assertTrackMatchesPreflight(
  track: Movie["videoTracks"][number],
  validated: InspectedVideoTrack,
): void {
  if (
    track.id !== validated.trackId ||
    track.codec !== validated.codec ||
    track.video?.width !== validated.codedWidth ||
    track.video?.height !== validated.codedHeight ||
    track.nb_samples !== validated.totalSamples ||
    track.timescale !== validated.timescale ||
    track.duration !== validated.duration
  ) {
    throw new Error(
      "MP4の映像トラックが事前確認時と一致しません。動画を選択し直してください。",
    );
  }
}

function validateEncodedSamples(
  samples: readonly Sample[],
  validated: InspectedVideoTrack,
): number {
  if (samples.length !== validated.totalSamples) {
    throw new Error(
      "MP4のsample tableが事前確認時と一致しません。動画を選択し直してください。",
    );
  }
  let maximum = 0;
  for (const sample of samples) {
    assertEncodedSampleSize(sample.size);
    maximum = Math.max(maximum, sample.size);
  }
  if (maximum !== validated.maxSampleBytes) {
    throw new Error(
      "MP4の最大圧縮フレームサイズが事前確認時と一致しません。動画を選択し直してください。",
    );
  }
  return maximum;
}

function assertEncodedSampleSize(size: number): void {
  if (
    !Number.isSafeInteger(size) ||
    size <= 0 ||
    size > MAX_ENCODED_SAMPLE_BYTES
  ) {
    throw new Error(
      `圧縮フレームが${MAX_ENCODED_SAMPLE_BYTES / (1024 * 1024)}MiBを超えています。映像品質またはビットレートを下げてMP4を再エンコードしてください。`,
    );
  }
}

function toVideoSample(
  sample: Sample,
  createChunk: (sample: Sample) => EncodedVideoChunk,
): Mp4VideoSample {
  if (!sample.data) throw new Error("MP4 sample dataがありません");
  const timestampUs = mp4TimestampUs(sample.cts, sample.timescale);
  return {
    metadata: {
      isSync: sample.is_sync,
      timestampUs,
      offset: sample.offset,
      size: sample.size,
    },
    chunk: createChunk(sample),
  };
}

function encodedVideoChunk(sample: Sample): EncodedVideoChunk {
  if (!sample.data) throw new Error("MP4 sample dataがありません");
  return new EncodedVideoChunk({
    type: sample.is_sync ? "key" : "delta",
    timestamp: mp4TimestampUs(sample.cts, sample.timescale),
    duration: (sample.duration * 1_000_000) / sample.timescale,
    data: sample.data,
  });
}

interface ObservableMp4Stream {
  readonly buffers?: readonly ArrayBuffer[];
  readonly buffer?: ArrayBuffer;
  readonly _buffer?: ArrayBuffer;
}

function retainedMp4Buffers(file: ISOFile): {
  readonly bytes: number;
  readonly identities: ReadonlySet<ArrayBuffer>;
} {
  const streams = new Set<ObservableMp4Stream>();
  const buffers = new Set<ArrayBuffer>();
  if (file.stream) streams.add(file.stream as ObservableMp4Stream);
  for (const mdat of file.mdats ?? []) {
    const stream = (mdat as { stream?: ObservableMp4Stream }).stream;
    if (stream) streams.add(stream);
  }
  for (const stream of streams) {
    for (const buffer of stream.buffers ?? []) buffers.add(buffer);
    if (stream.buffer) buffers.add(stream.buffer);
    if (stream._buffer) buffers.add(stream._buffer);
  }
  let total = 0;
  for (const buffer of buffers) total += buffer.byteLength;
  return { bytes: total, identities: buffers };
}

function assertPositiveInteger(value: number, label: string): void {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw new Error(`${label} must be a positive safe integer`);
  }
}
