import type { StripPixels } from "../frame-extraction/strip-extractor.js";

export interface ClosableFrame {
  close(): void;
}

export interface StripFrameExtractor<Frame, Pending> {
  createBitmaps(frame: Frame, frameIndex: number): Pending;
  readBitmaps(pending: Pending): Promise<StripPixels>;
}

interface FrameDispatcherOptions<Frame, Pending> {
  readonly extractor: StripFrameExtractor<Frame, Pending>;
  readonly sendFrame: (
    frameIndex: number,
    pixels: StripPixels,
  ) => Promise<void>;
  readonly onError: (error: unknown) => void;
  readonly now?: () => number;
}

/** Starts bitmap extraction eagerly, then serializes canvas access and delivery. */
export class FrameDispatcher<Frame extends ClosableFrame, Pending> {
  readonly #options: FrameDispatcherOptions<Frame, Pending>;
  #chain = Promise.resolve();
  #drawTime = 0;

  constructor(options: FrameDispatcherOptions<Frame, Pending>) {
    this.#options = options;
  }

  get drawTime(): number {
    return this.#drawTime;
  }

  dispatch(frame: Frame, frameIndex: number): void {
    try {
      const pending = this.#options.extractor.createBitmaps(frame, frameIndex);
      this.#chain = this.#chain.then(async () => {
        const startedAt = this.#now();
        try {
          const pixels = await this.#options.extractor.readBitmaps(pending);
          this.#drawTime += this.#now() - startedAt;
          await this.#options.sendFrame(frameIndex, pixels);
        } finally {
          frame.close();
        }
      });
      void this.#chain.catch(this.#options.onError);
    } catch (error) {
      try {
        frame.close();
      } catch {
        // Ignore a secondary close failure while reporting extraction failure.
      }
      this.#options.onError(error);
    }
  }

  drain(): Promise<void> {
    return this.#chain;
  }

  #now(): number {
    return this.#options.now?.() ?? performance.now();
  }
}
