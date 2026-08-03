import { describe, expect, test } from "bun:test";
import { MAX_ENCODED_SAMPLE_BYTES } from "./encoded-video-limits.js";
import {
  type InspectedVideo,
  type InspectedVideoTrack,
  matchesValidatedVideoFile,
  validateInspectedVideo,
  videoFileIdentity,
  videoPreflightFailure,
} from "./video-preflight.js";

function file(
  name = "replay.mp4",
  options: FilePropertyBag = {
    type: "video/mp4",
    lastModified: 123,
  },
): File {
  return new File(["video"], name, options);
}

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
    ...overrides,
  };
}

function inspected(overrides: Partial<InspectedVideo> = {}): InspectedVideo {
  return {
    container: "mp4",
    fragmented: false,
    metadataBytesRead: 4096,
    track: track(),
    ...overrides,
  };
}

describe("video input preflight", () => {
  test("1920x1080・約60fps・CFR・回転なしのMP4だけを検証済みにする", () => {
    for (const framesPerSecond of [59, 59.94, 60, 61]) {
      const source = file();
      const result = validateInspectedVideo(
        source,
        inspected({ track: track({ framesPerSecond }) }),
      );
      expect(result.status).toBe("valid");
      if (result.status !== "valid") throw new Error("expected valid video");
      expect(result.video.file).toBe(source);
      expect(result.video.identity).toEqual(videoFileIdentity(source));
      expect(result.video.metadataBytesRead).toBe(4096);
      expect(matchesValidatedVideoFile(source, result.video)).toBe(true);
      expect(
        matchesValidatedVideoFile(
          file("replay.mp4", {
            type: "video/mp4",
            lastModified: 123,
          }),
          result.video,
        ),
      ).toBe(false);
    }
  });

  test("container・video track・fragment・timingを理由別に拒否する", () => {
    expect(
      validateInspectedVideo(file("replay.webm"), {
        ...inspected(),
        container: "other",
      }),
    ).toMatchObject({ status: "invalid", code: "non_mp4" });
    expect(
      validateInspectedVideo(file(), inspected({ track: null })),
    ).toMatchObject({ status: "invalid", code: "missing_video_track" });
    expect(
      validateInspectedVideo(file(), inspected({ fragmented: true })),
    ).toMatchObject({ status: "invalid", code: "fragmented_mp4" });
    for (const invalidTrack of [
      track({ totalSamples: 1 }),
      track({ timescale: 0 }),
      track({ duration: 0 }),
      track({ framesPerSecond: Number.NaN }),
    ]) {
      expect(
        validateInspectedVideo(file(), inspected({ track: invalidTrack })),
      ).toMatchObject({ status: "invalid", code: "timing_unavailable" });
    }
  });

  test("回転・寸法・fps・VFRを個別の案内にする", () => {
    for (const rotation of [90, 180, 270, null] as const) {
      const result = validateInspectedVideo(
        file(),
        inspected({ track: track({ rotation }) }),
      );
      expect(result).toMatchObject({ status: "invalid", code: "rotation" });
      if (result.status === "invalid") {
        expect(result.message).toContain(
          rotation === null ? "非標準の変形" : `${rotation}°`,
        );
      }
    }

    const dimensions = validateInspectedVideo(
      file(),
      inspected({
        track: track({
          codedWidth: 2560,
          codedHeight: 1440,
          displayWidth: Number.NaN,
          displayHeight: 1440,
        }),
      }),
    );
    expect(dimensions).toMatchObject({
      status: "invalid",
      code: "dimensions",
    });
    if (dimensions.status === "invalid") {
      expect(dimensions.message).toContain("coded 2560×1440 / 表示 不明×1440");
    }

    for (const framesPerSecond of [58.99, 61.01]) {
      const result = validateInspectedVideo(
        file(),
        inspected({ track: track({ framesPerSecond }) }),
      );
      expect(result).toMatchObject({
        status: "invalid",
        code: "frame_rate",
      });
      if (result.status === "invalid") {
        expect(result.message).toContain(framesPerSecond.toFixed(2));
      }
    }

    expect(
      validateInspectedVideo(
        file(),
        inspected({ track: track({ constantFrameRate: false }) }),
      ),
    ).toMatchObject({ status: "invalid", code: "variable_frame_rate" });
  });

  test("巨大な圧縮フレームを解析開始前に再エンコード案内で拒否する", () => {
    const result = validateInspectedVideo(
      file(),
      inspected({
        track: track({ maxSampleBytes: MAX_ENCODED_SAMPLE_BYTES + 1 }),
      }),
    );

    expect(result).toMatchObject({
      status: "invalid",
      code: "encoded_sample_size",
    });
    if (result.status === "invalid") {
      expect(result.message).toContain("再エンコード");
      expect(result.message).toContain("16MiB");
    }
  });

  test("adapter固有の失敗も同じ閉じたfailure形式にする", () => {
    expect(
      videoPreflightFailure(
        "unsupported_codec",
        "このcodecには対応していません",
      ),
    ).toEqual({
      status: "invalid",
      code: "unsupported_codec",
      message: "このcodecには対応していません",
    });
  });
});
