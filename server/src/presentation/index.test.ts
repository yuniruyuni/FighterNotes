import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import type { Hono } from "hono";
import { RuntimeConfig } from "../config";
import type { Database } from "../infra/db";
import type { ILogger } from "../infra/logger/types";
import type { Context } from "../usecases/context";
import type {
  RuntimeServices,
  SharingRateLimitBucket,
} from "../usecases/services";
import { createApp } from "./index";

let app: Hono;
let staticDir: string;
const warningEntries: unknown[][] = [];
const logger: ILogger = {
  debug() {},
  info() {},
  warn(message, ...args) {
    warningEntries.push([message, ...args]);
  },
  error() {},
  child() {
    return logger;
  },
};

const INDEX_HTML =
  "<!DOCTYPE html><html><body>fighter-notes test</body></html>";

function testServices(): RuntimeServices {
  const counts = new Map<string, number>();
  return {
    publishedAnalysisSecurity: {
      generateShareId() {
        throw new Error("unexpected share id generation");
      },
      async hashDeletePassword() {
        throw new Error("unexpected password hash");
      },
      async verifyDeletePassword() {
        throw new Error("unexpected password verification");
      },
    },
    sharingRateLimit: {
      async consume(
        bucket: SharingRateLimitBucket,
        clientKey: string,
        limit: number,
      ) {
        const key = `${bucket}:${clientKey}`;
        const count = (counts.get(key) ?? 0) + 1;
        counts.set(key, count);
        return {
          allowed: count <= limit,
          retryAfterSeconds: count <= limit ? 0 : 60,
        };
      },
    },
  };
}

beforeAll(async () => {
  staticDir = mkdtempSync(join(tmpdir(), "fighter-static-"));
  writeFileSync(join(staticDir, "index.html"), INDEX_HTML);
  writeFileSync(join(staticDir, "app.js"), "console.log('ok');");
  writeFileSync(join(staticDir, "analyzer.wasm"), "wasm-test");
  // このsuiteはDBへ進む正常なmutationを呼ばないため、Contextはstubで十分。
  const config = RuntimeConfig.fromEnvironment({
    STATIC_DIR: staticDir,
    K_SERVICE: "fighter-test",
    TRUST_CLOUDFLARE_CONNECTING_IP: "true",
  });
  app = createApp({
    logger,
    config,
    services: testServices(),
  } as unknown as Context);
});

afterAll(() => {
  rmSync(staticDir, { recursive: true, force: true });
});

describe("/health", () => {
  test("200 で status ok を返す", async () => {
    const res = await app.request("/health");
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ status: "ok" });
  });
});

describe("/ready", () => {
  function appWithReadiness(result: boolean | Error): Hono {
    let queryCount = 0;
    const transaction = {
      async queryGet() {
        queryCount += 1;
        if (queryCount === 1) return { statement_timeout: "750ms" };
        return { compatible: result };
      },
    } as unknown as Database;
    const db = {
      async readTransaction<T>(fn: (tx: Database) => Promise<T>) {
        if (result instanceof Error) throw result;
        return fn(transaction);
      },
    } as Database;
    const config = RuntimeConfig.fromEnvironment({ STATIC_DIR: staticDir });
    return createApp({ logger, config, db } as unknown as Context);
  }

  test("DBとschemaが互換なら固定responseで200を返す", async () => {
    const res = await appWithReadiness(true).request("/ready");
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ status: "ready" });
  });

  test("DB異常は詳細を開示せず503にし、livenessは維持する", async () => {
    const failure = appWithReadiness(
      new Error("password authentication failed: secret-value"),
    );
    const ready = await failure.request("/ready");
    expect(ready.status).toBe(503);
    const body = await ready.text();
    expect(JSON.parse(body)).toEqual({ status: "unavailable" });
    expect(body).not.toContain("secret-value");

    const health = await failure.request("/health");
    expect(health.status).toBe(200);
    expect(await health.json()).toEqual({ status: "ok" });
  });
});

