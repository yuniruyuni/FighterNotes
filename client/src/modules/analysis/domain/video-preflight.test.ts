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
      // 単位換算がずれると案内する上限値が変わるので、文言全体を固定する。
      expect(result.message).toBe(
        "動画内の圧縮フレームが大きすぎるため解析できません。" +
          "映像品質またはビットレートを下げ、1フレームを16MiB以下にして" +
          "MP4を再エンコードしてください。",
      );
    }
  });

  /**
   * 圧縮フレームの上限は「大きすぎる」だけでなく、そもそも測れない値も
   * 弾く。0以下や非整数を通すと、batch を組めないまま解析へ進む。
   */
  test("測定できない圧縮フレームサイズを拒否する", () => {
    for (const maxSampleBytes of [
      0,
      -1,
      1.5,
      Number.NaN,
      Number.POSITIVE_INFINITY,
      MAX_ENCODED_SAMPLE_BYTES + 1,
    ]) {
      expect(
        validateInspectedVideo(
          file(),
          inspected({ track: track({ maxSampleBytes }) }),
        ),
      ).toMatchObject({ status: "invalid", code: "encoded_sample_size" });
    }

    // 上限ちょうどは受け入れる。
    expect(
      validateInspectedVideo(
        file(),
        inspected({
          track: track({ maxSampleBytes: MAX_ENCODED_SAMPLE_BYTES }),
        }),
      ).status,
    ).toBe("valid");
  });

  test("VFRは固定fpsでの録画し直しを案内する", () => {
    const result = validateInspectedVideo(
      file(),
      inspected({ track: track({ constantFrameRate: false }) }),
    );

    expect(result).toMatchObject({
      status: "invalid",
      code: "variable_frame_rate",
    });
    if (result.status !== "invalid") throw new Error("expected a failure");
    expect(result.message).toBe(
      "可変フレームレート（VFR）を検出しました。" +
        "OBSなどで固定60fps（CFR）を指定して録画し直してください。",
    );
  });

  test("File識別子を実際の値として取り出す", () => {
    const source = file("replay.mp4", {
      type: "video/mp4",
      lastModified: 456,
    });

    expect(videoFileIdentity(source)).toEqual({
      name: "replay.mp4",
      size: source.size,
      lastModified: 456,
      type: "video/mp4",
    });
    expect(source.size).toBeGreaterThan(0);
  });

  test("同一Fileでも記録した識別子と1項目でも違えば別物として扱う", () => {
    const source = file();
    const result = validateInspectedVideo(source, inspected());
    if (result.status !== "valid") throw new Error("expected valid video");
    const validated = result.video;

    expect(matchesValidatedVideoFile(source, validated)).toBe(true);

    // File参照が同じでも、記録済み識別子が食い違えば再利用してはならない。
    // 各項目が独立に効くことを1項目ずつ確認する。
    const drifted: Array<Partial<typeof validated.identity>> = [
      { name: "other.mp4" },
      { size: validated.identity.size + 1 },
      { lastModified: validated.identity.lastModified + 1 },
      { type: "video/quicktime" },
    ];
    for (const override of drifted) {
      expect(
        matchesValidatedVideoFile(source, {
          ...validated,
          identity: { ...validated.identity, ...override },
        }),
      ).toBe(false);
    }
  });

  test("拒否理由ごとに対処方法を含む案内を返す", () => {
    const cases = [
      {
        video: { ...inspected(), container: "other" as const },
        code: "non_mp4",
        contains: "MP4",
      },
      {
        video: inspected({ track: null }),
        code: "missing_video_track",
        contains: "映像トラック",
      },
      {
        video: inspected({ fragmented: true }),
        code: "fragmented_mp4",
        contains: "再多重化",
      },
      {
        video: inspected({ track: track({ timescale: 0 }) }),
        code: "timing_unavailable",
        contains: "CFR",
      },
    ];

    for (const { video, code, contains } of cases) {
      const result = validateInspectedVideo(file(), video);
      expect(result).toMatchObject({ status: "invalid", code });
      if (result.status !== "invalid") throw new Error("expected a failure");
      expect(result.message).toContain(contains);
      expect(result.message.length).toBeGreaterThan(0);
    }
  });

  test("フレーム時刻を測れる最小sample数を受け入れる", () => {
    // 2 sample あれば間隔を1つ測れる。ここが境界なので、1は拒否・2は受理。
    expect(
      validateInspectedVideo(
        file(),
        inspected({ track: track({ totalSamples: 2 }) }),
      ).status,
    ).toBe("valid");
    expect(
      validateInspectedVideo(
        file(),
        inspected({ track: track({ totalSamples: 1 }) }),
      ),
    ).toMatchObject({ status: "invalid", code: "timing_unavailable" });
  });

  test("coded・表示のどの寸法が違っても寸法違いとして拒否する", () => {
    const overrides: Array<Partial<InspectedVideoTrack>> = [
      { codedWidth: 1280 },
      { codedHeight: 720 },
      { displayWidth: 1280 },
      { displayHeight: 720 },
    ];

    for (const override of overrides) {
      expect(
        validateInspectedVideo(file(), inspected({ track: track(override) })),
      ).toMatchObject({ status: "invalid", code: "dimensions" });
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
