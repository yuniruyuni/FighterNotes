import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import type { FrameSample } from "../../domain/result.js";
import { decodeFrameAt, decodeSampleRange } from "./webcodecs-frame-decoder.js";

const globals = new Map<
  "VideoDecoder" | "EncodedVideoChunk",
  PropertyDescriptor | undefined
>();

beforeEach(() => {
  globals.set(
    "VideoDecoder",
    Object.getOwnPropertyDescriptor(globalThis, "VideoDecoder"),
  );
  globals.set(
    "EncodedVideoChunk",
    Object.getOwnPropertyDescriptor(globalThis, "EncodedVideoChunk"),
  );
  Object.defineProperty(globalThis, "VideoDecoder", {
    configurable: true,
    writable: true,
    value: FakeVideoDecoder,
  });
  Object.defineProperty(globalThis, "EncodedVideoChunk", {
    configurable: true,
    writable: true,
    value: FakeEncodedVideoChunk,
  });
  FakeVideoDecoder.instances = [];
  FakeVideoDecoder.errorAtTimestamp = null;
  FakeVideoDecoder.flushError = null;
  FakeVideoDecoder.omitTimestampOnReplay = null;
  FakeVideoDecoder.outputMode = "normal";
  FakeVideoFrame.instances = [];
});

afterEach(() => {
  restoreGlobal("VideoDecoder");
  restoreGlobal("EncodedVideoChunk");
});

