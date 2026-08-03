import { describe, expect, mock, test } from "bun:test";
import type { DebugFrameSourceData } from "../../application/debug-frame-source.js";
import { browserDebugFrameSourceFactory } from "./browser-debug-frame-source.js";

describe("browser debug frame source lifecycle", () => {
  test("初期化中のdestroyでObject URLを一度だけ解放する", async () => {
    const createObjectURL = mock(() => "blob:debug-video");
    const revokeObjectURL = mock(() => {});
    const restoreUrls = installObjectUrlMocks(createObjectURL, revokeObjectURL);
    const data = sourceData();
    const source = browserDebugFrameSourceFactory.create(data, () => {});
    expect((source.fallbackSource as HTMLVideoElement).preload).toBe(
      "metadata",
    );

    try {
      const initialization = source.initialize();
      source.destroy();
      source.destroy();

      await expect(initialization).rejects.toHaveProperty("name", "AbortError");
      expect(createObjectURL).toHaveBeenCalledTimes(1);
      expect(revokeObjectURL).toHaveBeenCalledTimes(1);
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
    const data = sourceData();
    const source = browserDebugFrameSourceFactory.create(data, () => {});

    try {
      const initialization = source.initialize();
      (source.fallbackSource as HTMLVideoElement).dispatchEvent(
        new Event("loadedmetadata"),
      );
      await initialization;

      const decoding = source.decode(0);
      await Promise.resolve();
      expect(ControlledVideoDecoder.instances).toHaveLength(1);
      source.destroy();
      source.destroy();

      await expect(decoding).rejects.toHaveProperty("name", "AbortError");
      expect(ControlledVideoDecoder.instances[0].closeCount).toBe(1);
      expect(revokeObjectURL).toHaveBeenCalledTimes(1);
    } finally {
      restoreUrls();
      restoreGlobal("VideoDecoder", videoDecoder);
      restoreGlobal("EncodedVideoChunk", encodedVideoChunk);
    }
  });

  test("100件のdecode要求をlatest-winsで直列化する", async () => {
    const restoreUrls = installObjectUrlMocks(
      mock(() => "blob:debug-video"),
      mock(() => {}),
    );
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
    ControlledVideoDecoder.activeCount = 0;
    ControlledVideoDecoder.peakActiveCount = 0;
    const source = browserDebugFrameSourceFactory.create(
      sourceData(),
      () => {},
    );

    try {
      const initialization = source.initialize();
      (source.fallbackSource as HTMLVideoElement).dispatchEvent(
        new Event("loadedmetadata"),
      );
      await initialization;

      const requests: Array<Promise<VideoFrame | null>> = [source.decode(0)];
      await Promise.resolve();
      expect(ControlledVideoDecoder.instances).toHaveLength(1);
      for (let index = 1; index < 100; index += 1) {
        requests.push(source.decode(0));
      }

      expect(await Promise.all(requests.slice(0, -1))).toEqual(
        Array.from({ length: 99 }, () => null),
      );
      expect(ControlledVideoDecoder.instances).toHaveLength(2);
      expect(ControlledVideoDecoder.peakActiveCount).toBe(1);

      source.destroy();
      await expect(requests.at(-1)!).rejects.toHaveProperty(
        "name",
        "AbortError",
      );
      expect(ControlledVideoDecoder.activeCount).toBe(0);
    } finally {
      source.destroy();
      restoreUrls();
      restoreGlobal("VideoDecoder", videoDecoder);
      restoreGlobal("EncodedVideoChunk", encodedVideoChunk);
    }
  });
});

function sourceData(): DebugFrameSourceData {
  return {
    file: new File(["video"], "replay.mp4", { type: "video/mp4" }),
    frameTimestamps: [0],
    sampleData: [{ isSync: true, timestampUs: 0, offset: 0, size: 1 }],
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
  static activeCount = 0;
  static peakActiveCount = 0;
  state: CodecState = "unconfigured";
  closeCount = 0;
  #flushed: ReturnType<typeof deferred<void>> | undefined;

  constructor() {
    ControlledVideoDecoder.instances.push(this);
  }

  configure(): void {
    this.state = "configured";
    ControlledVideoDecoder.activeCount += 1;
    ControlledVideoDecoder.peakActiveCount = Math.max(
      ControlledVideoDecoder.peakActiveCount,
      ControlledVideoDecoder.activeCount,
    );
  }

  decode(): void {}

  flush(): Promise<void> {
    this.#flushed ??= deferred<void>();
    return this.#flushed.promise;
  }

  close(): void {
    if (this.state === "closed") return;
    this.state = "closed";
    this.closeCount += 1;
    ControlledVideoDecoder.activeCount -= 1;
    this.#flushed?.reject(new DOMException("decoder closed", "AbortError"));
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
