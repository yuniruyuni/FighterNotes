import { describe, expect, mock, test } from "bun:test";
import {
  createFile,
  type ISOFile,
  type Movie,
  type MP4BoxBuffer,
  type Sample,
} from "mp4box";
import {
  MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES,
  MAX_DEMUX_METADATA_BYTES,
  MAX_ENCODED_BATCH_SAMPLES,
  MAX_ENCODED_SAMPLE_BYTES,
} from "../../domain/encoded-video-limits.js";
import type { InspectedVideoTrack } from "../../domain/video-preflight.js";
import { BlobRangeReader } from "./blob-range-reader.js";
import { type Mp4VideoSample, Mp4VideoSource } from "./mp4-video-source.js";

describe("Mp4VideoSource", () => {
  test("rejects an extraction batch size above the fixed sample limit", () => {
    expect(
      () =>
        new Mp4VideoSource(
          new Blob([new Uint8Array(1)]),
          {
            onTrack: async () => undefined,
            onSamples: () => undefined,
            onError: () => undefined,
          },
          inspectedTrack(1),
          { extractionBatchSamples: MAX_ENCODED_BATCH_SAMPLES + 1 },
        ),
    ).toThrow(`must not exceed ${MAX_ENCODED_BATCH_SAMPLES}`);
  });

  test("rejects a parser batch above the effective extraction limit", async () => {
    const samples = Array.from(
      { length: MAX_ENCODED_BATCH_SAMPLES + 1 },
      (_, index) => sample(index, index, [index]),
    );
    const parser = new FakeSparseIsoFile(
      32,
      0,
      samples,
      [],
      samples.map((_, index) => 8 + index),
    );
    parser.forcedSamples = samples;
    let reportError!: (error: unknown) => void;
    const reported = new Promise<unknown>((resolve) => {
      reportError = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(32)]),
      {
        onTrack: async () => undefined,
        onSamples: () => undefined,
        onError: reportError,
      },
      inspectedTrack(samples.length),
      {
        file: parser.asIsoFile(),
        chunkBytes: 8,
        createChunk: copiedChunk,
      },
    );

    source.start();

    expect(String(await reported)).toContain("fixed sample batch limit");
    expect(parser.releaseCalls).toEqual([]);
  });

  test("waits for asynchronous decoder setup before reading or extracting", async () => {
    const calls: string[] = [];
    let resolveTrack!: () => void;
    const trackReady = new Promise<void>((resolve) => {
      resolveTrack = resolve;
    });
    const parser = new FakeSparseIsoFile(4, 0, [sample(0, 0, [1])], calls, [0]);
    let resolveSamples!: () => void;
    const samplesDone = new Promise<void>((resolve) => {
      resolveSamples = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(4)]),
      {
        onTrack: async () => {
          calls.push("track");
          await trackReady;
        },
        onSamples: () => resolveSamples(),
        onError: (error) => {
          throw error;
        },
      },
      inspectedTrack(1),
      {
        file: parser.asIsoFile(),
        chunkBytes: 4,
        createChunk: copiedChunk,
        now: () => 110,
      },
    );

    source.start(100);
    await Promise.resolve();
    expect(calls).toEqual(["track"]);
    expect(parser.readOffsets).toEqual([]);

    resolveTrack();
    await samplesDone;
    expect(calls.indexOf("track")).toBeLessThan(calls.indexOf("options"));
    expect(calls.indexOf("options")).toBeLessThan(calls.indexOf("start"));
    expect(parser.releaseCalls).toEqual([1]);
    expect(source.statistics.timeToFirstSampleMs).toBe(10);
    source.stop();
  });

  test("uses the same bounded sparse state machine for tail and head moov", async () => {
    const tail = await runSparseLayout(56);
    const head = await runSparseLayout(0);

    expect(tail.timestamps).toEqual([0, 1_000_000]);
    expect(head.timestamps).toEqual(tail.timestamps);
    expect(tail.offsets).toEqual([0, 56, 8, 16]);
    expect(head.offsets).toEqual([0, 8, 16]);
    expect(new Set(tail.offsets).size).toBe(tail.offsets.length);
    expect(new Set(head.offsets).size).toBe(head.offsets.length);
    expect(tail.lastFlags).toEqual([false, false, false, false]);
    expect(head.lastFlags).toEqual([false, false, false]);
    expect(tail.releaseCalls).toEqual([1, 2]);
    expect(head.releaseCalls).toEqual([1, 2]);
    expect(tail.stats.peakReadBytes).toBeLessThanOrEqual(8);
    expect(head.stats.peakReadBytes).toBeLessThanOrEqual(8);
  });

  test("does not release MP4Box samples when chunk construction fails", async () => {
    const parser = new FakeSparseIsoFile(8, 0, [sample(0, 0, [1])], [], [0]);
    const failure = new Error("chunk constructor failed");
    let rejectUnexpected!: (error: unknown) => void;
    const reported = new Promise<unknown>((resolve) => {
      rejectUnexpected = resolve;
    });
    const onSamples = mock(() => undefined);
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(8)]),
      {
        onTrack: async () => undefined,
        onSamples,
        onError: rejectUnexpected,
      },
      inspectedTrack(1),
      {
        file: parser.asIsoFile(),
        chunkBytes: 8,
        createChunk: () => {
          throw failure;
        },
      },
    );

    source.start();
    expect(await reported).toBe(failure);
    expect(parser.releaseCalls).toEqual([]);
    expect(onSamples).not.toHaveBeenCalled();
  });

  test("constructed chunks remain independent after MP4Box releases sample data", async () => {
    const original = sample(0, 0, [4, 5, 6]);
    const parser = new FakeSparseIsoFile(8, 0, [original], [], [0]);
    let resolveChunk!: (chunk: EncodedVideoChunk) => void;
    const received = new Promise<EncodedVideoChunk>((resolve) => {
      resolveChunk = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(8)]),
      {
        onTrack: async () => undefined,
        onSamples: (samples) => resolveChunk(samples[0].chunk),
        onError: (error) => {
          throw error;
        },
      },
      inspectedTrack(1, { maxSampleBytes: 3 }),
      {
        file: parser.asIsoFile(),
        chunkBytes: 8,
        createChunk: copiedChunk,
      },
    );

    source.start();
    const chunk = (await received) as EncodedVideoChunk & {
      readonly copiedBytes: Uint8Array;
    };
    expect(original.data).toBeUndefined();
    expect(chunk.copiedBytes).toEqual(new Uint8Array([4, 5, 6]));
    source.stop();
  });

  test("fails immediately when the downstream sample consumer throws", async () => {
    const parser = new FakeSparseIsoFile(
      16,
      0,
      [sample(0, 0, [1]), sample(1, 1000, [2])],
      [],
      [0, 8],
    );
    const failure = new Error("decode admission failed");
    let reportError!: (error: unknown) => void;
    const reported = new Promise<unknown>((resolve) => {
      reportError = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(16)]),
      {
        onTrack: async () => undefined,
        onSamples: () => {
          throw failure;
        },
        onError: reportError,
      },
      inspectedTrack(2),
      {
        file: parser.asIsoFile(),
        chunkBytes: 8,
        createChunk: copiedChunk,
      },
    );

    source.start();
    expect(await reported).toBe(failure);
    expect(parser.releaseCalls).toEqual([1]);
    expect(parser.readOffsets).toEqual([0]);
    expect(source.statistics.deliveredSamples).toBe(1);
  });

  test("propagates MP4Box parser errors and stops before extraction", async () => {
    const parser = new FakeSparseIsoFile(8, 0, [sample(0, 0, [1])]);
    parser.parserError = "invalid box";
    let reportError!: (error: unknown) => void;
    const reported = new Promise<unknown>((resolve) => {
      reportError = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(8)]),
      {
        onTrack: async () => undefined,
        onSamples: () => undefined,
        onError: reportError,
      },
      inspectedTrack(1),
      { file: parser.asIsoFile(), chunkBytes: 8 },
    );

    source.start();
    expect(String(await reported)).toContain("ISOFile: invalid box");
    expect(parser.readOffsets).toEqual([0]);
    expect(parser.startCalls).toBe(0);
    expect(parser.releaseCalls).toEqual([]);
  });

  test("rejects an oversized encoded sample before MP4Box extraction", async () => {
    const oversized = sample(0, 0, [1]);
    oversized.size = MAX_ENCODED_SAMPLE_BYTES + 1;
    const parser = new FakeSparseIsoFile(8, 0, [oversized], [], [0]);
    let reportError!: (error: unknown) => void;
    const reported = new Promise<unknown>((resolve) => {
      reportError = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(8)]),
      {
        onTrack: async () => undefined,
        onSamples: () => undefined,
        onError: reportError,
      },
      inspectedTrack(1, {
        maxSampleBytes: MAX_ENCODED_SAMPLE_BYTES + 1,
      }),
      { file: parser.asIsoFile(), chunkBytes: 8 },
    );

    source.start();

    expect(String(await reported)).toContain("再エンコード");
    expect(parser.startCalls).toBe(0);
    expect(parser.releaseCalls).toEqual([]);
    expect(source.statistics.mediaReadCount).toBe(0);
  });

  test("rejects metadata reads at the fixed 32 MiB budget", async () => {
    const sourceBlob = new SyntheticSparseBlob(MAX_DEMUX_METADATA_BYTES + 1);
    const parser = new FakeSparseIsoFile(
      sourceBlob.size,
      Number.MAX_SAFE_INTEGER,
      [sample(0, 0, [1])],
    );
    const reader = new BlobRangeReader(sourceBlob, undefined, {
      readSlice: async (slice) => new ArrayBuffer(slice.size),
    });
    let reportError!: (error: unknown) => void;
    const reported = new Promise<unknown>((resolve) => {
      reportError = resolve;
    });
    const source = new Mp4VideoSource(
      sourceBlob,
      {
        onTrack: async () => undefined,
        onSamples: () => undefined,
        onError: reportError,
      },
      inspectedTrack(1),
      { file: parser.asIsoFile(), reader, createChunk: copiedChunk },
    );

    source.start();

    expect(String(await reported)).toContain("動画情報");
    expect(source.statistics.metadataBytesRead).toBe(MAX_DEMUX_METADATA_BYTES);
    expect(source.statistics.metadataReadCount).toBe(32);
  });

  test("rejects observable MP4 buffers above the fixed retention limit", async () => {
    const parser = new FakeSparseIsoFile(8, 0, [sample(0, 0, [1])], [], [0]);
    parser.stream.buffer = {
      byteLength: MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES + 1,
    } as ArrayBuffer;
    let reportError!: (error: unknown) => void;
    const reported = new Promise<unknown>((resolve) => {
      reportError = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(8)]),
      {
        onTrack: async () => undefined,
        onSamples: () => undefined,
        onError: reportError,
      },
      inspectedTrack(1),
      { file: parser.asIsoFile(), chunkBytes: 8, createChunk: copiedChunk },
    );

    source.start();

    expect(String(await reported)).toContain("fixed byte limit");
    expect(parser.releaseCalls).toEqual([]);
  });

  test("does not append a slice whose read finishes after abort", async () => {
    const controller = new AbortController();
    let finishRead!: (buffer: ArrayBuffer) => void;
    let readStarted!: () => void;
    const started = new Promise<void>((resolve) => {
      readStarted = resolve;
    });
    const reader = new BlobRangeReader(
      new Blob([new Uint8Array(8)]),
      controller.signal,
      {
        readSlice: async () => {
          readStarted();
          return new Promise<ArrayBuffer>((resolve) => {
            finishRead = resolve;
          });
        },
      },
    );
    const parser = new FakeSparseIsoFile(8, 0, [sample(0, 0, [1])], [], [0]);
    let reportError!: (error: unknown) => void;
    const reported = new Promise<unknown>((resolve) => {
      reportError = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(8)]),
      {
        onTrack: async () => undefined,
        onSamples: () => undefined,
        onError: reportError,
      },
      inspectedTrack(1),
      { file: parser.asIsoFile(), reader, chunkBytes: 8 },
    );
    const reason = new Error("cancelled during slice read");

    source.start();
    await started;
    controller.abort(reason);
    finishRead(new ArrayBuffer(8));

    expect(await reported).toBe(reason);
    expect(parser.readOffsets).toEqual([]);
    expect(parser.startCalls).toBe(0);
  });

  test("keeps small reads above 4 GiB bounded for a synthetic 8 GiB source", async () => {
    const totalSamples = 64;
    const chunkBytes = 1024;
    const sourceBlob = new SyntheticSparseBlob(8 * 1024 ** 3);
    const samples = Array.from({ length: totalSamples }, (_, number) =>
      sample(number, number * 1000, [number & 0xff]),
    );
    const offsets = samples.map(
      (_, index) => 4 * 1024 ** 3 + (index + 1) * chunkBytes,
    );
    const parser = new FakeSparseIsoFile(
      sourceBlob.size,
      0,
      samples,
      [],
      offsets,
    );
    let source!: Mp4VideoSource;
    let resolveDone!: () => void;
    const done = new Promise<void>((resolve) => {
      resolveDone = resolve;
    });
    source = new Mp4VideoSource(
      sourceBlob,
      {
        onTrack: async () => undefined,
        onSamples: (batch) => {
          if (source.statistics.deliveredSamples >= totalSamples) resolveDone();
          else source.pull();
          expect(batch.length).toBeLessThanOrEqual(1);
        },
        onError: (error) => {
          throw error;
        },
      },
      inspectedTrack(totalSamples),
      {
        file: parser.asIsoFile(),
        chunkBytes,
        createChunk: copiedChunk,
      },
    );

    source.start();
    await done;
    expect(source.statistics).toMatchObject({
      deliveredSamples: totalSamples,
      releasedSamples: totalSamples,
      peakBatchSamples: 1,
      peakBatchBytes: 1,
      peakReadBytes: chunkBytes,
    });
    expect(source.statistics.peakMp4BufferBytes).toBeLessThanOrEqual(
      chunkBytes,
    );
    expect(source.statistics.peakMp4SampleBytes).toBeLessThanOrEqual(
      totalSamples,
    );
    // Peak retention is the scalability contract. This synthetic sparse file
    // also skips metadata-like gaps, so its total read count is only an
    // auxiliary assertion; real analysis must still read every encoded byte.
    expect(source.statistics.totalBytesRead).toBeLessThan(1024 * 1024);
    expect(source.statistics.totalBytesRead).toBeLessThan(
      sourceBlob.size / 1000,
    );
    expect(offsets[0]).toBeGreaterThan(2 ** 32);
    source.stop();
  });

  test("keeps sparse metadata range work linear across 10,000 gapped samples", async () => {
    const totalSamples = 10_000;
    const sampleGap = 1024;
    const sourceBlob = new SyntheticSparseBlob((totalSamples + 1) * sampleGap);
    const samples = Array.from({ length: totalSamples }, (_, number) =>
      sample(number, number, [number & 0xff]),
    );
    const offsets = samples.map((_, index) => (index + 1) * sampleGap);
    const parser = new FakeSparseIsoFile(
      sourceBlob.size,
      0,
      samples,
      [],
      offsets,
    );
    const reader = new BlobRangeReader(sourceBlob, undefined, {
      readSlice: async (slice) => new ArrayBuffer(slice.size),
    });
    let source!: Mp4VideoSource;
    let resolveDone!: () => void;
    const done = new Promise<void>((resolve) => {
      resolveDone = resolve;
    });
    source = new Mp4VideoSource(
      sourceBlob,
      {
        onTrack: async () => undefined,
        onSamples: () => {
          if (source.statistics.deliveredSamples === totalSamples)
            resolveDone();
          else source.pull();
        },
        onError: (error) => {
          throw error;
        },
      },
      inspectedTrack(totalSamples),
      {
        file: parser.asIsoFile(),
        reader,
        chunkBytes: 64,
        extractionBatchSamples: 1,
        createChunk: copiedChunk,
      },
    );

    source.start();
    await done;

    expect(source.statistics.metadataSparseRangeCount).toBe(1);
    expect(source.statistics.metadataSparseRangeOperations).toBeLessThan(
      totalSamples * 3,
    );
    expect(source.statistics.readCount).toBe(totalSamples + 1);
    expect(source.statistics.deliveredSamples).toBe(totalSamples);
    source.stop();
  });

  test("reads exact video samples without retaining audio-like gaps", async () => {
    const parser = new FakeSparseIsoFile(
      96,
      0,
      [sample(0, 0, [1]), sample(1, 1000, [2])],
      [],
      [16, 64],
    );
    let source!: Mp4VideoSource;
    let delivered = 0;
    let resolveDone!: () => void;
    const done = new Promise<void>((resolve) => {
      resolveDone = resolve;
    });
    source = new Mp4VideoSource(
      new Blob([new Uint8Array(96)]),
      {
        onTrack: async () => undefined,
        onSamples: (samples) => {
          delivered += samples.length;
          if (delivered === 2) resolveDone();
          else source.pull();
        },
        onError: (error) => {
          throw error;
        },
      },
      inspectedTrack(2),
      {
        file: parser.asIsoFile(),
        chunkBytes: 8,
        extractionBatchSamples: 1,
        createChunk: copiedChunk,
      },
    );

    source.start();
    await done;

    expect(parser.readRanges).toEqual([
      { offset: 0, size: 8 },
      { offset: 16, size: 1 },
      { offset: 64, size: 1 },
    ]);
    expect(source.statistics).toMatchObject({
      metadataBytesRead: 8,
      mediaBytesRead: 2,
      peakMediaReadBytes: 1,
      peakMediaMp4BufferBytes: 1,
    });
    expect(parser.exactReleasedBufferClears).toBe(2);
    expect(parser.stream.buffers).toHaveLength(0);
    source.stop();
  });

  test("counts the MP4 stream current buffer after the buffer list is cleaned", async () => {
    const parser = new FakeSparseIsoFile(8, 0, [sample(0, 0, [1])], [], [0]);
    parser.stream.buffer = new ArrayBuffer(1024 * 1024);
    parser.clearBuffersBeforeCallbacks = true;
    let resolveDone!: () => void;
    const done = new Promise<void>((resolve) => {
      resolveDone = resolve;
    });
    const source = new Mp4VideoSource(
      new Blob([new Uint8Array(8)]),
      {
        onTrack: async () => undefined,
        onSamples: () => resolveDone(),
        onError: (error) => {
          throw error;
        },
      },
      inspectedTrack(1),
      {
        file: parser.asIsoFile(),
        chunkBytes: 8,
        createChunk: copiedChunk,
      },
    );

    source.start();
    await done;

    expect(parser.stream.buffers).toHaveLength(0);
    expect(source.statistics.peakMp4BufferBytes).toBe(1024 * 1024);
    expect(source.statistics.peakDemuxRetainedBytes).toBeGreaterThan(
      source.statistics.peakMp4BufferBytes,
    );
    source.stop();
  });

  test("uses zero-based sample numbers and exclusive release with real MP4Box", async () => {
    const blob = generatedMp4();
    const parser = createFile();
    const released: number[] = [];
    const sampleNumbers: number[] = [];
    const release = parser.releaseUsedSamples.bind(parser);
    parser.releaseUsedSamples = (id, sampleNumber) => {
      released.push(sampleNumber);
      release(id, sampleNumber);
    };
    let resolveSamples!: (samples: readonly Mp4VideoSample[]) => void;
    const received = new Promise<readonly Mp4VideoSample[]>((resolve) => {
      resolveSamples = resolve;
    });
    const source = new Mp4VideoSource(
      blob,
      {
        onTrack: async () => undefined,
        onSamples: resolveSamples,
        onError: (error) => {
          throw error;
        },
      },
      inspectedTrack(2, {
        codec: "avc1",
        codedWidth: 16,
        codedHeight: 16,
        timescale: 1000,
        duration: 2,
        maxSampleBytes: 3,
      }),
      {
        file: parser,
        chunkBytes: blob.size,
        extractionBatchSamples: 2,
        createChunk: (value) => {
          sampleNumbers.push(value.number);
          return copiedChunk(value);
        },
      },
    );

    source.start();
    const samples = await received;
    expect(samples.map(({ metadata }) => metadata.timestampUs)).toEqual([
      0, 1000,
    ]);
    expect(sampleNumbers).toEqual([0, 1]);
    expect(released).toEqual([2]);
    expect(parser.getTrackSamplesInfo(1).map((value) => value.data)).toEqual([
      undefined,
      undefined,
    ]);
    source.stop();
  });

  test("re-reads an early sample discarded before a tail moov is parsed", async () => {
    const chunkBytes = 36;
    const blob = generatedTailMoovMp4();
    const parser = createFile();
    const appendedOffsets: number[] = [];
    const append = parser.appendBuffer.bind(parser);
    parser.appendBuffer = (buffer, last) => {
      appendedOffsets.push(buffer.fileStart);
      return append(buffer, last);
    };
    let resolveSamples!: (samples: readonly Mp4VideoSample[]) => void;
    let rejectSamples!: (error: unknown) => void;
    const received = new Promise<readonly Mp4VideoSample[]>(
      (resolve, reject) => {
        resolveSamples = resolve;
        rejectSamples = reject;
      },
    );
    const source = new Mp4VideoSource(
      blob,
      {
        onTrack: async () => undefined,
        onSamples: resolveSamples,
        onError: rejectSamples,
      },
      inspectedTrack(2, {
        codec: "avc1.42001e",
        codedWidth: 16,
        codedHeight: 16,
        timescale: 1000,
        duration: 2,
        maxSampleBytes: 3,
      }),
      {
        file: parser,
        chunkBytes,
        extractionBatchSamples: 2,
        createChunk: copiedChunk,
      },
    );

    source.start();
    const samples = await received;
    const firstSampleOffset = parser.getTrackSamplesInfo(1)[0].offset;

    expect(samples).toHaveLength(2);
    expect(
      samples.map(
        ({ chunk }) =>
          (chunk as EncodedVideoChunk & { copiedBytes: Uint8Array })
            .copiedBytes,
      ),
    ).toEqual([Uint8Array.of(1, 2, 3), Uint8Array.of(4, 5)]);
    expect(firstSampleOffset).toBeLessThan(chunkBytes);
    expect(appendedOffsets[0]).toBe(0);
    expect(appendedOffsets).toContain(firstSampleOffset);
    expect(source.statistics.mediaReadCount).toBeGreaterThan(0);
    expect(source.statistics.releasedSamples).toBe(2);
    source.stop();
  });
});

