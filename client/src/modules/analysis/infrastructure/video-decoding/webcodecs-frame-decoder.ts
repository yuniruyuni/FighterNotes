import type { FrameSample, VideoCodecConfig } from "../../domain/result.js";
import type { BlobSliceSource } from "./blob-range-reader.js";
import {
  BlobSampleReader,
  type BlobSampleReaderStats,
} from "./blob-sample-reader.js";
import { FrameDecodePlan } from "./frame-decode-plan.js";

export interface DecodeBackpressureOptions {
  readonly queueHighWatermark: number;
  readonly queueLowWatermark: number;
  readonly outstandingHighWatermark: number;
  readonly outstandingLowWatermark: number;
}

export interface DecodeSampleRangeStats {
  readonly peakDecoderQueueSize: number;
  readonly peakDecoderOutstandingFrames: number;
  readonly sampleReads: BlobSampleReaderStats;
}

interface DecodeSampleRangeOptions {
  readonly samples: readonly FrameSample[];
  readonly videoBlob: BlobSliceSource;
  readonly codecConfig: VideoCodecConfig;
  readonly firstSampleIndex: number;
  readonly lastSampleIndex: number;
  readonly onFrame: (
    frame: VideoFrame,
    processingSignal: AbortSignal,
  ) => void | Promise<void>;
  /**
   * Runs synchronously from the decoder output callback. Frames rejected here
   * are closed immediately without consuming retained-frame capacity.
   */
  readonly shouldProcessFrame?: (frame: VideoFrame) => boolean;
  readonly signal?: AbortSignal;
  readonly backpressure?: DecodeBackpressureOptions;
  /** Test seam for a controlled Blob read. */
  readonly sampleReader?: BlobSampleReader;
}

interface FlowWaiter {
  readonly resolve: () => void;
  readonly signal: AbortSignal;
  readonly onAbort: () => void;
}

const DEFAULT_BACKPRESSURE: DecodeBackpressureOptions = {
  queueHighWatermark: 12,
  queueLowWatermark: 6,
  outstandingHighWatermark: 12,
  outstandingLowWatermark: 6,
};

export async function decodeSampleRange(
  options: DecodeSampleRangeOptions,
): Promise<DecodeSampleRangeStats> {
  throwIfAborted(options.signal);
  const limits = options.backpressure ?? DEFAULT_BACKPRESSURE;
  validateWatermarks(limits);
  const sampleReader =
    options.sampleReader ??
    new BlobSampleReader(options.videoBlob, options.signal);

  let retryFrames: readonly OutputFrameIdentity[] | undefined;
  let peakDecoderQueueSize = 0;
  let peakDecoderOutstandingFrames = 0;
  try {
    do {
      const pass = await decodeSampleRangePass(
        options,
        limits,
        retryFrames,
        sampleReader,
      );
      peakDecoderQueueSize = Math.max(
        peakDecoderQueueSize,
        pass.peakDecoderQueueSize,
      );
      peakDecoderOutstandingFrames = Math.max(
        peakDecoderOutstandingFrames,
        pass.peakDecoderOutstandingFrames,
      );
      if (pass.unmatchedRetryFrames > 0) {
        throw new Error(
          `再デコード対象のVideoFrameを${pass.unmatchedRetryFrames}件再取得できませんでした`,
        );
      }
      retryFrames = pass.deferredFrames;
    } while (retryFrames.length > 0);
  } finally {
    sampleReader.stop();
  }
  throwIfAborted(options.signal);

  return {
    peakDecoderQueueSize,
    peakDecoderOutstandingFrames,
    sampleReads: sampleReader.statistics,
  };
}

interface DecodePassResult {
  readonly peakDecoderQueueSize: number;
  readonly peakDecoderOutstandingFrames: number;
  readonly deferredFrames: readonly OutputFrameIdentity[];
  readonly unmatchedRetryFrames: number;
}

interface OutputFrameIdentity {
  readonly timestamp: number;
  readonly timestampOrdinal: number;
}

/**
 * Decodes one pass through the sample range.
 *
 * decodeQueueSize bounds encoded input admission. Actual VideoFrame objects
 * are counted only after output and are owned by this function: onFrame may
 * borrow them until its promise settles, after which they are closed exactly
 * once. A decoder may legally withhold output until later input or flush. If a
 * synchronous output burst would exceed the retained-frame watermark, the
 * suffix is closed immediately and replayed in presentation order by a later
 * pass instead of retaining an unbounded number of VideoFrames.
 */
