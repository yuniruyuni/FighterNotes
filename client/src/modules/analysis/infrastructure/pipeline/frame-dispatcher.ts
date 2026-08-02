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
  #failure: { readonly reason: unknown } | undefined;
  #drawTime = 0;

  constructor(options: FrameDispatcherOptions<Frame, Pending>) {
    this.#options = options;
  }

  get drawTime(): number {
    return this.#drawTime;
  }

  dispatch(frame: Frame, frameIndex: number): void {
    if (this.#failure) {
      this.#close(frame);
      return;
    }

    try {
      const pending = this.#options.extractor.createBitmaps(frame, frameIndex);
      this.#chain = this.#chain.then(() =>
        this.#readAndDeliver(frame, frameIndex, pending),
      );
    } catch (error) {
      this.#close(frame);
      this.#fail(error);
    }
  }

  async drain(): Promise<void> {
    await this.#chain;
    if (this.#failure) throw this.#failure.reason;
  }

  async #readAndDeliver(
    frame: Frame,
    frameIndex: number,
    pending: Pending,
  ): Promise<void> {
    let failure: { readonly reason: unknown } | undefined;
    try {
      const startedAt = this.#now();
      const pixels = await this.#options.extractor.readBitmaps(pending);
      if (!this.#failure) {
        this.#drawTime += this.#now() - startedAt;
        await this.#options.sendFrame(frameIndex, pixels);
      }
    } catch (error) {
      failure = { reason: error };
    }

    try {
      frame.close();
    } catch (error) {
      failure ??= { reason: error };
    }

    if (failure) this.#fail(failure.reason);
  }

  #close(frame: Frame): void {
    try {
      frame.close();
    } catch {
      // Preserve the first pipeline failure while releasing later frames.
    }
  }

  #fail(error: unknown): void {
    if (this.#failure) return;
    this.#failure = { reason: error };
    try {
      this.#options.onError(error);
    } catch {
      // Error delivery must not strand frames already queued for cleanup.
    }
  }

  #now(): number {
    return this.#options.now?.() ?? performance.now();
  }
}
