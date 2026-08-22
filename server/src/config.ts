import type { ShareCreateLimits } from "./models/published-analysis";

const DEFAULT_PUBLIC_BASE_URL = "https://fighter.yuniruyuni.net";
const DEFAULT_SHARE_RETENTION_DAYS = 30;
const DEFAULT_SHARE_CREATE_RATE_LIMIT = 10;
const DEFAULT_SHARE_DELETE_RATE_LIMIT = 10;
const DEFAULT_SHARE_GET_RATE_LIMIT = 120;
const DEFAULT_ARGON2_CONCURRENCY = 2;
const DEFAULT_ARGON2_QUEUE_LIMIT = 8;
const DEFAULT_ARGON2_WAIT_MILLIS = 250;
const DEFAULT_SHARE_DAILY_CREATE_LIMIT = 1_000;
const DEFAULT_SHARE_ACTIVE_LIMIT = 50_000;
const DEFAULT_SHARE_STORAGE_LIMIT_BYTES = 1024 * 1024 * 1024;
const DEFAULT_CLEANUP_BATCH_SIZE = 500;
const DEFAULT_CLEANUP_MAX_BATCHES = 1_000;

type Environment = Readonly<Record<string, string | undefined>>;

export interface CleanupSettings {
  batchSize: number;
  maxBatches: number;
  retentionDays: number;
}

export interface DatabaseSettings {
  host: string;
  port: number;
  user: string;
  password: string;
  database: string;
  max: number;
  connectionTimeoutMillis: number;
  idleTimeoutMillis: number;
  statementTimeoutMillis: number;
  lockTimeoutMillis: number;
  idleInTransactionSessionTimeoutMillis: number;
}

export interface RuntimeConfig {
  port: number;
  staticDir: string;
  publicBaseUrl: URL;
  sharing: {
    enabled: boolean;
    retentionDays: number;
    createRateLimit: number;
    deleteRateLimit: number;
    getRateLimit: number;
    trustCloudflareConnectingIp: boolean;
    argon2: {
      concurrency: number;
      queueLimit: number;
      waitMillis: number;
    };
    createLimits: ShareCreateLimits;
  };
  cleanup: CleanupSettings;
  database: DatabaseSettings;
}

export const RuntimeConfig = {
  fromEnvironment(environment: Environment = process.env): RuntimeConfig {
    const retentionDays = integerSetting(
      environment,
      "SHARE_RETENTION_DAYS",
      DEFAULT_SHARE_RETENTION_DAYS,
      1,
      3650,
    );
    const configuredPublicBaseUrl = publicBaseUrl(environment.PUBLIC_BASE_URL);
    const trustCloudflareConnectingIp = booleanSetting(
      environment,
      "TRUST_CLOUDFLARE_CONNECTING_IP",
      false,
    );
    // CF-Connecting-IP を信じてよいのは、アプリへ Cloudflare 経由でしか
    // 到達できないときだけ。直接叩ける場所でこれを信じると、送信元 IP を
    // 詐称してレート制限を回避できてしまう。
    //
    // かつては K_SERVICE の有無で「Cloud Run 上か」を見ていた。ingress を
    // 絞った Cloud Run ではそれが到達経路の保証になっていたが、VPS 上でも
    // 同じ性質は成り立つ (コンテナは loopback にだけ束縛され、その手前に
    // HAProxy と cloudflared がいる)。K_SERVICE はその性質の代理でしかなく、
    // 環境を移すと成り立たなくなるので見ない。
    //
    // 到達経路そのものはアプリからは確かめようがない。この変数を true に
    // すること自体が運用側の宣言であり、公開 URL が HTTPS であることだけを
    // 取り違えの歯止めとして残す。
    if (
      trustCloudflareConnectingIp &&
      configuredPublicBaseUrl.protocol !== "https:"
    ) {
      throw new Error(
        "TRUST_CLOUDFLARE_CONNECTING_IP requires an HTTPS public URL",
      );
    }
    return {
      port: integerSetting(environment, "PORT", 3000, 1, 65_535),
      staticDir: environment.STATIC_DIR ?? "./static",
      publicBaseUrl: configuredPublicBaseUrl,
      sharing: {
        enabled: environment.SHARE_RESULTS_ENABLED !== "false",
        retentionDays,
        createRateLimit: integerSetting(
          environment,
          "SHARE_CREATE_RATE_LIMIT_PER_MINUTE",
          DEFAULT_SHARE_CREATE_RATE_LIMIT,
          1,
          1_000,
        ),
        deleteRateLimit: integerSetting(
          environment,
          "SHARE_DELETE_RATE_LIMIT_PER_MINUTE",
          DEFAULT_SHARE_DELETE_RATE_LIMIT,
          1,
          1_000,
        ),
        getRateLimit: integerSetting(
          environment,
          "SHARE_GET_RATE_LIMIT_PER_MINUTE",
          DEFAULT_SHARE_GET_RATE_LIMIT,
          1,
          100_000,
        ),
        trustCloudflareConnectingIp,
        argon2: {
          concurrency: integerSetting(
            environment,
            "SHARE_ARGON2_CONCURRENCY",
            DEFAULT_ARGON2_CONCURRENCY,
            1,
            16,
          ),
          queueLimit: integerSetting(
            environment,
            "SHARE_ARGON2_QUEUE_LIMIT",
            DEFAULT_ARGON2_QUEUE_LIMIT,
            0,
            1_000,
          ),
          waitMillis: integerSetting(
            environment,
            "SHARE_ARGON2_WAIT_MS",
            DEFAULT_ARGON2_WAIT_MILLIS,
            1,
            10_000,
          ),
        },
        createLimits: {
          dailyCreates: integerSetting(
            environment,
            "SHARE_DAILY_CREATE_LIMIT",
            DEFAULT_SHARE_DAILY_CREATE_LIMIT,
            1,
            10_000_000,
          ),
          activeRows: integerSetting(
            environment,
            "SHARE_ACTIVE_LIMIT",
            DEFAULT_SHARE_ACTIVE_LIMIT,
            1,
            10_000_000,
          ),
          storageBytes: integerSetting(
            environment,
            "SHARE_STORAGE_LIMIT_BYTES",
            DEFAULT_SHARE_STORAGE_LIMIT_BYTES,
            1024 * 1024,
            Number.MAX_SAFE_INTEGER,
          ),
        },
      },
      cleanup: {
        batchSize: integerSetting(
          environment,
          "CLEANUP_BATCH_SIZE",
          DEFAULT_CLEANUP_BATCH_SIZE,
          1,
          10_000,
        ),
        maxBatches: integerSetting(
          environment,
          "CLEANUP_MAX_BATCHES",
          DEFAULT_CLEANUP_MAX_BATCHES,
          1,
          10_000,
        ),
        retentionDays,
      },
      database: databaseSettings(environment),
    };
  },
};

