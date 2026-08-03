import { describe, expect, mock, test } from "bun:test";
import type {
  InspectedVideo,
  InspectedVideoTrack,
} from "../../domain/video-preflight.js";
import {
  preflightBrowserVideo,
  probeVideoFrameBitmap,
} from "./browser-video-preflight.js";
import { Mp4InspectionError } from "./mp4-video-preflight.js";

function track(
  overrides: Partial<InspectedVideoTrack> = {},
): InspectedVideoTrack {
  return {
    trackId: 1,
    codec: "avc1.640028",
    codedWidth: 1920,
    codedHeight: 1080,
    displayWidth: 1920,
    displayHeight: 1080,
    rotation: 0,
    framesPerSecond: 59.94,
    constantFrameRate: true,
    totalSamples: 600,
    maxSampleBytes: 1024,
    timescale: 60_000,
    duration: 600_600,
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
    ...overrides,
  };
}

function inspected(overrides: Partial<InspectedVideo> = {}): InspectedVideo {
  return {
    container: "mp4",
    fragmented: false,
    metadataBytesRead: 8192,
    track: track(),
    ...overrides,
  };
}

describe("browser video preflight", () => {
  test("metadata・codec・VideoFrame bitmap probeが全て通った動画を返す", async () => {
    const file = new File(["mp4"], "replay.mp4", { type: "video/mp4" });
    const checkDecoder = mock(async (config: VideoDecoderConfig) => ({
      supported: true,
      config,
    }));
    const probeFrameBitmap = mock(async () => true);
    const result = await preflightBrowserVideo(
      file,
      new AbortController().signal,
      {
        inspect: async () => inspected(),
        checkDecoder,
        probeFrameBitmap,
      },
    );

    expect(result.status).toBe("valid");
    if (result.status !== "valid") throw new Error("expected valid video");
    expect(result.video.file).toBe(file);
    expect(checkDecoder).toHaveBeenCalledWith(result.video.track.decoderConfig);
    expect(probeFrameBitmap).toHaveBeenCalledTimes(1);
  });

  test("非対応metadataではcodec/bitmap probeまで進めない", async () => {
    const checkDecoder = mock(async () => ({ supported: true }));
    const probeFrameBitmap = mock(async () => true);
    const result = await preflightBrowserVideo(
      new File(["mp4"], "replay.mp4", { type: "video/mp4" }),
      new AbortController().signal,
      {
        inspect: async () => inspected({ track: track({ codedWidth: 1280 }) }),
        checkDecoder,
        probeFrameBitmap,
      },
    );

    expect(result).toMatchObject({ status: "invalid", code: "dimensions" });
    expect(checkDecoder).not.toHaveBeenCalled();
    expect(probeFrameBitmap).not.toHaveBeenCalled();
  });

  test("非MP4・壊れたMP4・非対応codec・bitmap失敗を個別に返す", async () => {
    const source = new File(["video"], "replay.mp4", { type: "video/mp4" });
    const signal = new AbortController().signal;
    expect(
      await preflightBrowserVideo(source, signal, {
        inspect: async () => {
          throw new Mp4InspectionError("non_mp4", "not mp4");
        },
      }),
    ).toMatchObject({ status: "invalid", code: "non_mp4" });
    const oversizedMetadata = await preflightBrowserVideo(source, signal, {
      inspect: async () => {
        throw new Mp4InspectionError("metadata_size", "too much metadata");
      },
    });
    expect(oversizedMetadata).toMatchObject({
      status: "invalid",
      code: "metadata_size",
    });
    if (oversizedMetadata.status === "invalid") {
      expect(oversizedMetadata.message).toContain("再エンコード");
    }
    const broken = await preflightBrowserVideo(source, signal, {
      inspect: async () => {
        throw new Error("broken moov");
      },
    });
    expect(broken).toMatchObject({ status: "invalid", code: "invalid_mp4" });
    if (broken.status === "invalid") {
      expect(broken.message).toContain("broken moov");
    }

    expect(
      await preflightBrowserVideo(source, signal, {
        inspect: async () => inspected(),
        checkDecoder: async () => ({ supported: false }),
      }),
    ).toMatchObject({ status: "invalid", code: "unsupported_codec" });
    expect(
      await preflightBrowserVideo(source, signal, {
        inspect: async () => inspected(),
        checkDecoder: async () => {
          throw new Error("codec probe failed");
        },
      }),
    ).toMatchObject({ status: "invalid", code: "unsupported_codec" });
    expect(
      await preflightBrowserVideo(source, signal, {
        inspect: async () => inspected(),
        checkDecoder: async (config) => ({ supported: true, config }),
        probeFrameBitmap: async () => false,
      }),
    ).toMatchObject({ status: "invalid", code: "frame_extraction" });
    const extractionError = await preflightBrowserVideo(source, signal, {
      inspect: async () => inspected(),
      checkDecoder: async (config) => ({ supported: true, config }),
      probeFrameBitmap: async () => {
        throw new Error("bitmap overload failed");
      },
    });
    expect(extractionError).toMatchObject({
      status: "invalid",
      code: "frame_extraction",
    });
    if (extractionError.status === "invalid") {
      expect(extractionError.message).toContain("bitmap overload failed");
    }
  });

  test("VideoFrameとImageBitmapを成功・中止の双方で解放する", async () => {
    const closeFrame = mock(() => undefined);
    const closeBitmap = mock(() => undefined);
    const canvas = {
      getContext: () => ({}),
    } as unknown as OffscreenCanvas;
    const frame = { close: closeFrame } as unknown as VideoFrame;
    const bitmap = {
      width: 2,
      height: 2,
      close: closeBitmap,
    } as unknown as ImageBitmap;
    const controller = new AbortController();
    expect(
      await probeVideoFrameBitmap(controller.signal, {
        createCanvas: () => canvas,
        createFrame: () => frame,
        createBitmap: async () => bitmap,
      }),
    ).toBe(true);
    expect(closeFrame).toHaveBeenCalledTimes(1);
    expect(closeBitmap).toHaveBeenCalledTimes(1);

    expect(
      await probeVideoFrameBitmap(new AbortController().signal, {
        createCanvas: () =>
          ({ getContext: () => null }) as unknown as OffscreenCanvas,
      }),
    ).toBe(false);

    const aborted = new AbortController();
    aborted.abort(new Error("stale selection"));
    await expect(
      probeVideoFrameBitmap(aborted.signal, {
        createCanvas: () => canvas,
      }),
    ).rejects.toThrow("stale selection");
  });
});
