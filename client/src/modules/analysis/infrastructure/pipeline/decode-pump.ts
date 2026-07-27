export interface DecodeQueue<T> {
  readonly state: string;
  readonly decodeQueueSize: number;
  decode(sample: T): void;
}

interface DecodePumpOptions {
  readonly maxDecodeQueue: number;
  readonly maxInflightFrames: number;
  readonly onReadyToFlush: () => void;
  readonly onError: (error: unknown) => void;
}

/** Owns sample admission and the one-shot transition into decoder flushing. */
export class DecodePump<T> {
  readonly #options: DecodePumpOptions;
  readonly #queue: T[] = [];
  #totalSamples: number | null = null;
  #samplesFed = 0;
  #flushing = false;

  constructor(options: DecodePumpOptions) {
    this.#options = options;
  }

  setTotalSamples(totalSamples: number): void {
    this.#totalSamples = totalSamples;
  }

  enqueue(sample: T): void {
    this.#queue.push(sample);
  }

  pump(decoder: DecodeQueue<T> | undefined, inflightFrames: number): void {
    if (decoder?.state !== "configured" || this.#flushing) return;

    while (
      this.#queue.length > 0 &&
      decoder.decodeQueueSize < this.#options.maxDecodeQueue &&
      inflightFrames < this.#options.maxInflightFrames
    ) {
      try {
        decoder.decode(this.#queue.shift()!);
        this.#samplesFed += 1;
      } catch (error) {
        this.#options.onError(error);
        return;
      }
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
}