function databaseSettings(environment: Environment): DatabaseSettings {
  const application = environment.DB_APP_NAME ?? "template";
  return {
    host: environment.PGHOST ?? "localhost",
    port: integerSetting(environment, "PGPORT", 5432, 1, 65_535),
    // DB_USER / DB_NAME も見る。yunirun は接続先をこの名前で渡すため。
    //
    // これが無いと、接続ユーザが DB_APP_NAME (= DB 名 = owner ロール名) へ
    // フォールバックする。owner は DDL 用のロールで、runtime が名乗ってよい
    // ものではない。bin/migrate.sh が PGUSER="$DB_USER" と書いているのと
    // 同じ規約に揃える。
    //
    // PGUSER / PGDATABASE を先に見るのは、Cloud Run 側がそちらを設定して
    // いるため。移行が済めば環境変数はどちらか一方しか来ない。
    user: environment.PGUSER ?? environment.DB_USER ?? application,
    password: environment.PGPASSWORD ?? environment.DB_PASSWORD ?? "template",
    database: environment.PGDATABASE ?? environment.DB_NAME ?? application,
    max: integerSetting(environment, "PGPOOL_MAX", 5, 1, 20),
    connectionTimeoutMillis: integerSetting(
      environment,
      "PG_CONNECTION_TIMEOUT_MS",
      5_000,
      100,
      60_000,
    ),
    idleTimeoutMillis: integerSetting(
      environment,
      "PG_IDLE_TIMEOUT_MS",
      30_000,
      1_000,
      600_000,
    ),
    statementTimeoutMillis: integerSetting(
      environment,
      "PG_STATEMENT_TIMEOUT_MS",
      15_000,
      100,
      300_000,
    ),
    lockTimeoutMillis: integerSetting(
      environment,
      "PG_LOCK_TIMEOUT_MS",
      5_000,
      100,
      300_000,
    ),
    idleInTransactionSessionTimeoutMillis: integerSetting(
      environment,
      "PG_IDLE_IN_TRANSACTION_SESSION_TIMEOUT_MS",
      30_000,
      1_000,
      600_000,
    ),
  };
}

function booleanSetting(
  environment: Environment,
  name: string,
  fallback: boolean,
): boolean {
  const raw = environment[name];
  if (raw === undefined) return fallback;
  if (raw === "true") return true;
  if (raw === "false") return false;
  throw new Error(`${name} must be true or false`);
}

function publicBaseUrl(configured = DEFAULT_PUBLIC_BASE_URL): URL {
  const url = new URL(configured);
  const localHttp = url.protocol === "http:" && url.hostname === "localhost";
  if (url.protocol !== "https:" && !localHttp) {
    throw new Error("PUBLIC_BASE_URL must use HTTPS outside localhost");
  }
  url.pathname = "/";
  url.search = "";
  url.hash = "";
  return url;
}

function integerSetting(
  environment: Environment,
  name: string,
  fallback: number,
  minimum: number,
  maximum: number,
): number {
  const raw = environment[name];
  if (raw === undefined) return fallback;
  const value = Number(raw);
  if (!Number.isSafeInteger(value) || value < minimum || value > maximum) {
    throw new Error(`${name} must be an integer from ${minimum} to ${maximum}`);
  }
  return value;
}
