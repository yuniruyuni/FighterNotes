import { describe, expect, test } from "bun:test";
import { AnalysisRuntime } from "./runtime.js";

describe("analysis runtime", () => {
  test("Secure ContextでVideoDecoderがあれば解析できる", () => {
    expect(
      AnalysisRuntime.evaluate({
        secureContext: true,
        hasVideoDecoder: true,
        origin: "https://fighter.example",
      }),
    ).toEqual({ available: true });
  });

  test("VideoDecoder非対応ブラウザでは解析を開始しない", () => {
    const readiness = AnalysisRuntime.evaluate({
      secureContext: true,
      hasVideoDecoder: false,
      origin: "http://localhost:3001",
    });

    expect(readiness.available).toBe(false);
    if (readiness.available) throw new Error("expected unavailable runtime");
    expect(readiness.reason).toBe(
      "このブラウザは動画解析に必要なWebCodecs VideoDecoderに対応していません。" +
        " ブラウザを最新版に更新するか、対応ブラウザで開いてください。",
    );
  });

  test("信頼されないHTTP originでは解析を開始しない", () => {
    const readiness = AnalysisRuntime.evaluate({
      secureContext: false,
      hasVideoDecoder: false,
      origin: "http://192.168.10.3:3001",
    });

    expect(readiness.available).toBe(false);
    if (readiness.available) throw new Error("expected unavailable runtime");
    expect(readiness.reason).toContain("HTTPSまたはlocalhost");
    expect(readiness.reason).toContain("http://192.168.10.3:3001");
  });
});
