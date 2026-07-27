import { describe, expect, test } from "bun:test";
import { RuntimeConfig } from "./config";

describe("RuntimeConfig", () => {
  test("空の環境から一貫した既定値を作る", () => {
    const config = RuntimeConfig.fromEnvironment({});
    expect(config).toMatchObject({
      port: 3000,
      staticDir: "./static",
      sharing: { enabled: true, retentionDays: 30 },
      cleanup: { retentionDays: 30 },
      database: { host: "localhost", port: 5432, max: 5 },
    });
  });

  test("解析後の設定は環境オブジェクト変更の影響を受けない", () => {
    const environment: Record<string, string> = {
      SHARE_RESULTS_ENABLED: "true",
    };
    const config = RuntimeConfig.fromEnvironment(environment);
    environment.SHARE_RESULTS_ENABLED = "false";
    expect(config.sharing.enabled).toBe(true);
  });

  test("localhost以外のHTTP公開URLと範囲外の整数を拒否する", () => {
    expect(() =>
      RuntimeConfig.fromEnvironment({ PUBLIC_BASE_URL: "http://example.com" }),
    ).toThrow("HTTPS");
    expect(() => RuntimeConfig.fromEnvironment({ PORT: "70000" })).toThrow(
      "PORT",
    );
  });
});