describe("decodeSampleRange backpressure", () => {
  test("bounds decoder queue and outstanding frames across 1,001 samples", async () => {
    const frameCount = 1_001;
    const stats = await decodeSampleRange({
      samples: samples(frameCount),
      videoArrayBuffer: new ArrayBuffer(0),
      codecConfig: { codec: "fake", width: 1920, height: 1080 },
      firstSampleIndex: 0,
      lastSampleIndex: frameCount - 1,
      backpressure: {
        queueHighWatermark: 12,
        queueLowWatermark: 6,
        outstandingHighWatermark: 12,
        outstandingLowWatermark: 6,
      },
      async onFrame(_frame) {
        await Promise.resolve();
      },
    });

    expect(FakeVideoDecoder.instances).toHaveLength(1);
    expect(FakeVideoDecoder.instances[0].decodeCount).toBe(frameCount);
    expect(FakeVideoDecoder.instances[0].peakQueueSize).toBeLessThanOrEqual(12);
    expect(stats.peakDecoderQueueSize).toBe(12);
    expect(stats.peakDecoderOutstandingFrames).toBeGreaterThan(0);
    expect(stats.peakDecoderOutstandingFrames).toBeLessThanOrEqual(12);
    expect(FakeVideoFrame.instances).toHaveLength(frameCount);
    expect(
      FakeVideoFrame.instances.every((frame) => frame.closeCount === 1),
    ).toBeTrue();
  });

  test("keeps one decode pass while a normal-output consumer is stalled", async () => {
    const frameCount = 100;
    const consumerStarted = deferred<void>();
    const releaseConsumer = deferred<void>();
    const seen: number[] = [];
    const decoding = decodeSampleRange({
      samples: samples(frameCount),
      videoArrayBuffer: new ArrayBuffer(0),
      codecConfig: { codec: "fake", width: 1920, height: 1080 },
      firstSampleIndex: 0,
      lastSampleIndex: frameCount - 1,
      backpressure: {
        queueHighWatermark: 12,
        queueLowWatermark: 6,
        outstandingHighWatermark: 12,
        outstandingLowWatermark: 6,
      },
      async onFrame(frame) {
        seen.push(frame.timestamp);
        consumerStarted.resolve();
        await releaseConsumer.promise;
      },
    });

    await consumerStarted.promise;
    await Promise.resolve();
    expect(FakeVideoDecoder.instances).toHaveLength(1);
    expect(FakeVideoDecoder.instances[0].decodeCount).toBeLessThan(frameCount);
    expect(
      FakeVideoFrame.instances.filter((frame) => frame.closeCount === 0).length,
    ).toBeLessThanOrEqual(12);

    releaseConsumer.resolve();
    const stats = await decoding;

    expect(FakeVideoDecoder.instances).toHaveLength(1);
    expect(FakeVideoDecoder.instances[0].decodeCount).toBe(frameCount);
    expect(seen).toEqual(
      samples(frameCount).map((sample) => sample.timestampUs),
    );
    expect(stats.peakDecoderOutstandingFrames).toBeLessThanOrEqual(12);
    expect(
      FakeVideoFrame.instances.every((frame) => frame.closeCount === 1),
    ).toBeTrue();
  });

  test("reaches flush when output is delayed and replays a bounded suffix in order", async () => {
    FakeVideoDecoder.outputMode = "flush";
    const input = samples(29).map((sample, index) => ({
      ...sample,
      timestampUs: Math.floor(index / 2),
    }));
    const seen: number[] = [];
    const seenSourceIndexes: number[] = [];

    const stats = await decodeSampleRange({
      samples: input,
      videoArrayBuffer: new ArrayBuffer(0),
      codecConfig: { codec: "fake", width: 1920, height: 1080 },
      firstSampleIndex: 0,
      lastSampleIndex: input.length - 1,
      backpressure: {
        queueHighWatermark: 12,
        queueLowWatermark: 6,
        outstandingHighWatermark: 11,
        outstandingLowWatermark: 5,
      },
      onFrame(frame) {
        seen.push(frame.timestamp);
        seenSourceIndexes.push(
          (frame as unknown as FakeVideoFrame).sourceIndex,
        );
      },
    });

    expect(seen).toEqual(input.map((sample) => sample.timestampUs));
    expect(seenSourceIndexes).toEqual(input.map((_, index) => index));
    expect(FakeVideoDecoder.instances).toHaveLength(3);
    expect(
      FakeVideoDecoder.instances.every((decoder) => decoder.flushCount === 1),
    ).toBeTrue();
    expect(stats.peakDecoderOutstandingFrames).toBe(11);
    expect(
      FakeVideoFrame.instances.every((frame) => frame.closeCount === 1),
    ).toBeTrue();
  });

  test("feeds every chunk and flushes when the decoder produces no output", async () => {
    FakeVideoDecoder.outputMode = "none";
    let received = 0;

    const stats = await decodeSampleRange({
      samples: samples(25),
      videoArrayBuffer: new ArrayBuffer(0),
      codecConfig: { codec: "fake", width: 1920, height: 1080 },
      firstSampleIndex: 0,
      lastSampleIndex: 24,
      onFrame() {
        received += 1;
      },
    });

    expect(FakeVideoDecoder.instances).toHaveLength(1);
    expect(FakeVideoDecoder.instances[0].decodeCount).toBe(25);
    expect(FakeVideoDecoder.instances[0].flushCount).toBe(1);
    expect(received).toBe(0);
    expect(stats.peakDecoderOutstandingFrames).toBe(0);
  });

  test("fails rather than silently losing an overflow frame missing from replay", async () => {
    FakeVideoDecoder.outputMode = "flush";
    FakeVideoDecoder.omitTimestampOnReplay = 12;

    const decoding = decodeSampleRange({
      samples: samples(13),
      videoArrayBuffer: new ArrayBuffer(0),
      codecConfig: { codec: "fake", width: 1920, height: 1080 },
      firstSampleIndex: 0,
      lastSampleIndex: 12,
      onFrame() {},
    });

    await expect(decoding).rejects.toThrow(
      "再デコード対象のVideoFrameを1件再取得できませんでした",
    );
    expect(FakeVideoDecoder.instances).toHaveLength(2);
    expect(
      FakeVideoFrame.instances.every((frame) => frame.closeCount === 1),
    ).toBeTrue();
  });

  test("abort closes the decoder and every delivered VideoFrame", async () => {
    const controller = new AbortController();
    const reason = new Error("cancel decode range");
    const started = deferred<void>();
    const decoding = decodeSampleRange({
      samples: samples(100),
      videoArrayBuffer: new ArrayBuffer(0),
      codecConfig: { codec: "fake", width: 1920, height: 1080 },
      firstSampleIndex: 0,
      lastSampleIndex: 99,
      onFrame: async (_frame, processingSignal) => {
        started.resolve();
        await rejectOnAbort(processingSignal);
      },
      signal: controller.signal,
    });
    await started.promise;

    controller.abort(reason);

    await expect(decoding).rejects.toBe(reason);
    expect(FakeVideoDecoder.instances[0].closeCount).toBe(1);
    expect(FakeVideoFrame.instances.length).toBeGreaterThan(0);
    expect(
      FakeVideoFrame.instances.every((frame) => frame.closeCount === 1),
    ).toBeTrue();
  });

  test("decoder errors reject flow waiters and close delivered VideoFrames", async () => {
    const reason = new DOMException(
      "synthetic decoder failure",
      "EncodingError",
    );
    FakeVideoDecoder.errorAtTimestamp = 5;
    FakeVideoDecoder.decoderError = reason;

    const decoding = decodeSampleRange({
      samples: samples(100),
      videoArrayBuffer: new ArrayBuffer(0),
      codecConfig: { codec: "fake", width: 1920, height: 1080 },
      firstSampleIndex: 0,
      lastSampleIndex: 99,
      onFrame: async (_frame, processingSignal) => {
        await rejectOnAbort(processingSignal);
      },
    });

    await expect(decoding).rejects.toBe(reason);
    expect(FakeVideoFrame.instances.length).toBeGreaterThan(0);
    expect(
      FakeVideoFrame.instances.every((frame) => frame.closeCount === 1),
    ).toBeTrue();
  });

  test("propagates flush rejection after closing delivered VideoFrames", async () => {
    const reason = new DOMException("synthetic flush failure", "EncodingError");
    FakeVideoDecoder.flushError = reason;

    await expect(
      decodeSampleRange({
        samples: samples(4),
        videoArrayBuffer: new ArrayBuffer(0),
        codecConfig: { codec: "fake", width: 1920, height: 1080 },
        firstSampleIndex: 0,
        lastSampleIndex: 3,
        onFrame() {},
      }),
    ).rejects.toBe(reason);
    expect(FakeVideoDecoder.instances[0].closeCount).toBe(1);
    expect(
      FakeVideoFrame.instances.every((frame) => frame.closeCount === 1),
    ).toBeTrue();
  });

  test("decodeFrameAt returns a clone while closing every decoder-owned frame", async () => {
    const result = await decodeFrameAt({
      samples: samples(3),
      videoArrayBuffer: new ArrayBuffer(0),
      codecConfig: { codec: "fake", width: 1920, height: 1080 },
      frameToSampleIndex: [0, 1, 2],
      frameIndex: 2,
    });

    expect(result).not.toBeNull();
    const retained = result as unknown as FakeVideoFrame;
    expect(retained.timestamp).toBe(2);
    expect(retained.closeCount).toBe(0);
    expect(
      FakeVideoFrame.instances
        .filter((frame) => frame !== retained)
        .every((frame) => frame.closeCount === 1),
    ).toBeTrue();

    result?.close();
    expect(retained.closeCount).toBe(1);
  });
});