describe("CSP（解析機能の生命線 — 1 項目でも欠けると WASM/Worker/動画が全滅）", () => {
  const csp = async (): Promise<string> => {
    const res = await app.request("/health");
    return res.headers.get("content-security-policy") ?? "";
  };

  test("script-src に wasm-unsafe-eval（WASM インスタンス化に必須）", async () => {
    expect(await csp()).toMatch(/script-src[^;]*'wasm-unsafe-eval'/);
  });

  test("script-src は inline script を許可しない", async () => {
    expect(await csp()).not.toMatch(/script-src[^;]*'unsafe-inline'/);
  });

  test("worker-src に blob:（解析 Worker の生成に必須）", async () => {
    expect(await csp()).toMatch(/worker-src[^;]*blob:/);
  });

  test("media-src に blob:（アップロード動画のプレビュー再生に必須）", async () => {
    expect(await csp()).toMatch(/media-src[^;]*blob:/);
  });

  test("connect-src に blob: と data:", async () => {
    const v = await csp();
    expect(v).toMatch(/connect-src[^;]*blob:/);
    expect(v).toMatch(/connect-src[^;]*data:/);
  });

  test("font-src noneでWeb fontの取得を禁止する", async () => {
    expect(await csp()).toMatch(/font-src[^;]*'none'/);
  });

  test("frame-ancestors none", async () => {
    expect(await csp()).toMatch(/frame-ancestors[^;]*'none'/);
  });
});

describe("secure headers", () => {
  test("X-Frame-Options / X-Content-Type-Options / Referrer-Policy", async () => {
    const res = await app.request("/health");
    expect(res.headers.get("x-frame-options")).toBe("SAMEORIGIN");
    expect(res.headers.get("x-content-type-options")).toBe("nosniff");
    expect(res.headers.get("referrer-policy")).toBe(
      "strict-origin-when-cross-origin",
    );
  });
});

describe("静的配信 + SPA フォールバック", () => {
  test("index.html を配信し Cache-Control を付与する", async () => {
    const res = await app.request("/index.html");
    expect(res.status).toBe(200);
    expect(await res.text()).toBe(INDEX_HTML);
    expect(res.headers.get("cache-control")).toBe("no-store, must-revalidate");
  });

  test("JS/WASM は世代混在を避けるため保存させない", async () => {
    for (const path of ["/app.js", "/analyzer.wasm"]) {
      const res = await app.request(path);
      expect(res.status).toBe(200);
      expect(res.headers.get("cache-control")).toBe(
        "no-store, must-revalidate",
      );
    }
  });

  test("未知パスは index.html にフォールバック（SPA ルーティング）", async () => {
    const res = await app.request("/some/unknown/route");
    expect(res.status).toBe(200);
    expect(await res.text()).toBe(INDEX_HTML);
    expect(res.headers.get("cache-control")).toBe("no-store, must-revalidate");
  });
});

describe("共有結果ページ", () => {
  test("不正な短縮IDはSPAへ流さず同じ404ページを返す", async () => {
    for (const path of ["/s", "/s/", "/s/not-a-share-id", "/s/id/extra"]) {
      const res = await app.request(path);
      expect(res.status).toBe(404);
      const body = await res.text();
      expect(body).toContain("共有結果が見つかりません");
      expect(body).not.toBe(INDEX_HTML);
      expect(res.headers.get("cache-control")).toBe("no-store");
      expect(res.headers.get("x-content-type-options")).toBe("nosniff");
    }
  });
});

