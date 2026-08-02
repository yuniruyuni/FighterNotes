import type { FrameSample, VideoCodecConfig } from "../../domain/result.js";
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
}

interface DecodeSampleRangeOptions {
  readonly samples: readonly FrameSample[];
  readonly videoArrayBuffer: ArrayBuffer;
  readonly codecConfig: VideoCodecConfig;
  readonly firstSampleIndex: number;
  readonly lastSampleIndex: number;
  readonly onFrame: (
    frame: VideoFrame,
    processingSignal: AbortSignal,
  ) => void | Promise<void>;
  readonly signal?: AbortSignal;
  readonly backpressure?: DecodeBackpressureOptions;
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

  const processing = new AbortController();
  const waiters: FlowWaiter[] = [];
  let failure: { readonly reason: unknown } | undefined;
  let decoder!: VideoDecoder;
  let frameChain = Promise.resolve();
  let outstandingFrames = 0;
  let peakOutstandingFrames = 0;
  let peakQueueSize = 0;
  let queuePaused = false;
  let outstandingPaused = false;

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
    processing.abort(reason);
    wake();
    closeDecoder();
  };
  const onAbort = () => fail(abortReason(options.signal));

  decoder = new VideoDecoder({
    output(frame) {
      frameChain = frameChain.then(async () => {
        try {
          if (failure) {
            safeClose(frame);
            return;
          }
          await options.onFrame(frame, processing.signal);
        } catch (error) {
          safeClose(frame);
          fail(error);
        } finally {
          outstandingFrames = Math.max(0, outstandingFrames - 1);
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
      outstandingFrames += 1;
      peakOutstandingFrames = Math.max(
        peakOutstandingFrames,
        outstandingFrames,
      );
      try {
        decoder.decode(encodedChunk(sample, options.videoArrayBuffer));
        peakQueueSize = Math.max(peakQueueSize, decoder.decodeQueueSize);
      } catch (error) {
        outstandingFrames -= 1;
        throw error;
      }
    }
    await decoder.flush();
    await frameChain;
    throwFailure(failure);
    throwIfAborted(options.signal);
  } catch (error) {
    fail(preferredFailure(error, options.signal, failure));
  } finally {
    options.signal?.removeEventListener("abort", onAbort);
    if (decoder.ondequeue === wake) decoder.ondequeue = null;
    closeDecoder();
    processing.abort(failure?.reason ?? new Error("decode range completed"));
    wake();
    await frameChain;
  }

  throwFailure(failure);
  return {
    peakDecoderQueueSize: peakQueueSize,
    peakDecoderOutstandingFrames: peakOutstandingFrames,
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
    outstandingPaused = pausedAtWatermarks(
      outstandingPaused,
      outstandingFrames,
      limits.outstandingHighWatermark,
      limits.outstandingLowWatermark,
    );
    if (!queuePaused && !outstandingPaused) return undefined;
    return waitForChange(processing.signal, waiters).then(() =>
      waitForAdmission(),
    );
  }
}

export async function decodeFrameAt(options: {
  readonly samples: readonly FrameSample[];
  readonly videoArrayBuffer: ArrayBuffer;
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
      onFrame(frame) {
        if (!result.frame && frame.timestamp === plan.targetTimestampUs) {
          result.frame = frame;
        } else {
          frame.close();
        }
      },
    });
  } catch (error) {
    result.frame?.close();
    throw error;
  }
  return result.frame;
}

function encodedChunk(
  sample: FrameSample,
  videoArrayBuffer: ArrayBuffer,
): EncodedVideoChunk {
  return new EncodedVideoChunk({
    type: sample.isSync ? "key" : "delta",
    timestamp: sample.timestampUs,
    data: new Uint8Array(videoArrayBuffer, sample.offset, sample.size),
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