function samples(count: number): FrameSample[] {
  return Array.from({ length: count }, (_, timestampUs) => ({
    isSync: timestampUs === 0,
    timestampUs,
    offset: 0,
    size: 0,
  }));
}

function rejectOnAbort(signal: AbortSignal): Promise<never> {
  if (signal.aborted) return Promise.reject(signal.reason);
  return new Promise<never>((_, reject) => {
    signal.addEventListener("abort", () => reject(signal.reason), {
      once: true,
    });
  });
}

function restoreGlobal(key: "VideoDecoder" | "EncodedVideoChunk"): void {
  const descriptor = globals.get(key);
  if (descriptor) Object.defineProperty(globalThis, key, descriptor);
  else Reflect.deleteProperty(globalThis, key);
}

class FakeEncodedVideoChunk {
  readonly timestamp: number;

  constructor(init: EncodedVideoChunkInit) {
    this.timestamp = init.timestamp;
  }
}

class FakeVideoFrame {
  static instances: FakeVideoFrame[] = [];
  readonly timestamp: number;
  readonly sourceIndex: number;
  closeCount = 0;

  constructor(timestamp: number, sourceIndex: number) {
    this.timestamp = timestamp;
    this.sourceIndex = sourceIndex;
    FakeVideoFrame.instances.push(this);
  }

  close(): void {
    this.closeCount += 1;
  }

  clone(): FakeVideoFrame {
    return new FakeVideoFrame(this.timestamp, this.sourceIndex);
  }
}

interface QueuedChunk {
  readonly chunk: FakeEncodedVideoChunk;
  readonly sourceIndex: number;
}

