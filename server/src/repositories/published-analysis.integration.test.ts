import {
  afterAll,
  beforeAll,
  beforeEach,
  describe,
  expect,
  test,
} from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { createContext } from "../context";
import { inspectDatabaseReadiness, PgDatabase, sql } from "../infra/db";
import { PostgresSharingRateLimit } from "../infra/db/shared-rate-limit";
import { ConsoleLogger } from "../infra/logger";
import type { ILogger } from "../infra/logger/types";
import {
  createPersistablePublishedAnalysis,
  createPublishedAnalysisContent,
  PUBLISHED_ANALYSIS_CREATE_LOCK,
  PublishedAnalysisCreateEvent,
  PublishedAnalysisLifecycle,
  PublishedAnalysisStorageUsage,
  startOfUtcDay,
} from "../models/published-analysis";
import { createApp } from "../presentation";
import { appRouter } from "../presentation/trpc/routers";
import { createPublishedAnalysisUsecase } from "../usecases/published-analysis";
import { createRawRepos } from ".";
import { createDbReadCtx, createDbWriteCtx } from "./common/capability";
import { PublishedAnalysisRepository } from "./published-analysis/postgres";
import { registerPublishedAnalysisRepositoryIntegrationTests } from "./published-analysis/postgres/integration-suite";
import { PublishedAnalysisCreateEventRepository } from "./published-analysis-create-event/postgres";
import { registerPublishedAnalysisCreateEventRepositoryIntegrationTests } from "./published-analysis-create-event/postgres/integration-suite";
import { PublishedAnalysisLifecycleRepository } from "./published-analysis-lifecycle/postgres";
import { registerPublishedAnalysisLifecycleRepositoryIntegrationTests } from "./published-analysis-lifecycle/postgres/integration-suite";
import { registerPublishedAnalysisStorageUsageRepositoryIntegrationTests } from "./published-analysis-storage-usage/postgres/integration-suite";
import {
  candidate,
  createInput,
  DELETE_PASSWORD,
  DELETE_PASSWORD_HASH,
  FIXTURE_ID,
} from "./test-support/published-analysis";
import { registerTransactionLockRepositoryIntegrationTests } from "./transaction-lock/postgres/integration-suite";

