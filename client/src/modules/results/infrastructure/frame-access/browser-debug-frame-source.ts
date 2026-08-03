import { decodeFrameAt } from "~/modules/analysis/browser.js";
import type {
  DebugFrameSource,
  DebugFrameSourceData,
  DebugFrameSourceFactory,
} from "../../application/debug-frame-source.js";
import { frameToSeconds } from "../../domain/frame-time.js";

class BrowserDebugFrameSource implements DebugFrameSource {
  readonly fallbackSource = document.createElement("video");
  readonly #abort = new AbortController();
  readonly #abortError = new DOMException(
    "認識デバッグの動画読込を中断しました",
    "AbortError",
  );
  readonly #supersededError = new DOMException(
    "新しいフレーム要求で置き換えました",
    "AbortError",
  );
  #data: DebugFrameSourceData | null;
  #videoUrl: string | undefined;
  #destroyed = false;
  #decodeGeneration = 0;
  #activeDecode: AbortController | undefined;
  #decodeSettled: Promise<void> = Promise.resolve();

  constructor(data: DebugFrameSourceData, onFallbackFrame: () => void) {
    this.#data = data;
    this.fallbackSource.muted = true;
    this.fallbackSource.preload = "metadata";
    this.fallbackSource.addEventListener("seeked", onFallbackFrame, {
      signal: this.#abort.signal,
    });
  }

  get usesExactFrames(): boolean {
    const data = this.#data;
    return Boolean(data?.sampleData && data.codecConfig);
  }

  async initialize(): Promise<void> {
    const data = this.#requireData();
    this.#videoUrl = URL.createObjectURL(data.file);
    this.fallbackSource.src = this.#videoUrl;
    await new Promise<void>((resolve, reject) => {
      let settled = false;
      const cleanup = () => {
        this.#abort.signal.removeEventListener("abort", onAbort);
        this.fallbackSource.onloadedmetadata = null;
        this.fallbackSource.onerror = null;
      };
      const settle = (callback: () => void) => {
        if (settled) return;
        settled = true;
        cleanup();
        callback();
      };
      const onAbort = () => settle(() => reject(this.#abortError));
      this.fallbackSource.onloadedmetadata = () => settle(resolve);
      this.fallbackSource.onerror = () =>
        settle(() =>
          reject(
            this.fallbackSource.error ??
              new Error("デバッグ動画を読み込めませんでした"),
          ),
        );
      this.#abort.signal.addEventListener("abort", onAbort, { once: true });
      if (this.#abort.signal.aborted) onAbort();
    });
  }

  async decode(index: number): Promise<VideoFrame | null> {
    const generation = ++this.#decodeGeneration;
    this.#activeDecode?.abort(this.#supersededError);
    await this.#decodeSettled;
    if (generation !== this.#decodeGeneration) return null;
    const data = this.#requireData();
    const { file, sampleData, codecConfig, frameToSampleIndex } = data;
    if (!sampleData || !codecConfig) return null;
    const controller = new AbortController();
    this.#activeDecode = controller;
    const decoding = decodeFrameAt({
      samples: sampleData,
      videoBlob: file,
      codecConfig,
      frameToSampleIndex,
      frameIndex: index,
      signal: controller.signal,
    });
    this.#decodeSettled = decoding.then(
      () => undefined,
      () => undefined,
    );
    try {
      const frame = await decoding;
      if (generation !== this.#decodeGeneration || controller.signal.aborted) {
        frame?.close();
        if (controller.signal.reason === this.#supersededError) return null;
        throw abortReason(controller.signal, this.#abortError);
      }
      return frame;
    } catch (error) {
      if (
        controller.signal.aborted &&
        controller.signal.reason === this.#supersededError
      ) {
        return null;
      }
      throw error;
    } finally {
      if (this.#activeDecode === controller) this.#activeDecode = undefined;
    }
  }

  seekFallback(index: number): void {
    const data = this.#data;
    if (!data) return;
    this.fallbackSource.currentTime = frameToSeconds(
      index,
      data.frameTimestamps,
    );
  }

  destroy(): void {
    if (this.#destroyed) return;
    this.#destroyed = true;
    this.#decodeGeneration += 1;
    this.#activeDecode?.abort(this.#abortError);
    this.#activeDecode = undefined;
    this.#abort.abort(this.#abortError);
    if (this.#videoUrl) {
      URL.revokeObjectURL(this.#videoUrl);
      this.#videoUrl = undefined;
    }
    this.#data = null;
    this.fallbackSource.pause();
    this.fallbackSource.removeAttribute("src");
    this.fallbackSource.load();
  }

  #requireData(): DebugFrameSourceData {
    if (!this.#data) throw this.#abortError;
    return this.#data;
  }
}

function abortReason(signal: AbortSignal, fallback: DOMException): unknown {
  return signal.reason instanceof Error ? signal.reason : fallback;
}

export const browserDebugFrameSourceFactory: DebugFrameSourceFactory = {
  create(data, onFallbackFrame) {
    return new BrowserDebugFrameSource(data, onFallbackFrame);
  },
};
