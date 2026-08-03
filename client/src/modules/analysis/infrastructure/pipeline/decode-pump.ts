export interface DecodeQueue<T> {
  readonly state: string;
  readonly decodeQueueSize: number;
  decode(sample: T): void;
}

interface DecodePumpOptions {
  readonly maxDecodeQueue: number;
  readonly maxOutstandingFrames: number;
  readonly maxQueuedSamples: number;
  readonly queuedSampleLowWatermark: number;
  readonly maxQueuedBytes: number;
  readonly queuedByteLowWatermark: number;
  readonly onQueueLow: () => void;
  readonly onReadyToFlush: () => void;
  readonly onError: (error: unknown) => void;
}

export interface DecodePumpStats {
  readonly maxQueuedSamples: number;
  readonly queuedSampleLowWatermark: number;
  readonly maxQueuedBytes: number;
  readonly queuedByteLowWatermark: number;
  readonly peakQueuedSamples: number;
  readonly peakQueuedBytes: number;
}

interface QueuedSample<T> {
  readonly value: T;
  readonly byteLength: number;
}

/** Owns sample admission and the one-shot transition into decoder flushing. */
export class DecodePump<T> {
  readonly #options: DecodePumpOptions;
  readonly #queue: QueuedSample<T>[] = [];
  #totalSamples: number | null = null;
  #samplesFed = 0;
  #queuedBytes = 0;
  #peakQueuedSamples = 0;
  #peakQueuedBytes = 0;
  #flushing = false;
  #stopped = false;
  #queueLowNotified = true;

  constructor(options: DecodePumpOptions) {
    validateQueueWatermarks(
      options.maxQueuedSamples,
      options.queuedSampleLowWatermark,
      options.maxQueuedBytes,
      options.queuedByteLowWatermark,
    );
    this.#options = options;
  }

  get statistics(): DecodePumpStats {
    return {
      maxQueuedSamples: this.#options.maxQueuedSamples,
      queuedSampleLowWatermark: this.#options.queuedSampleLowWatermark,
      maxQueuedBytes: this.#options.maxQueuedBytes,
      queuedByteLowWatermark: this.#options.queuedByteLowWatermark,
      peakQueuedSamples: this.#peakQueuedSamples,
      peakQueuedBytes: this.#peakQueuedBytes,
    };
  }

  setTotalSamples(totalSamples: number): void {
    this.#totalSamples = totalSamples;
  }

  enqueue(sample: T, byteLength: number): void {
    if (this.#stopped) return;
    if (this.#queue.length >= this.#options.maxQueuedSamples) {
      throw new Error(
        `Encoded sample queue exceeded ${this.#options.maxQueuedSamples} samples`,
      );
    }
    if (!Number.isSafeInteger(byteLength) || byteLength < 0) {
      throw new Error(
        "Encoded sample byte length must be a non-negative integer",
      );
    }
    if (byteLength > this.#options.maxQueuedBytes - this.#queuedBytes) {
      throw new Error(
        `Encoded sample queue exceeded ${this.#options.maxQueuedBytes} bytes`,
      );
    }
    this.#queue.push({ value: sample, byteLength });
    // A newly admitted batch must earn its next pull only after pump() has
    // observed the resulting queue. Repeated dequeue/frame callbacks while a
    // Blob read is in flight therefore cannot accumulate stale pull credits.
    this.#queueLowNotified = false;
    this.#queuedBytes += byteLength;
    this.#peakQueuedSamples = Math.max(
      this.#peakQueuedSamples,
      this.#queue.length,
    );
    this.#peakQueuedBytes = Math.max(this.#peakQueuedBytes, this.#queuedBytes);
  }

  pump(decoder: DecodeQueue<T> | undefined, completedFrames: number): void {
    if (this.#stopped || decoder?.state !== "configured" || this.#flushing) {
      return;
    }

    while (
      this.#queue.length > 0 &&
      decoder.decodeQueueSize < this.#options.maxDecodeQueue &&
      this.#samplesFed - completedFrames < this.#options.maxOutstandingFrames
    ) {
      const queued = this.#queue.shift()!;
      this.#queuedBytes -= queued.byteLength;
      try {
        decoder.decode(queued.value);
        this.#samplesFed += 1;
      } catch (error) {
        this.#options.onError(error);
        return;
      }
    }

    if (
      !this.#queueLowNotified &&
      this.#queue.length <= this.#options.queuedSampleLowWatermark &&
      this.#queuedBytes <= this.#options.queuedByteLowWatermark
    ) {
      this.#queueLowNotified = true;
      this.#options.onQueueLow();
    }

    if (
      this.#totalSamples !== null &&
      this.#samplesFed >= this.#totalSamples &&
      !this.#flushing
    ) {
      this.#flushing = true;
      this.#options.onReadyToFlush();
    }
  }

  stop(): void {
    if (this.#stopped) return;
    this.#stopped = true;
    this.#queue.splice(0);
    this.#queuedBytes = 0;
  }
}

function validateQueueWatermarks(
  high: number,
  low: number,
  byteHigh: number,
  byteLow: number,
): void {
  if (!Number.isSafeInteger(high) || high <= 0) {
    throw new Error("Encoded sample queue high watermark must be positive");
  }
  if (!Number.isSafeInteger(low) || low < 0 || low >= high) {
    throw new Error(
      "Encoded sample queue low watermark must be below the high watermark",
    );
  }
  if (!Number.isSafeInteger(byteHigh) || byteHigh <= 0) {
    throw new Error(
      "Encoded sample queue byte high watermark must be positive",
    );
  }
  if (!Number.isSafeInteger(byteLow) || byteLow < 0 || byteLow >= byteHigh) {
    throw new Error(
      "Encoded sample queue byte low watermark must be below the high watermark",
    );
  }
}