async function runSparseLayout(moovOffset: number) {
  const parser = new FakeSparseIsoFile(64, moovOffset, [
    sample(0, 0, [1]),
    sample(1, 1000, [2]),
  ]);
  const timestamps: number[] = [];
  let source!: Mp4VideoSource;
  let resolveDone!: () => void;
  const done = new Promise<void>((resolve) => {
    resolveDone = resolve;
  });
  source = new Mp4VideoSource(
    new Blob([new Uint8Array(64)]),
    {
      onTrack: async () => undefined,
      onSamples: (samples) => {
        timestamps.push(...samples.map(({ metadata }) => metadata.timestampUs));
        if (timestamps.length === 2) resolveDone();
        else source.pull();
      },
      onError: (error) => {
        throw error;
      },
    },
    inspectedTrack(2),
    {
      file: parser.asIsoFile(),
      chunkBytes: 8,
      createChunk: copiedChunk,
    },
  );
  source.start();
  await done;
  const stats = source.statistics;
  source.stop();
  return {
    timestamps,
    offsets: parser.readOffsets,
    lastFlags: parser.lastFlags,
    releaseCalls: parser.releaseCalls,
    stats,
  };
}

class FakeSparseIsoFile {
  onReady: ((info: Movie) => void) | undefined;
  onSamples:
    | ((id: number, user: unknown, samples: Sample[]) => void)
    | undefined;
  onError: ((module: string, message: string) => void) | undefined;
  nextSeekPosition: number | undefined;
  readonly stream: {
    buffers: ArrayBuffer[];
    buffer?: ArrayBuffer;
  } = { buffers: [] };
  readonly mdats: never[] = [];
  readonly readOffsets: number[] = [];
  readonly readRanges: Array<{ offset: number; size: number }> = [];
  readonly lastFlags: boolean[] = [];
  readonly releaseCalls: number[] = [];
  readonly #loadedOffsets = new Set<number>();
  readonly #samples: Sample[];
  readonly #sampleOffsets: number[];
  readonly #movie: Movie;
  readonly #moovOffset: number;
  readonly #calls: string[];
  #ready = false;
  #processing = false;
  #nextSample = 0;
  #lastReleased = 0;
  startCalls = 0;
  exactReleasedBufferClears = 0;
  parserError: string | undefined;
  clearBuffersBeforeCallbacks = false;
  forcedSamples: Sample[] | undefined;

