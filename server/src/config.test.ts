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

  test("yunirunが渡すDB_USER/DB_NAMEで接続先を決める", () => {
    // yunirun は接続ユーザを DB_USER で渡す。これを見ないと DB_APP_NAME
    // (= DB 名 = owner ロール名) へフォールバックし、DDL 用の owner ロールを
    // runtime が名乗ることになる。
    const config = RuntimeConfig.fromEnvironment({
      DB_APP_NAME: "fighter",
      DB_USER: "fighter_app",
      DB_NAME: "fighter",
    });
    expect(config.database).toMatchObject({
      user: "fighter_app",
      database: "fighter",
    });
  });

  test("PGUSERがあればそちらを優先する", () => {
    // Cloud Run 側は PGUSER を設定している。移行中は両方来うる。
    const config = RuntimeConfig.fromEnvironment({
      DB_APP_NAME: "fighter",
      PGUSER: "fighter_app",
      DB_USER: "ignored",
    });
    expect(config.database.user).toBe("fighter_app");
  });

  test("DB_USERが無ければDB_APP_NAMEへ落ちる", () => {
    const config = RuntimeConfig.fromEnvironment({ DB_APP_NAME: "fighter" });
    expect(config.database.user).toBe("fighter");
  });

  test("Cloudflare client IPをHTTPS構成でだけ信頼する", () => {
    // 公開 URL が HTTPS でなければ、取り違えとみなして起動を止める。
    expect(() =>
      RuntimeConfig.fromEnvironment({
        TRUST_CLOUDFLARE_CONNECTING_IP: "true",
        PUBLIC_BASE_URL: "http://localhost:3000",
      }),
    ).toThrow("HTTPS");
    expect(
      RuntimeConfig.fromEnvironment({
        TRUST_CLOUDFLARE_CONNECTING_IP: "true",
        PUBLIC_BASE_URL: "https://fighter.yuniruyuni.net",
      }).sharing.trustCloudflareConnectingIp,
    ).toBe(true);
  });

  test("Cloud Runでなくても信頼できる", () => {
    // K_SERVICE は Cloud Run が注入する変数。VPS 上では存在しないので、
    // これを要求していると yunirun 上でアプリが起動しない。到達経路の
    // 保証は環境変数ではなく配置 (loopback 束縛 + HAProxy + cloudflared)
    // が担う。
    expect(
      RuntimeConfig.fromEnvironment({
        TRUST_CLOUDFLARE_CONNECTING_IP: "true",
        PUBLIC_BASE_URL: "https://fighter.yuniruyuni.net",
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
