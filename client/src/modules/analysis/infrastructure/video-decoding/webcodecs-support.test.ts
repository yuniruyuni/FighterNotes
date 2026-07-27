import { describe, expect, test } from "bun:test";
import { requireSupportedVideoDecoder } from "./webcodecs-support.js";

const config: VideoDecoderConfig = {
  codec: "avc1.640028",
  codedWidth: 1920,
  codedHeight: 1080,
};

describe("video decoder support", () => {
  test("対応するcodec設定を受け入れる", async () => {
    await expect(
      requireSupportedVideoDecoder(config, async () => ({ supported: true })),
    ).resolves.toBeUndefined();
  });

  test("非対応codec設定を利用者向けエラーにする", async () => {
    await expect(
      requireSupportedVideoDecoder(config, async () => ({ supported: false })),
    ).rejects.toThrow(/avc1\.640028.*WebCodecs/);
  });

  test("codec検証APIの例外も利用者向けエラーにする", async () => {
    await expect(
      requireSupportedVideoDecoder(config, async () => {
        throw new TypeError("invalid config");
      }),
    ).rejects.toThrow(/対応するMP4動画/);
  });
});