class FakeVideoDecoder extends EventTarget {
  static instances: FakeVideoDecoder[] = [];
  static errorAtTimestamp: number | null = null;
  static flushError: DOMException | null = null;
  static omitTimestampOnReplay: number | null = null;
  static outputMode: "normal" | "flush" | "none" = "normal";
  static decoderError: DOMException = new DOMException(
    "synthetic decoder failure",
    "EncodingError",
  );

  readonly #output: (frame: VideoFrame) => void;
  readonly #error: (error: DOMException) => void;
  readonly #queue: QueuedChunk[] = [];
  readonly #pendingOutputs: QueuedChunk[] = [];
  readonly #flushWaiters: Array<ReturnType<typeof deferred<void>>> = [];
  state: CodecState = "unconfigured";
  decodeQueueSize = 0;
  peakQueueSize = 0;
  closeCount = 0;
  decodeCount = 0;
  flushCount = 0;
  #scheduled = false;
  #flushRequested = false;

  constructor(init: VideoDecoderInit) {
    super();
    this.#output = init.output;
    this.#error = init.error;
    FakeVideoDecoder.instances.push(this);
  }

  configure(): void {
    this.state = "configured";
  }

  decode(chunk: EncodedVideoChunk): void {
    if (this.state !== "configured") throw new DOMException("closed");
    this.#queue.push({
      chunk: chunk as unknown as FakeEncodedVideoChunk,
      sourceIndex: this.decodeCount,
    });
    this.decodeCount += 1;
    this.decodeQueueSize = this.#queue.length;
    this.peakQueueSize = Math.max(this.peakQueueSize, this.decodeQueueSize);
    this.#schedule();
  }

  flush(): Promise<void> {
    this.flushCount += 1;
    if (this.state === "closed") {
      return Promise.reject(new DOMException("decoder closed", "AbortError"));
    }
    if (FakeVideoDecoder.flushError) {
      return Promise.reject(FakeVideoDecoder.flushError);
    }
    this.#flushRequested = true;
    const waiter = deferred<void>();
    this.#flushWaiters.push(waiter);
    if (this.#queue.length === 0) this.#completeFlush();
    return waiter.promise;
  }

  close(): void {
    if (this.state === "closed") return;
    this.state = "closed";
    this.closeCount += 1;
    this.#queue.length = 0;
    this.#pendingOutputs.length = 0;
    this.decodeQueueSize = 0;
    const reason = new DOMException("decoder closed", "AbortError");
    for (const waiter of this.#flushWaiters.splice(0)) waiter.reject(reason);
  }

  #schedule(): void {
    if (this.#scheduled) return;
    this.#scheduled = true;
    queueMicrotask(() => this.#drainOne());
  }

  #drainOne(): void {
    this.#scheduled = false;
    if (this.state !== "configured") return;
    const queued = this.#queue.shift();
    if (!queued) return;
    this.decodeQueueSize = this.#queue.length;
    this.dispatchEvent(new Event("dequeue"));
    if (queued.chunk.timestamp === FakeVideoDecoder.errorAtTimestamp) {
      this.state = "closed";
      const reason = FakeVideoDecoder.decoderError;
      for (const waiter of this.#flushWaiters.splice(0)) waiter.reject(reason);
      this.#error(reason);
      return;
    }
    if (FakeVideoDecoder.outputMode === "normal") this.#emit(queued);
    else if (FakeVideoDecoder.outputMode === "flush") {
      this.#pendingOutputs.push(queued);
    }
    if (this.#queue.length > 0) this.#schedule();
    else this.#completeFlush();
  }

  #emit(queued: QueuedChunk): void {
    if (
      FakeVideoDecoder.instances.indexOf(this) > 0 &&
      queued.chunk.timestamp === FakeVideoDecoder.omitTimestampOnReplay
    ) {
      return;
    }
    this.#output(
      new FakeVideoFrame(
        queued.chunk.timestamp,
        queued.sourceIndex,
      ) as unknown as VideoFrame,
    );
  }

  #completeFlush(): void {
    if (!this.#flushRequested) return;
    this.#flushRequested = false;
    if (FakeVideoDecoder.outputMode === "flush") {
      for (const queued of this.#pendingOutputs.splice(0)) this.#emit(queued);
    } else {
      this.#pendingOutputs.length = 0;
    }
    for (const waiter of this.#flushWaiters.splice(0)) waiter.resolve();
  }
}

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  void promise.catch(() => {});
  return { promise, resolve, reject };
}
