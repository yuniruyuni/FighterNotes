import { describe, expect, test } from "bun:test";
import { RuntimeConfig } from "./config";

describe("RuntimeConfig", () => {
  test("空の環境から一貫した既定値を作る", () => {
    const config = RuntimeConfig.fromEnvironment({});
    expect(config).toMatchObject({
      port: 3000,
      staticDir: "./static",
      sharing: {
        enabled: true,
        retentionDays: 30,
        createRateLimit: 10,
        deleteRateLimit: 10,
        getRateLimit: 120,
        trustCloudflareConnectingIp: false,
        argon2: { concurrency: 2, queueLimit: 8, waitMillis: 250 },
      },
      cleanup: { retentionDays: 30 },
      database: { host: "localhost", port: 5432, max: 5 },
    });
  });

  test("Cloudflare client IPをCloud Run内のHTTPS構成だけで信頼する", () => {
    expect(() =>
      RuntimeConfig.fromEnvironment({
        TRUST_CLOUDFLARE_CONNECTING_IP: "true",
      }),
    ).toThrow("requires Cloud Run");
    expect(() =>
      RuntimeConfig.fromEnvironment({
        TRUST_CLOUDFLARE_CONNECTING_IP: "true",
        K_SERVICE: "fighter",
        PUBLIC_BASE_URL: "http://localhost:3000",
      }),
    ).toThrow("HTTPS");
    expect(
      RuntimeConfig.fromEnvironment({
        TRUST_CLOUDFLARE_CONNECTING_IP: "true",
        K_SERVICE: "fighter",
      }).sharing.trustCloudflareConnectingIp,
    ).toBe(true);
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
