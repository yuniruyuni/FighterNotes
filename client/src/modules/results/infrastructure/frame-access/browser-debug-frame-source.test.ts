import { describe, expect, mock, test } from "bun:test";
import type { DebugFrameSourceData } from "../../application/debug-frame-source.js";
import { browserDebugFrameSourceFactory } from "./browser-debug-frame-source.js";

describe("browser debug frame source lifecycle", () => {
  test("初期化中のdestroyでObject URLと動画bufferを一度だけ解放する", async () => {
    const createObjectURL = mock(() => "blob:debug-video");
    const revokeObjectURL = mock(() => {});
    const restoreUrls = installObjectUrlMocks(createObjectURL, revokeObjectURL);
    const data = sourceData(new ArrayBuffer(8 * 1024 * 1024));
    const source = browserDebugFrameSourceFactory.create(data, () => {});

    try {
      const initialization = source.initialize();
      source.destroy();
      source.destroy();

      await expect(initialization).rejects.toHaveProperty("name", "AbortError");
      expect(createObjectURL).toHaveBeenCalledTimes(1);
      expect(revokeObjectURL).toHaveBeenCalledTimes(1);
      expect(data.videoArrayBuffer).toBeNull();
      expect(source.usesExactFrames).toBeFalse();
      expect(
        (source.fallbackSource as HTMLVideoElement).hasAttribute("src"),
      ).toBeFalse();
    } finally {
      restoreUrls();
    }
  });

  test("destroyで進行中decoderを閉じ、decode待機をabortする", async () => {
    const createObjectURL = mock(() => "blob:debug-video");
    const revokeObjectURL = mock(() => {});
    const restoreUrls = installObjectUrlMocks(createObjectURL, revokeObjectURL);
    const videoDecoder = Object.getOwnPropertyDescriptor(
      globalThis,
      "VideoDecoder",
    );
    const encodedVideoChunk = Object.getOwnPropertyDescriptor(
      globalThis,
      "EncodedVideoChunk",
    );
    Object.defineProperty(globalThis, "VideoDecoder", {
      configurable: true,
      writable: true,
      value: ControlledVideoDecoder,
    });
    Object.defineProperty(globalThis, "EncodedVideoChunk", {
      configurable: true,
      writable: true,
      value: FakeEncodedVideoChunk,
    });
    ControlledVideoDecoder.instances = [];
    const data = sourceData(new ArrayBuffer(1));
    const source = browserDebugFrameSourceFactory.create(data, () => {});

    try {
      const initialization = source.initialize();
      (source.fallbackSource as HTMLVideoElement).dispatchEvent(
        new Event("loadedmetadata"),
      );
      await initialization;

      const decoding = source.decode(0);
      expect(ControlledVideoDecoder.instances).toHaveLength(1);
      source.destroy();
      source.destroy();

      await expect(decoding).rejects.toHaveProperty("name", "AbortError");
      expect(ControlledVideoDecoder.instances[0].closeCount).toBe(1);
      expect(revokeObjectURL).toHaveBeenCalledTimes(1);
      expect(data.videoArrayBuffer).toBeNull();
    } finally {
      restoreUrls();
      restoreGlobal("VideoDecoder", videoDecoder);
      restoreGlobal("EncodedVideoChunk", encodedVideoChunk);
    }
  });
});

function sourceData(videoArrayBuffer: ArrayBuffer): DebugFrameSourceData {
  return {
    file: new File(["video"], "replay.mp4", { type: "video/mp4" }),
    frameTimestamps: [0],
    sampleData: [{ isSync: true, timestampUs: 0, offset: 0, size: 1 }],
    videoArrayBuffer,
    codecConfig: { codec: "avc1.42E01E", width: 1920, height: 1080 },
    frameToSampleIndex: [0],
  };
}

function installObjectUrlMocks(
  createObjectURL: () => string,
  revokeObjectURL: (url: string) => void,
): () => void {
  const create = Object.getOwnPropertyDescriptor(URL, "createObjectURL");
  const revoke = Object.getOwnPropertyDescriptor(URL, "revokeObjectURL");
  Object.defineProperty(URL, "createObjectURL", {
    configurable: true,
    value: createObjectURL,
  });
  Object.defineProperty(URL, "revokeObjectURL", {
    configurable: true,
    value: revokeObjectURL,
  });
  return () => {
    restoreProperty(URL, "createObjectURL", create);
    restoreProperty(URL, "revokeObjectURL", revoke);
  };
}

function restoreGlobal(
  key: "VideoDecoder" | "EncodedVideoChunk",
  descriptor: PropertyDescriptor | undefined,
): void {
  restoreProperty(globalThis, key, descriptor);
}

function restoreProperty(
  target: object,
  key: PropertyKey,
  descriptor: PropertyDescriptor | undefined,
): void {
  if (descriptor) Object.defineProperty(target, key, descriptor);
  else Reflect.deleteProperty(target, key);
}

class ControlledVideoDecoder {
  static instances: ControlledVideoDecoder[] = [];
  state: CodecState = "unconfigured";
  closeCount = 0;
  readonly #flushed = deferred<void>();

  constructor() {
    ControlledVideoDecoder.instances.push(this);
  }

  configure(): void {
    this.state = "configured";
  }

  decode(): void {}

  flush(): Promise<void> {
    return this.#flushed.promise;
  }

  close(): void {
    if (this.state === "closed") return;
    this.state = "closed";
    this.closeCount += 1;
    this.#flushed.reject(new DOMException("decoder closed", "AbortError"));
  }
}

class FakeEncodedVideoChunk {}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}
