import { describe, expect, test } from "bun:test";
import { runPublishedAnalysisCleanup } from "../../batch";
import { RuntimeConfig } from "../../config";
import type { Database } from "../../infra/db";
import type { ILogger } from "../../infra/logger/types";
import { createRuntimeServices } from "../../infra/security";
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
    const expiredBatches = [
      { deleted: 2, hasMore: true },
      { deleted: 2, hasMore: true },
      { deleted: 1, hasMore: false },
    ];
    const retainedBatches = [{ deleted: 2, hasMore: false }];
    const expiredAt: Date[] = [];
    const retentionCutoffs: Date[] = [];
    const deleteLimits: number[] = [];
    const rateLimitBatches = [
      { deleted: 2, hasMore: true },
      { deleted: 1, hasMore: false },
    ];
    let quotaSpec: unknown;

    ctx.rawRepos.publishedAnalysisLifecycle.deleteExpiredBatch = async (
      _ctx,
      at,
      limit,
    ) => {
      expiredAt.push(at);
      deleteLimits.push(limit);
      return expiredBatches.shift() ?? { deleted: 0, hasMore: false };
    };
    ctx.rawRepos.publishedAnalysisLifecycle.deleteCreatedAtOrBeforeBatch =
      async (_ctx, cutoff, limit) => {
        retentionCutoffs.push(cutoff);
        deleteLimits.push(limit);
        return retainedBatches.shift() ?? { deleted: 0, hasMore: false };
      };
    ctx.rawRepos.publishedAnalysisCreateEvent.delete = async (_ctx, spec) => {
      quotaSpec = spec;
      return 3;
    };
    ctx.services.sharingRateLimit.prune = async (before, limit) => {
      expect(before).toEqual(new Date("2026-07-15T12:32:56.000Z"));
      expect(limit).toBe(2);
      return rateLimitBatches.shift() ?? { deleted: 0, hasMore: false };
    };

    const result = await runPublishedAnalysisCleanup(ctx, {
      batchSize: 2,
      maxBatches: 10,
      retentionDays: 30,
    });

    expect(result).toEqual({
      ok: true,
      value: { expired: 7, rateLimits: 3, quotaEvents: 3, batches: 4 },
    });
    expect(expiredAt).toEqual([
      new Date("2026-07-15T12:34:56.000Z"),
      new Date("2026-07-15T12:34:56.000Z"),
      new Date("2026-07-15T12:34:56.000Z"),
    ]);
    expect(retentionCutoffs).toEqual([new Date("2026-06-15T12:34:56.000Z")]);
    expect(deleteLimits).toEqual([2, 2, 2, 2]);
    expect(quotaSpec).toMatchObject({
      type: "CreatedBefore",
      cutoff: new Date("2026-07-13T00:00:00.000Z"),
    });
    expect(database.transactions).toBe(5);
  });

  test("batch安全上限に達したら失敗しquota eventには進まない", async () => {
    const database = new TransactionCountingDatabase();
    const ctx = createTestContext(database);
    let quotaCleanupCalled = false;
    let rateLimitCleanupCalled = false;

    ctx.rawRepos.publishedAnalysisLifecycle.deleteExpiredBatch = async () => ({
      deleted: 2,
      hasMore: true,
    });
    ctx.rawRepos.publishedAnalysisCreateEvent.delete = async () => {
      quotaCleanupCalled = true;
      return 0;
    };
    ctx.services.sharingRateLimit.prune = async () => {
      rateLimitCleanupCalled = true;
      return { deleted: 0, hasMore: false };
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
    expect(rateLimitCleanupCalled).toBe(false);
    expect(database.transactions).toBe(2);
  });

  test("rate-limit pruneの安全上限でも失敗しquota eventへ進まない", async () => {
    const database = new TransactionCountingDatabase();
    const ctx = createTestContext(database);
    let quotaCleanupCalled = false;
    ctx.rawRepos.publishedAnalysisLifecycle.deleteExpiredBatch = async () => ({
      deleted: 0,
      hasMore: false,
    });
    ctx.rawRepos.publishedAnalysisLifecycle.deleteCreatedAtOrBeforeBatch =
      async () => ({ deleted: 0, hasMore: false });
    ctx.services.sharingRateLimit.prune = async () => ({
      deleted: 2,
      hasMore: true,
    });
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
  });

  test("DB失敗をusecaseのINTERNAL failureとして返す", async () => {
    const database = new TransactionCountingDatabase();
    const ctx = createTestContext(database);
    ctx.rawRepos.publishedAnalysisLifecycle.deleteExpiredBatch = async () => {
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
