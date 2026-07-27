import { describe, expect, test } from "bun:test";
import { runPublishedAnalysisCleanup } from "../../batch";
import { RuntimeConfig } from "../../config";
import type { Database } from "../../infra/db";
import type { ILogger } from "../../infra/logger/types";
import { createRuntimeServices } from "../../infra/security";
import type {
  PublishedAnalysisLifecycle,
  ShareId,
} from "../../models/published-analysis";
import { createRawRepos } from "../../repositories";
import {
  bindAllRepos,
  createFullCtx,
} from "../../repositories/common/capability";
import type { Context } from "../context";

describe("published analysis cleanup batch command", () => {
  test("期限切れrowを短いbatchで削除して古いquota eventをpruneする", async () => {
    const database = new TransactionCountingDatabase();
    const ctx = createTestContext(database);
    const pages = [
      page([lifecycle("a", 0), lifecycle("b", 1)], true),
      page([lifecycle("c", 2), lifecycle("d", 3)], true),
      page([lifecycle("e", 4)], false),
    ];
    const listSpecs: unknown[] = [];
    const deleteSpecs: unknown[] = [];
    let quotaSpec: unknown;

    ctx.rawRepos.publishedAnalysisLifecycle.list = async (
      _ctx,
      spec,
      cursor,
    ) => {
      listSpecs.push(spec);
      expect(cursor).toEqual({
        limit: 2,
        sort: { keys: ["expiresAt", "id"], order: "asc" },
      });
      return pages.shift() ?? page([], false);
    };
    ctx.rawRepos.publishedAnalysisLifecycle.delete = async (_ctx, spec) => {
      deleteSpecs.push(spec);
      return spec.type === "ByIds" ? spec.ids.length : 0;
    };
    ctx.rawRepos.publishedAnalysisCreateEvent.delete = async (_ctx, spec) => {
      quotaSpec = spec;
      return 3;
    };

    const result = await runPublishedAnalysisCleanup(ctx, {
      batchSize: 2,
      maxBatches: 10,
      retentionDays: 30,
    });

    expect(result).toEqual({
      ok: true,
      value: { expired: 5, quotaEvents: 3, batches: 3 },
    });
    expect(listSpecs).toHaveLength(3);
    expect(listSpecs[0]).toMatchObject({
      type: "or",
      children: [
        { type: "ExpiredAt", at: new Date("2026-07-15T12:34:56.000Z") },
        {
          type: "CreatedAtOrBefore",
          cutoff: new Date("2026-06-15T12:34:56.000Z"),
        },
      ],
    });
    expect(deleteSpecs).toHaveLength(3);
    expect(deleteSpecs[0]).toMatchObject({
      type: "ByIds",
      ids: [shareId("a"), shareId("b")],
    });
    expect(quotaSpec).toMatchObject({
      type: "CreatedBefore",
      cutoff: new Date("2026-07-13T00:00:00.000Z"),
    });
    expect(database.transactions).toBe(4);
  });

  test("batch安全上限に達したら失敗しquota eventには進まない", async () => {
    const database = new TransactionCountingDatabase();
    const ctx = createTestContext(database);
    let quotaCleanupCalled = false;

    ctx.rawRepos.publishedAnalysisLifecycle.list = async () =>
      page([lifecycle("a", 0), lifecycle("b", 1)], true);
    ctx.rawRepos.publishedAnalysisLifecycle.delete = async () => 2;
    ctx.rawRepos.publishedAnalysisCreateEvent.delete = async () => {
      quotaCleanupCalled = true;
      return 0;
    };

    const result = await runPublishedAnalysisCleanup(ctx, {
      batchSize: 2,
      maxBatches: 2,
      retentionDays: 30,
    });

    expect(result).toMatchObject({
      ok: false,
      error: { code: "RESOURCE_LIMIT" },
    });
    expect(quotaCleanupCalled).toBe(false);
    expect(database.transactions).toBe(2);
  });

  test("DB失敗をusecaseのINTERNAL failureとして返す", async () => {
    const database = new TransactionCountingDatabase();
    const ctx = createTestContext(database);
    ctx.rawRepos.publishedAnalysisLifecycle.list = async () => {
      throw new Error("database unavailable");
    };

    const result = await runPublishedAnalysisCleanup(ctx, {
      batchSize: 500,
      maxBatches: 1_000,
      retentionDays: 30,
    });

    expect(result).toMatchObject({
      ok: false,
      error: { code: "INTERNAL" },
    });
  });
});

class TransactionCountingDatabase implements Database {
  transactions = 0;

  queryGet(): Promise<never> {
    throw new Error("Unexpected queryGet");
  }

  queryAll(): Promise<never[]> {
    throw new Error("Unexpected queryAll");
  }

  queryRun(): Promise<never> {
    throw new Error("Unexpected queryRun");
  }

  async transaction<T>(fn: (tx: Database) => Promise<T>): Promise<T> {
    this.transactions++;
    return fn(this);
  }

  readTransaction<T>(): Promise<T> {
    throw new Error("Unexpected readTransaction");
  }

  async close(): Promise<void> {}
}

const logger: ILogger = {
  debug() {},
  info() {},
  warn() {},
  error() {},
  child() {
    return this;
  },
};

function createTestContext(database: Database): Context {
  const rawRepos = createRawRepos();
  return {
    now: new Date("2026-07-15T12:34:56.000Z"),
    logger,
    db: database,
    rawRepos,
    repos: bindAllRepos(rawRepos, createFullCtx(database)),
    config: RuntimeConfig.fromEnvironment({}),
    services: createRuntimeServices(),
  };
}

function page(items: PublishedAnalysisLifecycle[], hasMore: boolean) {
  return { items, hasMore };
}

function lifecycle(seed: string, day: number): PublishedAnalysisLifecycle {
  return {
    id: shareId(seed),
    deletePasswordHash: null,
    createdAt: new Date(Date.UTC(2026, 0, day + 1)),
    expiresAt: new Date(Date.UTC(2026, 1, day + 1)),
  };
}

function shareId(seed: string): ShareId {
  return seed.repeat(22).slice(0, 22) as ShareId;
}