async function decodeSampleRangePass(
  options: DecodeSampleRangeOptions,
  limits: DecodeBackpressureOptions,
  retryFrames: readonly OutputFrameIdentity[] | undefined,
  sampleReader: BlobSampleReader,
): Promise<DecodePassResult> {
  throwIfAborted(options.signal);

  const processing = new AbortController();
  const waiters: FlowWaiter[] = [];
  const selector = new OutputFrameSelector(
    options.shouldProcessFrame,
    retryFrames,
  );
  const deferredFrames: OutputFrameIdentity[] = [];
  let failure: { readonly reason: unknown } | undefined;
  let decoder!: VideoDecoder;
  let frameChain = Promise.resolve();
  let outstandingFrames = 0;
  let peakOutstandingFrames = 0;
  let peakQueueSize = 0;
  let queuePaused = false;
  let outstandingPaused = false;
  let deferRemainingOutput = false;

  const wake = () => {
    for (const waiter of waiters.splice(0)) {
      waiter.signal.removeEventListener("abort", waiter.onAbort);
      waiter.resolve();
    }
  };
  const closeDecoder = () => {
    if (!decoder?.state || decoder.state === "closed") return;
    try {
      decoder.close();
    } catch (error) {
      if (!failure) {
        failure = { reason: error };
        processing.abort(error);
        wake();
      }
    }
  };
  const fail = (reason: unknown) => {
    if (failure) return;
    failure = { reason };
    sampleReader.stop(reason);
    processing.abort(reason);
    wake();
    closeDecoder();
  };
  const onAbort = () => fail(abortReason(options.signal));

  decoder = new VideoDecoder({
    output(frame) {
      let selected: OutputFrameIdentity | undefined;
      try {
        selected = selector.select(frame);
      } catch (error) {
        safeClose(frame);
        fail(error);
        return;
      }
      if (failure || !selected) {
        safeClose(frame);
        return;
      }
      if (
        deferRemainingOutput ||
        outstandingFrames >= limits.outstandingHighWatermark
      ) {
        deferRemainingOutput = true;
        deferredFrames.push(selected);
        safeClose(frame);
        return;
      }
      outstandingFrames += 1;
      peakOutstandingFrames = Math.max(
        peakOutstandingFrames,
        outstandingFrames,
      );
      frameChain = frameChain.then(async () => {
        try {
          if (!failure) await options.onFrame(frame, processing.signal);
        } catch (error) {
          fail(error);
        } finally {
          safeClose(frame);
          outstandingFrames -= 1;
          wake();
        }
      });
    },
    error: fail,
  });
  decoder.ondequeue = wake;
  options.signal?.addEventListener("abort", onAbort, { once: true });

  try {
    decoder.configure({
      codec: options.codecConfig.codec,
      codedWidth: options.codecConfig.width,
      codedHeight: options.codecConfig.height,
      description: options.codecConfig.description,
    });
    for (
      let sampleIndex = options.firstSampleIndex;
      sampleIndex <= options.lastSampleIndex;
      sampleIndex += 1
    ) {
      const admission = waitForAdmission();
      if (admission) await admission;
      const sample = options.samples[sampleIndex];
      if (!sample) continue;
      const sampleRead = sampleReader.readSample(options.samples, sampleIndex);
      const sampleBytes =
        sampleRead instanceof Uint8Array ? sampleRead : await sampleRead;
      throwFailure(failure);
      throwIfAborted(options.signal);
      const chunk = encodedChunk(sample, sampleBytes);
      throwFailure(failure);
      throwIfAborted(options.signal);
      decoder.decode(chunk);
      peakQueueSize = Math.max(peakQueueSize, decoder.decodeQueueSize);
    }
    await decoder.flush();
    await frameChain;
    throwFailure(failure);
    throwIfAborted(options.signal);
  } catch (error) {
    fail(preferredFailure(error, options.signal, failure));
  } finally {
    if (decoder.ondequeue === wake) decoder.ondequeue = null;
    closeDecoder();
    processing.abort(failure?.reason ?? new Error("decode range completed"));
    wake();
    await frameChain;
    options.signal?.removeEventListener("abort", onAbort);
  }

  throwFailure(failure);
  throwIfAborted(options.signal);
  return {
    peakDecoderQueueSize: peakQueueSize,
    peakDecoderOutstandingFrames: peakOutstandingFrames,
    deferredFrames,
    unmatchedRetryFrames: selector.unmatchedRetryFrames,
  };

  function waitForAdmission(): Promise<void> | undefined {
    throwFailure(failure);
    throwIfAborted(options.signal);
    queuePaused = pausedAtWatermarks(
      queuePaused,
      decoder.decodeQueueSize,
      limits.queueHighWatermark,
      limits.queueLowWatermark,
    );
    outstandingPaused = deferRemainingOutput
      ? false
      : pausedAtWatermarks(
          outstandingPaused,
          outstandingFrames + decoder.decodeQueueSize,
          limits.outstandingHighWatermark,
          limits.outstandingLowWatermark,
        );
    if (!queuePaused && !outstandingPaused) return undefined;
    return waitForChange(processing.signal, waiters).then(() =>
      waitForAdmission(),
    );
  }
}

class OutputFrameSelector {
  readonly #shouldProcessFrame: DecodeSampleRangeOptions["shouldProcessFrame"];
  readonly #retryOrdinals: Map<number, Set<number>> | undefined;
  readonly #seenOrdinals = new Map<number, number>();
  #unmatchedRetryFrames: number;