const databaseUrl = process.env.TEST_DATABASE_URL;
const integration = databaseUrl ? describe : describe.skip;
integration("Postgres repositories + published analysis flow", () => {
  let db: PgDatabase;
  const repository = new PublishedAnalysisRepository();
  const lifecycleRepository = new PublishedAnalysisLifecycleRepository();
  const createEventRepository = new PublishedAnalysisCreateEventRepository();

  beforeAll(async () => {
    db = new PgDatabase({ connectionString: databaseUrl });
    const roles = readFileSync(
      join(import.meta.dir, "../../../schema/tables/000_roles.sql"),
      "utf8",
    );
    const schema = readFileSync(
      join(import.meta.dir, "../../../schema/tables/published_analyses.sql"),
      "utf8",
    );
    await db.queryRun(sql.raw(roles));
    await db.queryRun(sql.raw(schema));
  });

  beforeEach(async () => {
    await db.queryRun(
      sql.raw(
        "TRUNCATE published_analysis_rate_limits, published_analysis_create_events, published_analyses CASCADE",
      ),
    );
  });

  afterAll(async () => {
    await db?.close();
  });

  async function appRoleReadiness(): Promise<boolean> {
    return db.readTransaction(async (tx) => {
      await tx.queryRun(sql.raw("SET LOCAL ROLE fighter_app"));
      return inspectDatabaseReadiness(tx);
    });
  }

  test("app roleのread-only queryでschema versionと必要grantを確認する", async () => {
    expect(await appRoleReadiness()).toBe(true);
  });

  test("app roleの必要grant欠落をreadiness異常にする", async () => {
    await db.queryRun(
      sql.raw("REVOKE SELECT ON published_analysis_findings FROM fighter_app"),
    );
    try {
      expect(await appRoleReadiness()).toBe(false);
    } finally {
      await db.queryRun(
        sql.raw("GRANT SELECT ON published_analysis_findings TO fighter_app"),
      );
    }
  });

  test("schema version contract欠落をreadiness異常にする", async () => {
    await db.queryRun(
      sql.raw(`ALTER TABLE published_analyses
        DROP CONSTRAINT published_analyses_schema_version_check`),
    );
    try {
      expect(await appRoleReadiness()).toBe(false);
    } finally {
      await db.queryRun(
        sql.raw(`ALTER TABLE published_analyses
          ADD CONSTRAINT published_analyses_schema_version_check
          CHECK (schema_version = 1)`),
      );
    }
  });

  test("2 instanceとcold startで同じrate limit bucketをatomicに共有する", async () => {
    const firstInstance = new PostgresSharingRateLimit(db);
    const secondInstance = new PostgresSharingRateLimit(db);
    const attempts = await Promise.all(
      Array.from({ length: 20 }, (_, index) =>
        (index % 2 === 0 ? firstInstance : secondInstance).consume(
          "create",
          "203.0.113.20",
          10,
        ),
      ),
    );
    expect(attempts.filter((decision) => decision.allowed)).toHaveLength(10);
    expect(
      await new PostgresSharingRateLimit(db).consume(
        "create",
        "203.0.113.20",
        10,
      ),
    ).toMatchObject({ allowed: false });
    expect(await secondInstance.consume("delete", "203.0.113.20", 10)).toEqual({
      allowed: true,
      retryAfterSeconds: 0,
    });
    expect(
      await firstInstance.consume("public_read", "203.0.113.20", 120),
    ).toEqual({ allowed: true, retryAfterSeconds: 0 });

    const stored = await db.queryGet<{ client_key_hash: string }>(
      sql.raw(`SELECT client_key_hash
        FROM published_analysis_rate_limits
        WHERE bucket = 'create'`),
    );
    expect(stored?.client_key_hash).toHaveLength(64);
    expect(stored?.client_key_hash).not.toContain("203.0.113.20");
  });

  registerPublishedAnalysisRepositoryIntegrationTests(() => db);
  registerPublishedAnalysisCreateEventRepositoryIntegrationTests(() => db);
  registerPublishedAnalysisLifecycleRepositoryIntegrationTests(() => db);
  registerPublishedAnalysisStorageUsageRepositoryIntegrationTests(() => db);
  registerTransactionLockRepositoryIntegrationTests(() => db);

  test("retentionを超えたbatch cleanupが子行もCASCADE削除する", async () => {
    const content = createPublishedAnalysisContent(candidate());
    if (!content.ok) throw new Error("fixture is invalid");
    const created = createPersistablePublishedAnalysis({
      id: FIXTURE_ID,
      content: content.value,
      deletePasswordHash: DELETE_PASSWORD_HASH,
      now: new Date("2020-01-01T00:00:00.000Z"),
      retentionDays: 365,
    });
    await repository.create(createDbWriteCtx(db), created.analysis);

    const candidates = await lifecycleRepository.list(
      createDbReadCtx(db),
      PublishedAnalysisLifecycle.ExpiredAt(
        new Date("2020-01-03T00:00:00.000Z"),
      ).or(
        PublishedAnalysisLifecycle.CreatedAtOrBefore(
          new Date("2020-01-02T00:00:00.000Z"),
        ),
      ),
      { limit: 500, sort: PublishedAnalysisLifecycle.defaultSort },
    );
    expect(candidates.items.map((item) => item.id)).toEqual([
      created.analysis.id,
    ]);
    expect(
      await lifecycleRepository.delete(
        createDbWriteCtx(db),
        PublishedAnalysisLifecycle.ByIds(
          ...candidates.items.map((item) => item.id),
        ),
      ),
    ).toBe(1);
    for (const table of [
      "published_analyses",
      "published_analysis_findings",
      "published_analysis_tactics",
    ]) {
      const row = await db.queryGet<{ count: string }>(
        sql.raw(`SELECT count(*) AS count FROM ${table}`),
      );
      expect(row?.count).toBe("0");
    }
  });

  test("日次quota eventは結果rowと独立して単調増加する", async () => {
    const now = new Date("2026-07-14T00:00:00.000Z");
    const limits = {
      dailyCreates: 1,
      activeRows: 10,
      storageBytes: Number.MAX_SAFE_INTEGER,
    };

    const firstContext = createContext(
      db,
      new ConsoleLogger({ minLevel: "error" }),
    );
    firstContext.now = now;
    const first = await createPublishedAnalysisUsecase(
      candidate(),
      DELETE_PASSWORD,
      30,
      limits,
    ).run(firstContext);
    expect(first.ok).toBe(true);
    if (!first.ok) return;

    await lifecycleRepository.delete(
      createDbWriteCtx(db),
      PublishedAnalysisLifecycle.ByIds(first.value.id),
    );
    expect(
      await createEventRepository.count(
        createDbReadCtx(db),
        PublishedAnalysisCreateEvent.CreatedAtOrAfter(now),
      ),
    ).toBe(1);

    const secondContext = createContext(
      db,
      new ConsoleLogger({ minLevel: "error" }),
    );
    secondContext.now = new Date("2026-07-14T01:00:00.000Z");
    const second = await createPublishedAnalysisUsecase(
      candidate(),
      DELETE_PASSWORD,
      30,
      limits,
    ).run(secondContext);
    expect(second).toMatchObject({
      ok: false,
      error: { code: "RESOURCE_LIMIT" },
    });
  });

  test("競合createでもadvisory lockにより日次quotaを超えない", async () => {
    const now = new Date("2026-07-14T00:00:00.000Z");
    const limits = {
      dailyCreates: 1,
      activeRows: 10,
      storageBytes: Number.MAX_SAFE_INTEGER,
    };

    const results = await Promise.all(
      [0, 1].map(async () => {
        const context = createContext(
          db,
          new ConsoleLogger({ minLevel: "error" }),
        );
        context.now = now;
        return createPublishedAnalysisUsecase(
          candidate(),
          DELETE_PASSWORD,
          30,
          limits,
        ).run(context);
      }),
    );
    expect(results.filter((result) => result.ok)).toHaveLength(1);
    expect(results.filter((result) => !result.ok)).toHaveLength(1);
    const row = await db.queryGet<{ count: string }>(
      sql.raw("SELECT count(*) AS count FROM published_analyses"),
    );
    expect(row?.count).toBe("1");
  });

  test("relation容量のhard limitはinsert前に拒否する", async () => {
    const context = createContext(db, new ConsoleLogger({ minLevel: "error" }));
    const result = await createPublishedAnalysisUsecase(
      candidate(),
      DELETE_PASSWORD,
      30,
      {
        dailyCreates: 1,
        activeRows: 1,
        storageBytes: 1,
      },
    ).run(context);
    expect(result).toMatchObject({
      ok: false,
      error: { code: "RESOURCE_LIMIT" },
    });
    const row = await db.queryGet<{ count: string }>(
      sql.raw("SELECT count(*) AS count FROM published_analyses"),
    );
    expect(row?.count).toBe("0");
  });

  test("app roleはruntimeとcleanupに必要なDMLだけを持つ", async () => {
    const privileges = await db.queryGet<{
      app_parent_select: boolean;
      app_parent_insert: boolean;
      app_parent_update: boolean;
      app_parent_delete: boolean;
      app_events_select: boolean;
      app_events_insert: boolean;
      app_events_update: boolean;
      app_events_delete: boolean;
      app_limits_select: boolean;
      app_limits_insert: boolean;
      app_limits_update: boolean;
      app_limits_delete: boolean;
    }>(
      sql.raw(`
      SELECT
        has_table_privilege('fighter_app', 'published_analyses', 'SELECT')
          AS app_parent_select,
        has_table_privilege('fighter_app', 'published_analyses', 'INSERT')
          AS app_parent_insert,
        has_table_privilege('fighter_app', 'published_analyses', 'UPDATE')
          AS app_parent_update,
        has_table_privilege('fighter_app', 'published_analyses', 'DELETE')
          AS app_parent_delete,
        has_table_privilege(
          'fighter_app',
          'published_analysis_create_events',
          'SELECT'
        ) AS app_events_select,
        has_table_privilege(
          'fighter_app',
          'published_analysis_create_events',
          'INSERT'
        ) AS app_events_insert,
        has_table_privilege(
          'fighter_app',
          'published_analysis_create_events',
          'UPDATE'
        ) AS app_events_update,
        has_table_privilege(
          'fighter_app',
          'published_analysis_create_events',
          'DELETE'
        ) AS app_events_delete,
        has_table_privilege(
          'fighter_app',
          'published_analysis_rate_limits',
          'SELECT'
        ) AS app_limits_select,
        has_table_privilege(
          'fighter_app',
          'published_analysis_rate_limits',
          'INSERT'
        ) AS app_limits_insert,
        has_table_privilege(
          'fighter_app',
          'published_analysis_rate_limits',
          'UPDATE'
        ) AS app_limits_update,
        has_table_privilege(
          'fighter_app',
          'published_analysis_rate_limits',
          'DELETE'
        ) AS app_limits_delete
    `),
    );
    expect(privileges).toEqual({
      app_parent_select: true,
      app_parent_insert: true,
      app_parent_update: false,
      app_parent_delete: true,
      app_events_select: true,
      app_events_insert: true,
      app_events_update: false,
      app_events_delete: true,
      app_limits_select: true,
      app_limits_insert: true,
      app_limits_update: true,
      app_limits_delete: false,
    });
  });

  test("app roleでSpec CRUDとquota lockを実行できる", async () => {
    const content = createPublishedAnalysisContent(candidate());
    if (!content.ok) throw new Error("fixture is invalid");
    const now = new Date("2026-07-14T00:00:00.000Z");
    const created = createPersistablePublishedAnalysis({
      id: FIXTURE_ID,
      content: content.value,
      deletePasswordHash: DELETE_PASSWORD_HASH,
      now,
      retentionDays: 30,
    });
    const repos = createRawRepos();

    await db.transaction(async (tx) => {
      await tx.queryRun(sql.raw("SET LOCAL ROLE fighter_app"));
      const ctx = createDbWriteCtx(tx);

      await repos.transactionLock.acquire(ctx, PUBLISHED_ANALYSIS_CREATE_LOCK);
      expect(
        await repos.publishedAnalysisCreateEvent.count(
          ctx,
          PublishedAnalysisCreateEvent.CreatedAtOrAfter(startOfUtcDay(now)),
        ),
      ).toBe(0);
      expect(
        await repos.publishedAnalysisLifecycle.count(
          ctx,
          PublishedAnalysisLifecycle.ActiveAt(now),
        ),
      ).toBe(0);
      expect(
        await repos.publishedAnalysisStorageUsage.get(
          ctx,
          PublishedAnalysisStorageUsage.Current(),
        ),
      ).toMatchObject({ bytes: expect.any(Number) });

      await repos.publishedAnalysisCreateEvent.create(ctx, {
        analysisId: created.analysis.id,
        createdAt: now,
      });
      await repos.publishedAnalysis.create(ctx, created.analysis);
      const page = await repos.publishedAnalysisLifecycle.list(
        ctx,
        PublishedAnalysisLifecycle.ById(created.analysis.id),
        { limit: 1, sort: PublishedAnalysisLifecycle.defaultSort },
      );
      expect(page.items).toHaveLength(1);
      expect(
        await repos.publishedAnalysisLifecycle.delete(
          ctx,
          PublishedAnalysisLifecycle.ByIds(created.analysis.id),
        ),
      ).toBe(1);
      expect(
        await repos.publishedAnalysisCreateEvent.delete(
          ctx,
          PublishedAnalysisCreateEvent.ByAnalysisId(created.analysis.id),
        ),
      ).toBe(1);
    });
  });

  test("transaction失敗時は3テーブルすべてrollbackする", async () => {
    const content = createPublishedAnalysisContent(candidate());
    if (!content.ok) throw new Error("fixture is invalid");
    const created = createPersistablePublishedAnalysis({
      id: FIXTURE_ID,
      content: content.value,
      deletePasswordHash: DELETE_PASSWORD_HASH,
      now: new Date(),
      retentionDays: 365,
    });

    await expect(
      db.transaction(async (tx) => {
        await repository.create(createDbWriteCtx(tx), created.analysis);
        throw new Error("rollback marker");
      }),
    ).rejects.toThrow("rollback marker");
    const row = await db.queryGet<{ count: string }>(
      sql.raw("SELECT count(*) AS count FROM published_analyses"),
    );
    expect(row?.count).toBe("0");
  });

  test("DB制約が未知文字列を直接挿入しても拒否する", async () => {
    await expect(
      db.queryRun(sql`
        INSERT INTO published_analyses (
          id, schema_version, ruleset_version, presentation_revision,
          own_character, opponent_character,
          rounds_detected, rounds_won, rounds_lost, rounds_unresolved,
          created_at, expires_at
        ) VALUES (
          ${"aaaaaaaaaaaaaaaaaaaaaa"}, 1, 3, 1,
          ${"<script>alert(1)</script>"}, ${"LUKE"},
          0, 0, 0, 0, now(), now() + interval '1 day'
        )
      `),
    ).rejects.toThrow();
  });

  test("最大モデルの実データ行サイズが8KiB未満", async () => {
    const content = createPublishedAnalysisContent(candidate(true));
    if (!content.ok) throw new Error("fixture is invalid");
    const created = createPersistablePublishedAnalysis({
      id: FIXTURE_ID,
      content: content.value,
      deletePasswordHash: DELETE_PASSWORD_HASH,
      now: new Date(),
      retentionDays: 365,
    });
    await repository.create(createDbWriteCtx(db), created.analysis);
    const row = await db.queryGet<{ bytes: number }>(sql`
      SELECT
        pg_column_size(a.*) + pg_column_size(t.*) +
        COALESCE((
          SELECT sum(pg_column_size(f.*))
          FROM published_analysis_findings f
          WHERE f.analysis_id = a.id
        ), 0)::integer AS bytes
      FROM published_analyses a
      INNER JOIN published_analysis_tactics t ON t.analysis_id = a.id
      WHERE a.id = ${created.analysis.id}
    `);
    expect(row?.bytes).toBeLessThan(8 * 1024);
  });

  test("tRPCが削除キー付き共有を作り15秒cacheで配信・削除する", async () => {
    const logger = new CapturingLogger();
    const context = createContext(db, logger);
    context.now = new Date();
    const caller = appRouter.createCaller(context);
    const created = await caller.publishedAnalysis.create(createInput());
    expect(created.url).toMatch(
      /^https:\/\/fighter\.yuniruyuni\.net\/s\/[A-Za-z0-9_-]{22}$/,
    );
    expect(new Date(created.expiresAt).getTime()).toBeGreaterThan(
      context.now.getTime(),
    );
    expect(created).not.toHaveProperty("deletePassword");

    const id = new URL(created.url).pathname.split("/").pop();
    if (!id) throw new Error("share id missing");
    const app = createApp(context);
    const page = await app.request(`/s/${id}`);
    expect(page.status).toBe(200);
    expect(page.headers.get("cache-control")).toBe(
      "public, max-age=15, must-revalidate, stale-if-error=0",
    );
    expect(page.headers.get("cloudflare-cdn-cache-control")).toBe(
      "public, max-age=15, must-revalidate, stale-if-error=0",
    );
    const pageBody = await page.text();
    expect(pageBody).toContain("LUKE vs CHUN-LI 分析結果");
    expect(pageBody).toContain('property="og:title"');
    expect(pageBody).toContain(`/manage/${id}`);

    await expect(
      caller.publishedAnalysis.delete({
        id,
        deletePassword: "wrong-delete-password",
      }),
    ).rejects.toMatchObject({ code: "NOT_FOUND" });
    expect((await app.request(`/s/${id}`)).status).toBe(200);

    await expect(
      caller.publishedAnalysis.delete({ id, deletePassword: DELETE_PASSWORD }),
    ).resolves.toEqual({ deleted: true });
    expect((await app.request(`/s/${id}`)).status).toBe(404);
    expect(logger.serialized()).not.toContain(id);
    expect(logger.serialized()).not.toContain(DELETE_PASSWORD);
  });

  test("緊急停止中は新しい共有を作成できない", async () => {
    const previous = process.env.SHARE_RESULTS_ENABLED;
    process.env.SHARE_RESULTS_ENABLED = "false";
    const context = createContext(db, new ConsoleLogger({ minLevel: "error" }));
    const caller = appRouter.createCaller(context);

    try {
      await expect(
        caller.publishedAnalysis.create(createInput()),
      ).rejects.toMatchObject({ code: "NOT_FOUND" });
    } finally {
      if (previous === undefined) delete process.env.SHARE_RESULTS_ENABLED;
      else process.env.SHARE_RESULTS_ENABLED = previous;
    }
  });

  test("tRPC strict schemaが未知キーと自由文を拒否する", async () => {
    const context = createContext(db, new ConsoleLogger({ minLevel: "error" }));
    const caller = appRouter.createCaller(context);
    await expect(
      caller.publishedAnalysis.create({
        analysis: {
          ...candidate(),
          comment: "<script>alert(1)</script>",
        },
        deletePassword: DELETE_PASSWORD,
      } as unknown as Parameters<typeof caller.publishedAnalysis.create>[0]),
    ).rejects.toMatchObject({ code: "BAD_REQUEST" });
  });
});

class CapturingLogger implements ILogger {
  private readonly values: unknown[] = [];

  debug(message: string, ...args: unknown[]): void {
    this.values.push(message, ...args);
  }
  info(message: string, ...args: unknown[]): void {
    this.values.push(message, ...args);
  }
  warn(message: string, ...args: unknown[]): void {
    this.values.push(message, ...args);
  }
  error(message: string, ...args: unknown[]): void {
    this.values.push(message, ...args);
  }
  child(): ILogger {
    return this;
  }
  serialized(): string {
    return JSON.stringify(this.values);
  }
}
