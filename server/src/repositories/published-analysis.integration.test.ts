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
import {
  deleteCreatedAtOrBeforeBatchSQL,
  deleteExpiredBatchSQL,
} from "./published-analysis-lifecycle/postgres/delete-batch";
import { registerPublishedAnalysisLifecycleRepositoryIntegrationTests } from "./published-analysis-lifecycle/postgres/integration-suite";
import { registerPublishedAnalysisStorageUsageRepositoryIntegrationTests } from "./published-analysis-storage-usage/postgres/integration-suite";
import {
  candidate,
  createInput,
  DELETE_PASSWORD,
  DELETE_PASSWORD_HASH,
  FIXTURE_ID,
  v9Candidate,
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

  async function expectSchemaDrift(
    apply: string,
    restore: string,
  ): Promise<void> {
    await db.queryRun(sql.raw(apply));
    try {
      expect(await appRoleReadiness()).toBe(false);
    } finally {
      await db.queryRun(sql.raw(restore));
    }
    expect(await appRoleReadiness()).toBe(true);
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

  test("SA/CA集計tableのgrant欠落をreadiness異常にする", async () => {
    await db.queryRun(
      sql.raw(
        "REVOKE SELECT ON published_analysis_super_arts FROM fighter_app",
      ),
    );
    try {
      expect(await appRoleReadiness()).toBe(false);
    } finally {
      await db.queryRun(
        sql.raw("GRANT SELECT ON published_analysis_super_arts TO fighter_app"),
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

  test("runtime列のtypeとnullability driftをreadiness異常にする", async () => {
    await expectSchemaDrift(
      `ALTER TABLE published_analysis_rate_limits
        ALTER COLUMN request_count TYPE BIGINT`,
      `ALTER TABLE published_analysis_rate_limits
        ALTER COLUMN request_count TYPE INTEGER`,
    );
    await expectSchemaDrift(
      `ALTER TABLE published_analysis_findings
        ALTER COLUMN assessment SET NOT NULL`,
      `ALTER TABLE published_analysis_findings
        ALTER COLUMN assessment DROP NOT NULL`,
    );
  });

  test("rate-limit conflict key欠落をreadiness異常にする", async () => {
    await expectSchemaDrift(
      `ALTER TABLE published_analysis_rate_limits
        DROP CONSTRAINT published_analysis_rate_limits_pkey`,
      `ALTER TABLE published_analysis_rate_limits
        ADD CONSTRAINT published_analysis_rate_limits_pkey
        PRIMARY KEY (bucket, client_key_hash)`,
    );
  });

  test("logical quota default driftをreadiness異常にする", async () => {
    await expectSchemaDrift(
      `ALTER TABLE published_analyses
        ALTER COLUMN logical_size_bytes SET DEFAULT 4096`,
      `ALTER TABLE published_analyses
        ALTER COLUMN logical_size_bytes SET DEFAULT 8192`,
    );
    await expectSchemaDrift(
      `ALTER TABLE published_analyses
        DROP CONSTRAINT published_analyses_logical_size_bytes_check;
       ALTER TABLE published_analyses
        ADD CONSTRAINT published_analyses_logical_size_bytes_check
        CHECK (logical_size_bytes BETWEEN 1 AND 16384)`,
      `ALTER TABLE published_analyses
        DROP CONSTRAINT published_analyses_logical_size_bytes_check;
       ALTER TABLE published_analyses
        ADD CONSTRAINT published_analyses_logical_size_bytes_check
        CHECK (logical_size_bytes BETWEEN 1 AND 8192)`,
    );
  });

  test("child FKのCASCADE driftをreadiness異常にする", async () => {
    await expectSchemaDrift(
      `ALTER TABLE published_analysis_findings
        DROP CONSTRAINT published_analysis_findings_analysis_id_fkey;
       ALTER TABLE published_analysis_findings
        ADD CONSTRAINT published_analysis_findings_analysis_id_fkey
        FOREIGN KEY (analysis_id) REFERENCES published_analyses (id)`,
      `ALTER TABLE published_analysis_findings
        DROP CONSTRAINT published_analysis_findings_analysis_id_fkey;
       ALTER TABLE published_analysis_findings
        ADD CONSTRAINT published_analysis_findings_analysis_id_fkey
        FOREIGN KEY (analysis_id) REFERENCES published_analyses (id)
        ON DELETE CASCADE`,
    );
    await expectSchemaDrift(
      `ALTER TABLE published_analysis_super_arts
        DROP CONSTRAINT published_analysis_super_arts_analysis_id_fkey;
       ALTER TABLE published_analysis_super_arts
        ADD CONSTRAINT published_analysis_super_arts_analysis_id_fkey
        FOREIGN KEY (analysis_id) REFERENCES published_analyses (id)`,
      `ALTER TABLE published_analysis_super_arts
        DROP CONSTRAINT published_analysis_super_arts_analysis_id_fkey;
       ALTER TABLE published_analysis_super_arts
        ADD CONSTRAINT published_analysis_super_arts_analysis_id_fkey
        FOREIGN KEY (analysis_id) REFERENCES published_analyses (id)
        ON DELETE CASCADE`,
    );
  });

  test("SA/CA availability制約欠落をreadiness異常にする", async () => {
    await expectSchemaDrift(
      `ALTER TABLE published_analysis_super_arts
        DROP CONSTRAINT published_analysis_super_arts_own_availability_check`,
      `ALTER TABLE published_analysis_super_arts
        ADD CONSTRAINT published_analysis_super_arts_own_availability_check
        CHECK (
          (
            own_available
            AND own_sa1 IS NOT NULL AND own_sa2 IS NOT NULL
            AND own_sa3 IS NOT NULL AND own_ca IS NOT NULL
            AND own_hit IS NOT NULL AND own_block IS NOT NULL
            AND own_no_immediate_contact IS NOT NULL
            AND own_punished IS NOT NULL AND own_ko IS NOT NULL
            AND own_combo IS NOT NULL AND own_punish IS NOT NULL
            AND own_reversal IS NOT NULL AND own_neutral IS NOT NULL
          ) OR (
            NOT own_available
            AND own_sa1 IS NULL AND own_sa2 IS NULL
            AND own_sa3 IS NULL AND own_ca IS NULL
            AND own_hit IS NULL AND own_block IS NULL
            AND own_no_immediate_contact IS NULL
            AND own_punished IS NULL AND own_ko IS NULL
            AND own_combo IS NULL AND own_punish IS NULL
            AND own_reversal IS NULL AND own_neutral IS NULL
          )
        )`,
    );
  });

  test("cleanup index欠落をreadiness異常にする", async () => {
    await expectSchemaDrift(
      "DROP INDEX published_analyses_created_at_cleanup_idx",
      `CREATE INDEX published_analyses_created_at_cleanup_idx
        ON published_analyses (created_at, expires_at, id)`,
    );
  });

  test("row lock用column UPDATE grant欠落をreadiness異常にする", async () => {
    await expectSchemaDrift(
      `REVOKE UPDATE (schema_version)
        ON published_analyses FROM fighter_app`,
      `GRANT UPDATE (schema_version)
        ON published_analyses TO fighter_app`,
    );
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

    await db.queryRun(
      sql.raw(`UPDATE published_analysis_rate_limits
      SET window_started_at = clock_timestamp() - INTERVAL '10 minutes'`),
    );
    const cutoff = new Date(Date.now() - 2 * 60 * 1_000);
    expect(await firstInstance.prune(cutoff, 2)).toEqual({
      deleted: 2,
      hasMore: true,
    });
    expect(await secondInstance.prune(cutoff, 2)).toEqual({
      deleted: 1,
      hasMore: false,
    });
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
      "published_analysis_super_arts",
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

  test("logical容量のhard limitはinsert前に拒否する", async () => {
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

  test("expired row削除後はVACUUM FULLなしでlogical quotaから回復する", async () => {
    const content = createPublishedAnalysisContent(candidate());
    if (!content.ok) throw new Error("fixture is invalid");
    const expired = createPersistablePublishedAnalysis({
      id: FIXTURE_ID,
      content: content.value,
      deletePasswordHash: DELETE_PASSWORD_HASH,
      now: new Date("2026-01-01T00:00:00.000Z"),
      retentionDays: 1,
    });
    await repository.create(createDbWriteCtx(db), expired.analysis);
    const limits = {
      dailyCreates: 10,
      activeRows: 10,
      storageBytes: 8 * 1024,
    };
    const blockedContext = createContext(
      db,
      new ConsoleLogger({ minLevel: "error" }),
      { now: new Date("2026-07-15T00:00:00.000Z") },
    );
    expect(
      await createPublishedAnalysisUsecase(
        candidate(),
        DELETE_PASSWORD,
        30,
        limits,
      ).run(blockedContext),
    ).toMatchObject({
      ok: false,
      error: { code: "RESOURCE_LIMIT" },
    });

    expect(
      await lifecycleRepository.deleteExpiredBatch(
        createDbWriteCtx(db),
        blockedContext.now,
        1,
      ),
    ).toEqual({ deleted: 1, hasMore: false });
    const resumedContext = createContext(
      db,
      new ConsoleLogger({ minLevel: "error" }),
      { now: blockedContext.now },
    );
    expect(
      await createPublishedAnalysisUsecase(
        candidate(),
        DELETE_PASSWORD,
        30,
        limits,
      ).run(resumedContext),
    ).toMatchObject({ ok: true });
  });

  test("10k backlogは複合index planで並行batch cleanupできる", async () => {
    await db.queryRun(
      sql.raw(`
      INSERT INTO published_analyses (
        id, schema_version, ruleset_version, presentation_revision,
        own_character, opponent_character,
        rounds_detected, rounds_won, rounds_lost, rounds_unresolved,
        logical_size_bytes, created_at, expires_at
      )
      SELECT
        lpad(to_hex(value), 22, '0'), 1, 3, 1, 'LUKE', 'CHUN_LI',
        0, 0, 0, 0, 8192,
        TIMESTAMPTZ '2026-01-01 00:00:00+00',
        TIMESTAMPTZ '2026-02-01 00:00:00+00'
      FROM generate_series(1, 10000) AS value
    `),
    );
    await db.queryRun(
      sql.raw(`
      INSERT INTO published_analysis_findings (
        analysis_id, ordinal, kind, assessment, occurrences, severity_bp
      )
      SELECT id, 0, 'anti_air', 'observation', 1, 1
      FROM published_analyses
    `),
    );
    await db.queryRun(sql.raw("ANALYZE published_analyses"));
    const expiredAt = new Date("2026-07-15T00:00:00.000Z");
    const planRows = await db.queryAll<{ "QUERY PLAN": string }>(sql`
      EXPLAIN (COSTS OFF) ${deleteExpiredBatchSQL(expiredAt, 500)}
    `);
    const plan = planRows.map((row) => row["QUERY PLAN"]).join("\n");
    expect(plan).toContain("published_analyses_cleanup_idx");

    const startedAt = performance.now();
    const worker = async () => {
      let deleted = 0;
      for (;;) {
        const batch = await lifecycleRepository.deleteExpiredBatch(
          createDbWriteCtx(db),
          expiredAt,
          500,
        );
        deleted += batch.deleted;
        if (!batch.hasMore) return deleted;
      }
    };
    const deleted = await Promise.all([worker(), worker()]);
    const elapsedMillis = performance.now() - startedAt;
    expect(deleted[0] + deleted[1]).toBe(10_000);
    expect(elapsedMillis).toBeLessThan(30_000);
    expect(
      await lifecycleRepository.count(
        createDbReadCtx(db),
        PublishedAnalysisLifecycle.ExpiredAt(
          new Date("2026-07-15T00:00:00.000Z"),
        ),
      ),
    ).toBe(0);
    const findings = await db.queryGet<{ count: string }>(
      sql.raw("SELECT count(*) AS count FROM published_analysis_findings"),
    );
    expect(findings?.count).toBe("0");
  });

  test("active 100k行の後方にあるcreated_at超過rowを専用indexから削除する", async () => {
    await db.queryRun(
      sql.raw(`
      INSERT INTO published_analyses (
        id, schema_version, ruleset_version, presentation_revision,
        own_character, opponent_character,
        rounds_detected, rounds_won, rounds_lost, rounds_unresolved,
        logical_size_bytes, created_at, expires_at
      )
      SELECT
        'a' || lpad(to_hex(value), 21, '0'), 1, 3, 1, 'LUKE', 'CHUN_LI',
        0, 0, 0, 0, 8192,
        TIMESTAMPTZ '2026-07-01 00:00:00+00',
        TIMESTAMPTZ '2026-08-01 00:00:00+00'
      FROM generate_series(1, 100000) AS value;

      INSERT INTO published_analyses (
        id, schema_version, ruleset_version, presentation_revision,
        own_character, opponent_character,
        rounds_detected, rounds_won, rounds_lost, rounds_unresolved,
        logical_size_bytes, created_at, expires_at
      )
      SELECT
        'b' || lpad(to_hex(value), 21, '0'), 1, 3, 1, 'LUKE', 'CHUN_LI',
        0, 0, 0, 0, 8192,
        TIMESTAMPTZ '2026-01-01 00:00:00+00',
        TIMESTAMPTZ '2030-01-01 00:00:00+00'
      FROM generate_series(1, 1000) AS value
    `),
    );
    await db.queryRun(sql.raw("ANALYZE published_analyses"));
    const cutoff = new Date("2026-06-15T00:00:00.000Z");
    const planRows = await db.queryAll<{ "QUERY PLAN": string }>(sql`
      EXPLAIN (COSTS OFF)
      ${deleteCreatedAtOrBeforeBatchSQL(cutoff, 500)}
    `);
    const plan = planRows.map((row) => row["QUERY PLAN"]).join("\n");
    expect(plan).toContain("published_analyses_created_at_cleanup_idx");
    expect(plan).not.toContain("published_analyses_cleanup_idx");

    const startedAt = performance.now();
    let deleted = 0;
    for (;;) {
      const batch = await lifecycleRepository.deleteCreatedAtOrBeforeBatch(
        createDbWriteCtx(db),
        cutoff,
        500,
      );
      deleted += batch.deleted;
      if (!batch.hasMore) break;
    }
    const elapsedMillis = performance.now() - startedAt;
    expect(deleted).toBe(1_000);
    expect(elapsedMillis).toBeLessThan(30_000);
    const remaining = await db.queryGet<{ count: string }>(
      sql.raw(`
      SELECT count(*) AS count
      FROM published_analyses
      WHERE created_at > TIMESTAMPTZ '2026-06-15 00:00:00+00'
    `),
    );
    expect(remaining?.count).toBe("100000");
  });

  test("app roleはruntimeとcleanupに必要なDMLだけを持つ", async () => {
    const privileges = await db.queryGet<{
      app_parent_select: boolean;
      app_parent_insert: boolean;
      app_parent_update: boolean;
      app_parent_schema_version_update: boolean;
      app_parent_id_update: boolean;
      app_parent_delete: boolean;
      app_events_select: boolean;
      app_events_insert: boolean;
      app_events_update: boolean;
      app_events_delete: boolean;
      app_limits_select: boolean;
      app_limits_insert: boolean;
      app_limits_update: boolean;
      app_limits_delete: boolean;
      app_super_arts_select: boolean;
      app_super_arts_insert: boolean;
      app_super_arts_update: boolean;
      app_super_arts_delete: boolean;
    }>(
      sql.raw(`
      SELECT
        has_table_privilege('fighter_app', 'published_analyses', 'SELECT')
          AS app_parent_select,
        has_table_privilege('fighter_app', 'published_analyses', 'INSERT')
          AS app_parent_insert,
        has_table_privilege('fighter_app', 'published_analyses', 'UPDATE')
          AS app_parent_update,
        has_column_privilege(
          'fighter_app',
          'published_analyses',
          'schema_version',
          'UPDATE'
        ) AS app_parent_schema_version_update,
        has_column_privilege(
          'fighter_app',
          'published_analyses',
          'id',
          'UPDATE'
        ) AS app_parent_id_update,
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
        ) AS app_limits_delete,
        has_table_privilege(
          'fighter_app',
          'published_analysis_super_arts',
          'SELECT'
        ) AS app_super_arts_select,
        has_table_privilege(
          'fighter_app',
          'published_analysis_super_arts',
          'INSERT'
        ) AS app_super_arts_insert,
        has_table_privilege(
          'fighter_app',
          'published_analysis_super_arts',
          'UPDATE'
        ) AS app_super_arts_update,
        has_table_privilege(
          'fighter_app',
          'published_analysis_super_arts',
          'DELETE'
        ) AS app_super_arts_delete
    `),
    );
    expect(privileges).toEqual({
      app_parent_select: true,
      app_parent_insert: true,
      app_parent_update: false,
      app_parent_schema_version_update: true,
      app_parent_id_update: false,
      app_parent_delete: true,
      app_events_select: true,
      app_events_insert: true,
      app_events_update: false,
      app_events_delete: true,
      app_limits_select: true,
      app_limits_insert: true,
      app_limits_update: true,
      app_limits_delete: true,
      app_super_arts_select: true,
      app_super_arts_insert: true,
      app_super_arts_update: false,
      app_super_arts_delete: false,
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
        await repos.publishedAnalysisLifecycle.deleteExpiredBatch(
          ctx,
          created.analysis.expiresAt,
          1,
        ),
      ).toEqual({ deleted: 1, hasMore: false });
      for (const table of [
        "published_analyses",
        "published_analysis_findings",
        "published_analysis_tactics",
        "published_analysis_super_arts",
      ]) {
        const remaining = await tx.queryGet<{ count: string }>(
          sql.raw(`SELECT count(*) AS count FROM ${table}`),
        );
        expect(remaining?.count).toBe("0");
      }
      expect(
        await repos.publishedAnalysisCreateEvent.delete(
          ctx,
          PublishedAnalysisCreateEvent.ByAnalysisId(created.analysis.id),
        ),
      ).toBe(1);
    });
  });

  test("transaction失敗時は4テーブルすべてrollbackする", async () => {
    const content = createPublishedAnalysisContent(v9Candidate());
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

  test("DB制約がSA/CA集計不能と0回を混同する行を拒否する", async () => {
    const content = createPublishedAnalysisContent(v9Candidate());
    if (!content.ok) throw new Error("fixture is invalid");
    const created = createPersistablePublishedAnalysis({
      id: FIXTURE_ID,
      content: content.value,
      deletePasswordHash: DELETE_PASSWORD_HASH,
      now: new Date(),
      retentionDays: 365,
    });
    await repository.create(createDbWriteCtx(db), created.analysis);

    await expect(
      db.queryRun(sql`
        UPDATE published_analysis_super_arts
        SET own_available = false
        WHERE analysis_id = ${created.analysis.id}
      `),
    ).rejects.toThrow();
  });

  test("最大モデルの実データ行サイズが8KiB未満", async () => {
    const content = createPublishedAnalysisContent(v9Candidate(true));
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
        pg_column_size(a.*) + pg_column_size(t.*) + pg_column_size(s.*) +
        COALESCE((
          SELECT sum(pg_column_size(f.*))
          FROM published_analysis_findings f
          WHERE f.analysis_id = a.id
        ), 0)::integer AS bytes
      FROM published_analyses a
      INNER JOIN published_analysis_tactics t ON t.analysis_id = a.id
      INNER JOIN published_analysis_super_arts s ON s.analysis_id = a.id
      WHERE a.id = ${created.analysis.id}
    `);
    expect(row?.bytes).toBeLessThan(8 * 1024);
  });

  test("tRPCが削除キー付き共有を作り15秒cacheで配信・削除する", async () => {
    const logger = new CapturingLogger();
    const context = createContext(db, logger);
    context.now = new Date();
    const caller = appRouter.createCaller(context);
    const created = await caller.publishedAnalysis.create(
      createInput(v9Candidate()),
    );
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
    expect(pageBody).toContain("SA / CA 集計");
    expect(pageBody).toContain("自分のSA / CA使用");
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
