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
  #data: DebugFrameSourceData | null;
  #videoUrl: string | undefined;
  #destroyed = false;

  constructor(data: DebugFrameSourceData, onFallbackFrame: () => void) {
    this.#data = data;
    this.fallbackSource.muted = true;
    this.fallbackSource.addEventListener("seeked", onFallbackFrame, {
      signal: this.#abort.signal,
    });
  }

  get usesExactFrames(): boolean {
    const data = this.#data;
    return Boolean(
      data?.sampleData && data.videoArrayBuffer && data.codecConfig,
    );
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
    const data = this.#requireData();
    const { sampleData, videoArrayBuffer, codecConfig, frameToSampleIndex } =
      data;
    if (!sampleData || !videoArrayBuffer || !codecConfig) return null;
    return decodeFrameAt({
      samples: sampleData,
      videoArrayBuffer,
      codecConfig,
      frameToSampleIndex,
      frameIndex: index,
      signal: this.#abort.signal,
    });
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
    this.#abort.abort(this.#abortError);
    if (this.#videoUrl) {
      URL.revokeObjectURL(this.#videoUrl);
      this.#videoUrl = undefined;
    }
    if (this.#data) this.#data.videoArrayBuffer = null;
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

export const browserDebugFrameSourceFactory: DebugFrameSourceFactory = {
  create(data, onFallbackFrame) {
    return new BrowserDebugFrameSource(data, onFallbackFrame);
  },
};
