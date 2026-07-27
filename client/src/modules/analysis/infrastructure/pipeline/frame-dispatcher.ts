import type { StripPixels } from "../frame-extraction/strip-extractor.js";

export interface ClosableFrame {
  close(): void;
}

export interface StripFrameExtractor<Frame, Pending> {
  createBitmaps(frame: Frame): Pending;
  readBitmaps(pending: Pending): Promise<StripPixels>;
  readFrame(frame: Frame): StripPixels;
}

interface FrameDispatcherOptions<Frame, Pending> {
  readonly extractor: StripFrameExtractor<Frame, Pending>;
  readonly maxPendingBitmaps: number;
  readonly sendFrame: (
    frameIndex: number,
    pixels: StripPixels,
  ) => Promise<void>;
  readonly onError: (error: unknown) => void;
  readonly now?: () => number;
}

/** Serializes canvas access while bounding asynchronous bitmap creation. */
export class FrameDispatcher<Frame extends ClosableFrame, Pending> {
  readonly #options: FrameDispatcherOptions<Frame, Pending>;
  #chain = Promise.resolve();
  #pendingBitmaps = 0;
  #drawTime = 0;

  constructor(options: FrameDispatcherOptions<Frame, Pending>) {
    this.#options = options;
  }

  get drawTime(): number {
    return this.#drawTime;
  }

  dispatch(frame: Frame, frameIndex: number): void {
    try {
      if (this.#pendingBitmaps < this.#options.maxPendingBitmaps) {
        this.#pendingBitmaps += 1;
        const pending = this.#options.extractor.createBitmaps(frame);
        this.#chain = this.#chain.then(async () => {
          const startedAt = this.#now();
          try {
            const pixels = await this.#options.extractor.readBitmaps(pending);
            this.#drawTime += this.#now() - startedAt;
            await this.#options.sendFrame(frameIndex, pixels);
          } finally {
            frame.close();
            this.#pendingBitmaps -= 1;
          }
        });
      } else {
        const startedAt = this.#now();
        const pixels = this.#options.extractor.readFrame(frame);
        this.#drawTime += this.#now() - startedAt;
        frame.close();
        this.#chain = this.#chain.then(() =>
          this.#options.sendFrame(frameIndex, pixels),
        );
      }
      void this.#chain.catch(this.#options.onError);
    } catch (error) {
      try {
        frame.close();
      } catch {
        // The synchronous extraction path may already have released the frame.
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