  constructor(
    fileSize: number,
    moovOffset: number,
    samples: Sample[],
    calls: string[] = [],
    sampleOffsets = samples.map((_, index) => (index + 1) * 8),
  ) {
    this.#moovOffset = moovOffset;
    this.#samples = samples;
    this.#sampleOffsets = sampleOffsets;
    for (let index = 0; index < samples.length; index += 1) {
      samples[index].offset = sampleOffsets[index];
    }
    this.#calls = calls;
    this.#movie = movie(samples.length, fileSize);
  }

  asIsoFile(): ISOFile {
    return this as unknown as ISOFile;
  }

  appendBuffer(buffer: MP4BoxBuffer, last = false): number {
    this.readOffsets.push(buffer.fileStart);
    this.readRanges.push({
      offset: buffer.fileStart,
      size: buffer.byteLength,
    });
    this.lastFlags.push(last);
    this.stream.buffers.splice(0, this.stream.buffers.length, buffer);
    this.#loadedOffsets.add(buffer.fileStart);
    if (this.clearBuffersBeforeCallbacks) this.stream.buffers.splice(0);
    if (this.parserError) {
      this.onError?.("ISOFile", this.parserError);
      return buffer.fileStart + buffer.byteLength;
    }
    if (!this.#ready && buffer.fileStart === this.#moovOffset) {
      this.#ready = true;
      this.onReady?.(this.#movie);
    }
    if (this.#processing) this.#emitAvailableSample();
    if (!this.#ready) return this.#moovOffset;
    const suggested = this.nextSeekPosition;
    this.nextSeekPosition = undefined;
    return suggested ?? buffer.fileStart + buffer.byteLength;
  }

  setExtractionOptions(): void {
    this.#calls.push("options");
  }

  start(): void {
    this.startCalls += 1;
    this.#calls.push("start");
    this.#processing = true;
    this.#emitAvailableSample();
  }

  stop(): void {
    this.#processing = false;
  }

  flush(): void {
    if (this.#processing) this.#emitAvailableSample();
  }

  releaseUsedSamples(_id: number, sampleNumber: number): void {
    this.releaseCalls.push(sampleNumber);
    const current = this.stream.buffers[0] as MP4BoxBuffer | undefined;
    const firstReleased = this.#samples[this.#lastReleased];
    const releasedBytes = this.#samples
      .slice(this.#lastReleased, sampleNumber)
      .reduce((total, value) => total + value.size, 0);
    if (
      current &&
      firstReleased &&
      current.fileStart === firstReleased.offset &&
      current.byteLength === releasedBytes
    ) {
      this.stream.buffers.splice(0);
      this.exactReleasedBufferClears += 1;
    }
    for (let index = this.#lastReleased; index < sampleNumber; index += 1) {
      const data = this.#samples[index]?.data;
      data?.fill(0);
      if (this.#samples[index]) this.#samples[index].data = undefined;
    }
    this.#lastReleased = sampleNumber;
  }

  getAllocatedSampleDataSize(): number {
    return this.#samples.reduce(
      (total, value) => total + (value.data?.byteLength ?? 0),
      0,
    );
  }

  getTrackSamplesInfo(): Sample[] {
    return this.#samples;
  }

  getTrackById(): { readonly nextSample: number; readonly samples: Sample[] } {
    return { nextSample: this.#nextSample, samples: this.#samples };
  }

  #emitAvailableSample(): void {
    const offset = this.#sampleOffsets[this.#nextSample];
    if (offset === undefined) return;
    if (!this.#loadedOffsets.has(offset)) {
      this.nextSeekPosition = offset;
      return;
    }
    const value = this.#samples[this.#nextSample];
    const emitted = this.forcedSamples ?? [value];
    this.forcedSamples = undefined;
    this.#nextSample += emitted.length;
    this.onSamples?.(1, undefined, emitted);
  }
}

class SyntheticSparseBlob {
  readonly size: number;

  constructor(size: number) {
    this.size = size;
  }

  slice(start = 0, end = this.size): Blob {
    return new Blob([new Uint8Array(end - start)]);
  }
}

function inspectedTrack(
  totalSamples: number,
  overrides: Partial<InspectedVideoTrack> = {},
): InspectedVideoTrack {
  return {
    trackId: 1,
    codec: "avc1.42c028",
    codedWidth: 1920,
    codedHeight: 1080,
    displayWidth: 1920,
    displayHeight: 1080,
    rotation: 0,
    framesPerSecond: 60,
    constantFrameRate: true,
    totalSamples,
    maxSampleBytes: 1,
    timescale: 1000,
    duration: totalSamples,
    decoderConfig: {
      codec: "avc1.42c028",
      codedWidth: 1920,
      codedHeight: 1080,
    },
    codecConfig: {
      codec: "avc1.42c028",
      width: 1920,
      height: 1080,
    },
    ...overrides,
  };
}

function movie(totalSamples: number, _fileSize: number): Movie {
  return {
    videoTracks: [
      {
        id: 1,
        nb_samples: totalSamples,
        codec: "avc1.42c028",
        video: { width: 1920, height: 1080 },
        timescale: 1000,
        duration: totalSamples,
      },
    ],
  } as unknown as Movie;
}

function sample(number: number, cts: number, bytes: number[]): Sample {
  return {
    number,
    track_id: 1,
    timescale: 1000,
    cts,
    dts: cts,
    duration: 1,
    size: bytes.length,
    offset: (number + 1) * 8,
    is_sync: number === 0,
    data: new Uint8Array(bytes),
  } as Sample;
}

function copiedChunk(value: Sample): EncodedVideoChunk {
  const copiedBytes = new Uint8Array(value.data!);
  return {
    byteLength: copiedBytes.byteLength,
    copiedBytes,
  } as unknown as EncodedVideoChunk;
}

function generatedMp4(): Blob {
  const file = createFile();
  file.init({ timescale: 1000, duration: 2 });
  const trackId = file.addTrack({
    type: "avc1",
    width: 16,
    height: 16,
    timescale: 1000,
    media_duration: 2,
    duration: 2,
  });
  file.addSample(trackId, new Uint8Array([1, 2, 3]), {
    duration: 1,
    cts: 0,
    dts: 0,
    is_sync: true,
  });
  file.addSample(trackId, new Uint8Array([4, 5]), {
    duration: 1,
    cts: 1,
    dts: 1,
    is_sync: false,
  });
  return file.save("generated.mp4");
}

function generatedTailMoovMp4(): Blob {
  const ftyp = mp4Box(
    "ftyp",
    ascii("isom"),
    u32(0),
    ascii("isom"),
    ascii("avc1"),
  );
  const sampleBytes = Uint8Array.of(1, 2, 3, 4, 5);
  const mdat = mp4Box("mdat", sampleBytes);
  const sampleOffset = ftyp.byteLength + 8;
  const matrix = uint32s([
    0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000,
  ]);
  const mvhd = fullMp4Box(
    "mvhd",
    0,
    0,
    u32(0),
    u32(0),
    u32(1000),
    u32(2),
    u32(0x0001_0000),
    u16(0x0100),
    zeroes(10),
    matrix,
    zeroes(24),
    u32(2),
  );
  const tkhd = fullMp4Box(
    "tkhd",
    0,
    7,
    u32(0),
    u32(0),
    u32(1),
    u32(0),
    u32(2),
    zeroes(8),
    u16(0),
    u16(0),
    u16(0),
    u16(0),
    matrix,
    u32(16 << 16),
    u32(16 << 16),
  );
  const mdhd = fullMp4Box(
    "mdhd",
    0,
    0,
    u32(0),
    u32(0),
    u32(1000),
    u32(2),
    u16(0x55c4),
    u16(0),
  );
  const hdlr = fullMp4Box(
    "hdlr",
    0,
    0,
    u32(0),
    ascii("vide"),
    zeroes(12),
    ascii("VideoHandler\0"),
  );
  const vmhd = fullMp4Box("vmhd", 0, 1, u16(0), u16(0), u16(0), u16(0));
  const url = fullMp4Box("url ", 0, 1);
  const dref = fullMp4Box("dref", 0, 0, u32(1), url);
  const dinf = mp4Box("dinf", dref);
  const avcC = mp4Box("avcC", Uint8Array.of(1, 0x42, 0, 0x1e, 0xff, 0xe0, 0));
  const avc1 = mp4Box(
    "avc1",
    zeroes(6),
    u16(1),
    u16(0),
    u16(0),
    zeroes(12),
    u16(16),
    u16(16),
    u32(0x0048_0000),
    u32(0x0048_0000),
    u32(0),
    u16(1),
    zeroes(32),
    u16(0x0018),
    u16(0xffff),
    avcC,
  );
  const stsd = fullMp4Box("stsd", 0, 0, u32(1), avc1);
  const stts = fullMp4Box("stts", 0, 0, u32(1), u32(2), u32(1));
  const stsc = fullMp4Box("stsc", 0, 0, u32(1), u32(1), u32(2), u32(1));
  const stsz = fullMp4Box("stsz", 0, 0, u32(0), u32(2), u32(3), u32(2));
  const stco = fullMp4Box("stco", 0, 0, u32(1), u32(sampleOffset));
  const stss = fullMp4Box("stss", 0, 0, u32(1), u32(1));
  const stbl = mp4Box("stbl", stsd, stts, stsc, stsz, stco, stss);
  const minf = mp4Box("minf", vmhd, dinf, stbl);
  const mdia = mp4Box("mdia", mdhd, hdlr, minf);
  const trak = mp4Box("trak", tkhd, mdia);
  const moov = mp4Box("moov", mvhd, trak);
  return new Blob([ftyp, mdat, moov]);
}

type Bytes = Uint8Array<ArrayBuffer>;

function mp4Box(type: string, ...payload: Bytes[]): Bytes {
  const content = concatBytes(...payload);
  return concatBytes(u32(content.byteLength + 8), ascii(type), content);
}

function fullMp4Box(
  type: string,
  version: number,
  flags: number,
  ...payload: Bytes[]
): Bytes {
  return mp4Box(
    type,
    Uint8Array.of(
      version,
      (flags >>> 16) & 0xff,
      (flags >>> 8) & 0xff,
      flags & 0xff,
    ),
    ...payload,
  );
}

function concatBytes(...parts: Bytes[]): Bytes {
  const result = new Uint8Array(
    parts.reduce((total, part) => total + part.byteLength, 0),
  );
  let offset = 0;
  for (const part of parts) {
    result.set(part, offset);
    offset += part.byteLength;
  }
  return result;
}

function ascii(value: string): Bytes {
  return Uint8Array.from(value, (character) => character.charCodeAt(0));
}

function u16(value: number): Bytes {
  const bytes = new Uint8Array(2);
  new DataView(bytes.buffer).setUint16(0, value);
  return bytes;
}

function u32(value: number): Bytes {
  const bytes = new Uint8Array(4);
  new DataView(bytes.buffer).setUint32(0, value);
  return bytes;
}

function uint32s(values: readonly number[]): Bytes {
  return concatBytes(...values.map(u32));
}

function zeroes(size: number): Bytes {
  return new Uint8Array(size);
}