describe("共有rate-limit store過負荷の隔離", () => {
  test("共有requestをfail closedにしても静的配信とhealthを維持する", async () => {
    const services = testServices();
    services.sharingRateLimit.consume = async () => {
      throw new Error("private database overload detail");
    };
    const overloadedWarnings: unknown[][] = [];
    const overloadedLogger: ILogger = {
      ...logger,
      warn(message, ...args) {
        overloadedWarnings.push([message, ...args]);
      },
      child() {
        return this;
      },
    };
    const overloaded = createApp({
      logger: overloadedLogger,
      config: RuntimeConfig.fromEnvironment({
        STATIC_DIR: staticDir,
        K_SERVICE: "fighter-test",
        TRUST_CLOUDFLARE_CONNECTING_IP: "true",
      }),
      services,
    } as unknown as Context);

    const responses = await Promise.all(
      Array.from({ length: 25 }, (_, index) =>
        Promise.all([
          overloaded.request("/health"),
          overloaded.request("/app.js"),
          overloaded.request("/s/AAAAAAAAAAAAAAAAAAAAAA", {
            headers: { "CF-Connecting-IP": `203.0.113.${index + 1}` },
          }),
          overloaded.request("/api/trpc/publishedAnalysis.create", {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
              "CF-Connecting-IP": `203.0.113.${index + 1}`,
            },
            body: "{}",
          }),
        ]),
      ),
    );

    for (const [health, staticAsset, publicRead, create] of responses) {
      expect(health.status).toBe(200);
      expect(staticAsset.status).toBe(200);
      expect(publicRead.status).toBe(503);
      expect(publicRead.headers.get("cache-control")).toBe("no-store");
      expect(create.status).toBe(503);
    }
    expect(JSON.stringify(overloadedWarnings)).not.toContain(
      "private database overload detail",
    );
    expect(
      overloadedWarnings.every(
        ([message, fields]) =>
          message === "Published analysis rate limit unavailable" &&
          (fields as { bucket?: string }).bucket !== undefined,
      ),
    ).toBe(true);
  });

  test("緊急停止はrate-limit storeに依存せずcreateとpublic readを拒否する", async () => {
    const services = testServices();
    services.sharingRateLimit.consume = async () => {
      throw new Error("rate limit store must not be called");
    };
    const disabled = createApp({
      logger,
      config: RuntimeConfig.fromEnvironment({
        STATIC_DIR: staticDir,
        SHARE_RESULTS_ENABLED: "false",
      }),
      services,
    } as unknown as Context);

    const publicRead = await disabled.request("/s/AAAAAAAAAAAAAAAAAAAAAA");
    expect(publicRead.status).toBe(404);
    const create = await disabled.request(
      "/api/trpc/publishedAnalysis.create",
      {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: "{}",
      },
    );
    // input validation is intentionally earlier than the router's disabled 404,
    // but a rate-limit-store 503 must not mask emergency shutdown.
    expect(create.status).toBe(400);
  });
});

