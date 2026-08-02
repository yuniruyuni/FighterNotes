import { afterEach, beforeEach, describe, expect, test } from "bun:test";
import type { FrameSample } from "../../domain/result.js";
import { decodeSampleRange } from "./webcodecs-frame-decoder.js";

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
      async onFrame(frame) {
        await Promise.resolve();
        frame.close();
      },
    });

    expect(FakeVideoDecoder.instances).toHaveLength(1);
    expect(FakeVideoDecoder.instances[0].peakQueueSize).toBeLessThanOrEqual(12);
    expect(stats).toEqual({
      peakDecoderQueueSize: 12,
      peakDecoderOutstandingFrames: 12,
    });
    expect(FakeVideoFrame.instances).toHaveLength(frameCount);
    expect(FakeVideoFrame.instances.every((frame) => frame.closed)).toBeTrue();
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
      onFrame: async (frame, processingSignal) => {
        started.resolve();
        try {
          await rejectOnAbort(processingSignal);
        } finally {
          frame.close();
        }
      },
      signal: controller.signal,
    });
    await started.promise;

    controller.abort(reason);

    await expect(decoding).rejects.toBe(reason);
    expect(FakeVideoDecoder.instances[0].closeCount).toBe(1);
    expect(FakeVideoFrame.instances.length).toBeGreaterThan(0);
    expect(FakeVideoFrame.instances.every((frame) => frame.closed)).toBeTrue();
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
      onFrame: async (frame, processingSignal) => {
        try {
          await rejectOnAbort(processingSignal);
        } finally {
          frame.close();
        }
      },
    });

    await expect(decoding).rejects.toBe(reason);
    expect(FakeVideoFrame.instances.length).toBeGreaterThan(0);
    expect(FakeVideoFrame.instances.every((frame) => frame.closed)).toBeTrue();
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
  closed = false;

  constructor(timestamp: number) {
    this.timestamp = timestamp;
    FakeVideoFrame.instances.push(this);
  }

  close(): void {
    this.closed = true;
  }
}

class FakeVideoDecoder extends EventTarget {
  static instances: FakeVideoDecoder[] = [];
  static errorAtTimestamp: number | null = null;
  static decoderError: DOMException = new DOMException(
    "synthetic decoder failure",
    "EncodingError",
  );

  readonly #output: (frame: VideoFrame) => void;
  readonly #error: (error: DOMException) => void;
  readonly #queue: FakeEncodedVideoChunk[] = [];
  readonly #flushWaiters: Array<ReturnType<typeof deferred<void>>> = [];
  state: CodecState = "unconfigured";
  decodeQueueSize = 0;
  peakQueueSize = 0;
  closeCount = 0;
  #scheduled = false;

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
    this.#queue.push(chunk as unknown as FakeEncodedVideoChunk);
    this.decodeQueueSize = this.#queue.length;
    this.peakQueueSize = Math.max(this.peakQueueSize, this.decodeQueueSize);
    this.#schedule();
  }

  flush(): Promise<void> {
    if (this.state === "closed") {
      return Promise.reject(new DOMException("decoder closed", "AbortError"));
    }
    if (this.#queue.length === 0) return Promise.resolve();
    const waiter = deferred<void>();
    this.#flushWaiters.push(waiter);
    return waiter.promise;
  }

  close(): void {
    if (this.state === "closed") return;
    this.state = "closed";
    this.closeCount += 1;
    this.#queue.length = 0;
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
    const chunk = this.#queue.shift();
    if (!chunk) return;
    this.decodeQueueSize = this.#queue.length;
    this.dispatchEvent(new Event("dequeue"));
    if (chunk.timestamp === FakeVideoDecoder.errorAtTimestamp) {
      this.state = "closed";
      const reason = FakeVideoDecoder.decoderError;
      for (const waiter of this.#flushWaiters.splice(0)) waiter.reject(reason);
      this.#error(reason);
      return;
    }
    this.#output(new FakeVideoFrame(chunk.timestamp) as unknown as VideoFrame);
    if (this.#queue.length > 0) this.#schedule();
    else for (const waiter of this.#flushWaiters.splice(0)) waiter.resolve();
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
