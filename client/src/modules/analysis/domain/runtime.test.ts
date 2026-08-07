import { describe, expect, test } from "bun:test";
import { AnalysisRuntime } from "./runtime.js";

function capabilities(
  overrides: Partial<Parameters<typeof AnalysisRuntime.evaluate>[0]> = {},
) {
  return {
    secureContext: true,
    hasWorker: true,
    hasOffscreenCanvas2d: true,
    hasVideoFrameBitmap: true,
    hasVideoDecoder: true,
    origin: "https://fighter.example",
    ...overrides,
  };
}

describe("analysis runtime", () => {
  test("Secure ContextでVideoDecoderがあれば解析できる", () => {
    expect(AnalysisRuntime.evaluate(capabilities())).toEqual({
      available: true,
    });
  });

  test("VideoDecoder非対応ブラウザでは解析を開始しない", () => {
    const readiness = AnalysisRuntime.evaluate(
      capabilities({
        hasVideoDecoder: false,
        origin: "http://localhost:3001",
      }),
    );

    expect(readiness.available).toBe(false);
    if (readiness.available) throw new Error("expected unavailable runtime");
    expect(readiness.reason).toBe(
      "このブラウザは動画解析に必要なWebCodecs VideoDecoderに対応していません。" +
        " ブラウザを最新版に更新するか、対応ブラウザで開いてください。",
    );
  });

  test("信頼されないHTTP originでは解析を開始しない", () => {
    const readiness = AnalysisRuntime.evaluate(
      capabilities({
        secureContext: false,
        hasVideoDecoder: false,
        origin: "http://192.168.10.3:3001",
      }),
    );

    expect(readiness.available).toBe(false);
    if (readiness.available) throw new Error("expected unavailable runtime");
    expect(readiness.reason).toContain("HTTPSまたはlocalhost");
    expect(readiness.reason).toContain("http://192.168.10.3:3001");
  });

  test("Worker・OffscreenCanvas 2D・VideoFrame切り出しを理由別に要求する", () => {
    const cases = [
      ["hasWorker", "Web Worker"],
      ["hasOffscreenCanvas2d", "OffscreenCanvas 2D"],
      ["hasVideoFrameBitmap", "VideoFrameからの画像切り出し"],
    ] as const;
    for (const [capability, message] of cases) {
      const readiness = AnalysisRuntime.evaluate(
        capabilities({ [capability]: false }),
      );
      expect(readiness.available).toBe(false);
      if (readiness.available) throw new Error("expected unavailable runtime");
      // 不足している機能の名指しと、利用者が取れる対処の両方を必ず出す。
      // 名前だけでは何をすればよいか分からない。
      expect(readiness.reason).toContain(message);
      expect(readiness.reason).toContain(
        "ブラウザを最新版に更新するか、対応ブラウザで開いてください。",
      );
    }
  });
});