describe("tRPC マウント", () => {
  test("/api/trpc/* は SPA フォールバックせず tRPC が応答する", async () => {
    const res = await app.request("/api/trpc/nonexistent.procedure");
    // ルータ空なので NOT_FOUND だが、SPA の HTML ではなく tRPC の
    // エラー JSON が返ることでマウント自体を検証する
    const body = await res.text();
    expect(body).not.toContain("<!DOCTYPE html>");
    expect(body).toContain("error");
  });

  test("共有削除procedureを公開し厳密な入力を要求する", async () => {
    const res = await app.request("/api/trpc/publishedAnalysis.delete", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: "{}",
    });
    expect(res.status).toBe(400);
    expect(await res.text()).toContain("BAD_REQUEST");
  });

  test("POSTはJSON以外のContent-Typeを拒否する", async () => {
    for (const contentType of ["text/plain", "application/jsonp"]) {
      const res = await app.request("/api/trpc/publishedAnalysis.create", {
        method: "POST",
        headers: { "Content-Type": contentType },
        body: "{}",
      });
      expect(res.status).toBe(415);
    }
  });

  test("異なるOriginからのPOSTを拒否する", async () => {
    const res = await app.request("/api/trpc/publishedAnalysis.create", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Origin: "https://attacker.example",
      },
      body: "{}",
    });
    expect(res.status).toBe(403);
  });

  test("12KiBを超えるPOSTを413で拒否する", async () => {
    const res = await app.request("/api/trpc/publishedAnalysis.create", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ padding: "x".repeat(13 * 1024) }),
    });
    expect(res.status).toBe(413);
  });

  test("共有作成を接続元単位で制限しRetry-Afterを返す", async () => {
    const request = () =>
      app.request("/api/trpc/publishedAnalysis.create", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "CF-Connecting-IP": "203.0.113.40",
        },
        body: "{}",
      });

    for (let index = 0; index < 10; index += 1) {
      expect((await request()).status).not.toBe(429);
    }
    const limited = await request();
    expect(limited.status).toBe(429);
    expect(Number(limited.headers.get("retry-after"))).toBeGreaterThan(0);

    const anotherClient = await app.request(
      "/api/trpc/publishedAnalysis.create",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "CF-Connecting-IP": "203.0.113.41",
        },
        body: "{}",
      },
    );
    expect(anotherClient.status).not.toBe(429);
  });

  test("共有の作成・削除を含むtRPC batchを拒否する", async () => {
    const res = await app.request(
      "/api/trpc/publishedAnalysis.create,nonexistent.procedure?batch=1",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "CF-Connecting-IP": "203.0.113.42",
        },
        body: "{}",
      },
    );
    expect(res.status).toBe(400);
    expect(await res.json()).toEqual({
      error: "batch share mutation is not supported",
    });

    const deletion = await app.request(
      "/api/trpc/publishedAnalysis.delete,nonexistent.procedure?batch=1",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "CF-Connecting-IP": "203.0.113.44",
        },
        body: "{}",
      },
    );
    expect(deletion.status).toBe(400);
  });

  test("共有削除も接続元単位で制限する", async () => {
    const request = () =>
      app.request("/api/trpc/publishedAnalysis.delete", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "CF-Connecting-IP": "203.0.113.46",
        },
        body: "{}",
      });

    for (let index = 0; index < 10; index += 1) {
      expect((await request()).status).not.toBe(429);
    }
    const limited = await request();
    expect(limited.status).toBe(429);
    expect(Number(limited.headers.get("retry-after"))).toBeGreaterThan(0);
  });

  test("共有結果のrandom ID走査を接続元単位で制限する", async () => {
    const request = () =>
      app.request("/s/AAAAAAAAAAAAAAAAAAAAAA", {
        headers: { "CF-Connecting-IP": "203.0.113.45" },
      });
    for (let index = 0; index < 120; index += 1) {
      expect((await request()).status).not.toBe(429);
    }
    const limited = await request();
    expect(limited.status).toBe(429);
    expect(limited.headers.get("cache-control")).toBe("no-store");
    expect(Number(limited.headers.get("retry-after"))).toBeGreaterThan(0);
  });

  test("共有入力の検証失敗は値を含めずfield pathだけを記録する", async () => {
    warningEntries.length = 0;
    const privateValue = "<script>private input</script>";
    const res = await app.request("/api/trpc/publishedAnalysis.create", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "CF-Connecting-IP": "203.0.113.43",
      },
      body: JSON.stringify({
        analysis: {
          rulesetVersion: 999,
          comment: privateValue,
        },
        deletePassword: "fighter-notes-delete-key",
      }),
    });

    expect(res.status).toBe(400);
    const entry = warningEntries.find(
      ([message]) => message === "Published analysis input rejected",
    );
    expect(entry).toBeDefined();
    expect(JSON.stringify(entry)).not.toContain(privateValue);
    expect(entry?.[1]).toMatchObject({
      procedure: "publishedAnalysis.create",
      issues: expect.arrayContaining([
        expect.objectContaining({ path: "analysis.rulesetVersion" }),
      ]),
    });
  });
});
