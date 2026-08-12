import { describe, expect, mock, test } from "bun:test";
import type { DebugFrameInspector } from "../../application/debug-frame-inspection.js";
import type { DebugFrameSource } from "../../application/debug-frame-source.js";
import { initialDebugOverlayVisibility } from "./debug-viewer-model.js";
import { createDebugViewerSession } from "./debug-viewer-session.js";

describe("debug viewer request ordering", () => {
  test("same-frame superseded null never renders stale fallback or frame info", async () => {
    const initialFrame = fakeFrame();
    const superseded = deferred<VideoFrame | null>();
    const latest = deferred<VideoFrame | null>();
    const decode = mock((index: number) => {
      expect(index).toBe(0);
      const call = decode.mock.calls.length;
      if (call === 1) return Promise.resolve(initialFrame.frame);
      if (call === 2) return superseded.promise;
      if (call === 3) return latest.promise;
      throw new Error(`unexpected decode call ${call}`);
    });
    const source: DebugFrameSource = {
      fallbackSource: document.createElement("canvas"),
      usesExactFrames: true,
      initialize: async () => undefined,
      decode,
      seekFallback() {},
      destroy() {},
    };
    const drawImage = mock(() => undefined);
    const onFrameInfo = mock(() => undefined);
    const canvas = document.createElement("canvas");
    Object.defineProperty(canvas, "getContext", {
      value: () =>
        ({
          drawImage,
          fillRect() {},
          fillText() {},
        }) as unknown as CanvasRenderingContext2D,
    });
    const controller = new AbortController();
    const session = await createDebugViewerSession({
      canvas,
      data: {
        file: new File([Uint8Array.of(1)], "replay.mp4", {
          type: "video/mp4",
        }),
        timeline: {
          left: { side: "left", segments: [] },
          right: { side: "right", segments: [] },
          video_map: {},
        },
        hpFeatures: [],
        trackedInputs: null,
        attackInfo: [],
        frameCount: 1,
        frameTimestamps: [0],
        sampleData: [{ isSync: true, timestampUs: 0, offset: 0, size: 1 }],
        codecConfig: { codec: "fake", width: 1920, height: 1080 },
        frameToSampleIndex: [0],
      },
      ownSide: "p1",
      signal: controller.signal,
      visibility: initialDebugOverlayVisibility(),
      frameSourceFactory: { create: () => source },
      frameInspector: inspector(),
      onFrameInfo,
      onPlayingChange: () => undefined,
      onError: (cause) => {
        throw cause;
      },
    });
    expect(initialFrame.close).toHaveBeenCalledTimes(1);
    expect(drawImage).toHaveBeenCalledTimes(1);
    expect(onFrameInfo).toHaveBeenCalledTimes(1);

    const firstRequest = session.setVisibility(initialDebugOverlayVisibility());
    const latestRequest = session.setVisibility(
      initialDebugOverlayVisibility(),
    );
    superseded.resolve(null);
    await firstRequest;

    expect(drawImage).toHaveBeenCalledTimes(1);
    expect(onFrameInfo).toHaveBeenCalledTimes(1);
    const failure = new Error("latest decode failed");
    latest.reject(failure);
    await expect(latestRequest).rejects.toBe(failure);
    expect(drawImage).toHaveBeenCalledTimes(1);
    expect(onFrameInfo).toHaveBeenCalledTimes(1);
    session.destroy();
  });
});

function fakeFrame(): {
  readonly frame: VideoFrame;
  readonly close: ReturnType<typeof mock>;
} {
  const close = mock(() => undefined);
  return {
    frame: { close } as unknown as VideoFrame,
    close,
  };
}

function inspector(): DebugFrameInspector {
  const unexpected = () => {
    throw new Error("overlay inspection was not expected");
  };
  return {
    initialize: async () => ({
      p1: parallelogram(),
      p2: parallelogram(),
    }),
    inspectMeter: unexpected,
    inspectHp: unexpected,
    inspectDrive: unexpected,
    inspectSuper: unexpected,
    inspectInput: unexpected,
    inspectAttackInfo: unexpected,
  };
}

function parallelogram() {
  return {
    top_left: { x: 0, y: 0 },
    top_right: { x: 1, y: 0 },
    bottom_right: { x: 1, y: 1 },
    bottom_left: { x: 0, y: 1 },
  };
}

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}