  constructor(
    shouldProcessFrame: DecodeSampleRangeOptions["shouldProcessFrame"],
    retryFrames: readonly OutputFrameIdentity[] | undefined,
  ) {
    this.#shouldProcessFrame = shouldProcessFrame;
    this.#unmatchedRetryFrames = retryFrames?.length ?? 0;
    if (!retryFrames) return;
    this.#retryOrdinals = new Map();
    for (const frame of retryFrames) {
      const ordinals = this.#retryOrdinals.get(frame.timestamp) ?? new Set();
      ordinals.add(frame.timestampOrdinal);
      this.#retryOrdinals.set(frame.timestamp, ordinals);
    }
  }

  get unmatchedRetryFrames(): number {
    return this.#unmatchedRetryFrames;
  }

  select(frame: VideoFrame): OutputFrameIdentity | undefined {
    if (this.#shouldProcessFrame && !this.#shouldProcessFrame(frame)) {
      return undefined;
    }
    const timestampOrdinal = this.#seenOrdinals.get(frame.timestamp) ?? 0;
    this.#seenOrdinals.set(frame.timestamp, timestampOrdinal + 1);
    const identity = { timestamp: frame.timestamp, timestampOrdinal };
    if (!this.#retryOrdinals) return identity;
    const ordinals = this.#retryOrdinals.get(frame.timestamp);
    if (!ordinals?.delete(timestampOrdinal)) return undefined;
    this.#unmatchedRetryFrames -= 1;
    return identity;
  }
}

export async function decodeFrameAt(options: {
  readonly samples: readonly FrameSample[];
  readonly videoBlob: BlobSliceSource;
  readonly codecConfig: VideoCodecConfig;
  readonly frameToSampleIndex: readonly number[] | null;
  readonly frameIndex: number;
  readonly signal?: AbortSignal;
}): Promise<VideoFrame | null> {
  const plan = FrameDecodePlan.create(
    options.samples,
    options.frameToSampleIndex,
    options.frameIndex,
  );
  if (!plan) return null;

  const result: { frame: VideoFrame | null } = { frame: null };
  try {
    await decodeSampleRange({
      ...options,
      firstSampleIndex: plan.firstSampleIndex,
      lastSampleIndex: plan.lastSampleIndex,
      shouldProcessFrame: (frame) => frame.timestamp === plan.targetTimestampUs,
      onFrame(frame) {
        if (!result.frame) result.frame = frame.clone();
      },
    });
    throwIfAborted(options.signal);
  } catch (error) {
    result.frame?.close();
    throw error;
  }
  return result.frame;
}

function encodedChunk(
  sample: FrameSample,
  sampleBytes: Uint8Array<ArrayBuffer>,
): EncodedVideoChunk {
  return new EncodedVideoChunk({
    type: sample.isSync ? "key" : "delta",
    timestamp: sample.timestampUs,
    data: sampleBytes,
  });
}

function pausedAtWatermarks(
  paused: boolean,
  value: number,
  high: number,
  low: number,
): boolean {
  if (paused) return value > low;
  return value >= high;
}

function waitForChange(
  signal: AbortSignal,
  waiters: FlowWaiter[],
): Promise<void> {
  if (signal.aborted) return Promise.reject(abortReason(signal));
  return new Promise<void>((resolve, reject) => {
    const waiter: FlowWaiter = {
      resolve,
      signal,
      onAbort: () => {
        const index = waiters.indexOf(waiter);
        if (index >= 0) waiters.splice(index, 1);
        reject(abortReason(signal));
      },
    };
    waiters.push(waiter);
    signal.addEventListener("abort", waiter.onAbort, { once: true });
    if (signal.aborted) waiter.onAbort();
  });
}

function validateWatermarks(options: DecodeBackpressureOptions): void {
  validatePair(
    options.queueHighWatermark,
    options.queueLowWatermark,
    "decoder queue",
  );
  validatePair(
    options.outstandingHighWatermark,
    options.outstandingLowWatermark,
    "decoder outstanding frame",
  );
}

function validatePair(high: number, low: number, label: string): void {
  if (!Number.isInteger(high) || high <= 0) {
    throw new Error(`${label} high watermark must be a positive integer`);
  }
  if (!Number.isInteger(low) || low < 0 || low >= high) {
    throw new Error(`${label} low watermark must be an integer below high`);
  }
}

function safeClose(frame: VideoFrame): void {
  try {
    frame.close();
  } catch {
    // Preserve the first decoder or consumer failure.
  }
}

function preferredFailure(
  error: unknown,
  signal: AbortSignal | undefined,
  failure: { readonly reason: unknown } | undefined,
): unknown {
  if (failure) return failure.reason;
  if (signal?.aborted) return abortReason(signal);
  return error;
}

function throwFailure(
  failure: { readonly reason: unknown } | undefined,
): asserts failure is undefined {
  if (failure) throw failure.reason;
}

function abortReason(signal?: AbortSignal): unknown {
  return signal?.reason instanceof Error
    ? signal.reason
    : new Error("動画のデコードを中断しました");
}

function throwIfAborted(signal?: AbortSignal): void {
  if (signal?.aborted) throw abortReason(signal);
}
