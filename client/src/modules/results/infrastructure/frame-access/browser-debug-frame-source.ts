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
  #videoUrl: string | undefined;

  constructor(
    private readonly data: DebugFrameSourceData,
    onFallbackFrame: () => void,
  ) {
    this.fallbackSource.muted = true;
    this.fallbackSource.addEventListener("seeked", onFallbackFrame, {
      signal: this.#abort.signal,
    });
  }

  get usesExactFrames(): boolean {
    return Boolean(
      this.data.sampleData &&
        this.data.videoArrayBuffer &&
        this.data.codecConfig,
    );
  }

  async initialize(): Promise<void> {
    this.#videoUrl = URL.createObjectURL(this.data.file);
    this.fallbackSource.src = this.#videoUrl;
    await new Promise<void>((resolve, reject) => {
      this.fallbackSource.onloadedmetadata = () => resolve();
      this.fallbackSource.onerror = () => reject(this.fallbackSource.error);
    });
  }

  async decode(index: number): Promise<VideoFrame | null> {
    const { sampleData, videoArrayBuffer, codecConfig, frameToSampleIndex } =
      this.data;
    if (!sampleData || !videoArrayBuffer || !codecConfig) return null;
    return decodeFrameAt({
      samples: sampleData,
      videoArrayBuffer,
      codecConfig,
      frameToSampleIndex,
      frameIndex: index,
    });
  }

  seekFallback(index: number): void {
    this.fallbackSource.currentTime = frameToSeconds(
      index,
      this.data.frameTimestamps,
    );
  }

  destroy(): void {
    this.#abort.abort();
    if (this.#videoUrl) URL.revokeObjectURL(this.#videoUrl);
    this.fallbackSource.removeAttribute("src");
  }
}

export const browserDebugFrameSourceFactory: DebugFrameSourceFactory = {
  create(data, onFallbackFrame) {
    return new BrowserDebugFrameSource(data, onFallbackFrame);
  },
};
