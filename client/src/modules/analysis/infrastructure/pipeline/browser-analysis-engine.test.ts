import { expect, test } from "bun:test";
import type { ValidatedVideoInput } from "../../domain/video-preflight.js";
import { analyzeVideo, preflightVideo } from "./browser-analysis-engine.js";

function validatedVideo(file: File): ValidatedVideoInput {
  return {
    file,
    identity: {
      name: file.name,
      size: file.size,
      lastModified: file.lastModified,
      type: file.type,
    },
    metadataBytesRead: 1024,
    track: {
      trackId: 1,
      codec: "avc1.640028",
      codedWidth: 1920,
      codedHeight: 1080,
      displayWidth: 1920,
      displayHeight: 1080,
      rotation: 0,
      framesPerSecond: 60,
      constantFrameRate: true,
      totalSamples: 600,
      maxSampleBytes: 1024,
      timescale: 60_000,
      duration: 600_000,
      decoderConfig: {
        codec: "avc1.640028",
        codedWidth: 1920,
        codedHeight: 1080,
      },
      codecConfig: {
        codec: "avc1.640028",
        width: 1920,
        height: 1080,
      },
    },
  };
}

test("stale validated file is rejected before creating an analysis Worker", async () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "Worker");
  let workersCreated = 0;
  class CountingWorker {
    constructor() {
      workersCreated += 1;
    }
  }
  Object.defineProperty(globalThis, "Worker", {
    configurable: true,
    value: CountingWorker,
  });
  try {
    const selected = new File(["selected"], "selected.mp4", {
      type: "video/mp4",
    });
    const stale = new File(["stale"], "stale.mp4", { type: "video/mp4" });
    await expect(
      analyzeVideo(selected, validatedVideo(stale), "p1", () => undefined),
    ).rejects.toThrow("事前確認済みの動画が一致しません");
    expect(workersCreated).toBe(0);
  } finally {
    if (descriptor) {
      Object.defineProperty(globalThis, "Worker", descriptor);
    }
  }
});

test("runtime feature不足はMP4 parserより前に返す", async () => {
  const descriptor = Object.getOwnPropertyDescriptor(globalThis, "Worker");
  Reflect.deleteProperty(globalThis, "Worker");
  try {
    const result = await preflightVideo(
      new File(["not parsed"], "replay.mp4", { type: "video/mp4" }),
      new AbortController().signal,
    );
    expect(result).toMatchObject({ status: "invalid" });
    if (result.status === "invalid") {
      expect(result.message).toContain("Web Worker");
    }
  } finally {
    if (descriptor) {
      Object.defineProperty(globalThis, "Worker", descriptor);
    }
  }
});
