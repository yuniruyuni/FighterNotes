import { describe, expect, test } from "bun:test";
import { RuntimeConfig } from "../../config";
import type { Database } from "../../infra/db";
import type { ILogger } from "../../infra/logger/types";
import type {
  DeletePasswordHash,
  PublishedAnalysisCandidate,
  ShareId,
} from "../../models/published-analysis";
import { createRawRepos } from "../../repositories";
import {
  bindAllRepos,
  createFullCtx,
} from "../../repositories/common/capability";
import type { Context } from "../context";
import type { RuntimeServices } from "../services";
import { createPublishedAnalysisUsecase } from "./create";

describe("createPublishedAnalysisUsecase", () => {
  test("lock後にSpecでquotaを読み、eventとaggregateを同じtransactionで作る", async () => {
    const ctx = createTestContext();
    const calls: string[] = [];

    ctx.services.publishedAnalysisSecurity.hashDeletePassword = async () => {
      calls.push("hash");
      expect(ctx.db.transactions).toBe(0);
      return DELETE_PASSWORD_HASH;
    };

    ctx.rawRepos.transactionLock.acquire = async (_ctx, lock) => {
      calls.push("lock");
      expect(lock).toEqual({ namespace: 1_179_537_442, id: 1 });
    };
    ctx.rawRepos.publishedAnalysisCreateEvent.count = async (_ctx, spec) => {
      calls.push("daily-count");
      expect(spec).toMatchObject({
        type: "CreatedAtOrAfter",
        start: new Date("2026-07-15T00:00:00.000Z"),
      });
      return 0;
    };
    ctx.rawRepos.publishedAnalysisLifecycle.count = async (_ctx, spec) => {
      calls.push("active-count");
      expect(spec).toMatchObject({
        type: "ActiveAt",
        at: new Date("2026-07-15T12:34:56.000Z"),
      });
      return 0;
    };
    ctx.rawRepos.publishedAnalysisStorageUsage.get = async (_ctx, spec) => {
      calls.push("storage-get");
      expect(spec).toMatchObject({ type: "Current" });
      return { bytes: 1_000_000 };
    };
    ctx.rawRepos.publishedAnalysisCreateEvent.create = async (_ctx, event) => {
      calls.push("event-create");
      expect(event.createdAt).toEqual(ctx.now);
    };
    ctx.rawRepos.publishedAnalysis.create = async (_ctx, analysis) => {
      calls.push("analysis-create");
      expect(analysis.createdAt).toEqual(ctx.now);
      expect(analysis.deletePasswordHash).toBe(DELETE_PASSWORD_HASH);
      expect(analysis.deletePasswordHash).not.toContain(
        "fighter-notes-delete-key",
      );
    };

    const result = await createPublishedAnalysisUsecase(
      candidate(),
      "fighter-notes-delete-key",
      30,
      {
        dailyCreates: 10,
        activeRows: 100,
        storageBytes: 10_000_000,
      },
    ).run(ctx);

    expect(result.ok).toBe(true);
    expect(calls).toEqual([
      "hash",
      "lock",
      "daily-count",
      "active-count",
      "storage-get",
      "event-create",
      "analysis-create",
    ]);
    expect(ctx.db.transactions).toBe(1);
  });

  test("quota拒否時はeventとaggregateを作らない", async () => {
    const ctx = createTestContext();
    let created = false;

    ctx.rawRepos.transactionLock.acquire = async () => {};
    ctx.rawRepos.publishedAnalysisCreateEvent.count = async () => 1;
    ctx.rawRepos.publishedAnalysisLifecycle.count = async () => 0;
    ctx.rawRepos.publishedAnalysisStorageUsage.get = async () => ({ bytes: 0 });
    ctx.rawRepos.publishedAnalysisCreateEvent.create = async () => {
      created = true;
    };
    ctx.rawRepos.publishedAnalysis.create = async () => {
      created = true;
    };

    const result = await createPublishedAnalysisUsecase(
      candidate(),
      "fighter-notes-delete-key",
      30,
      {
        dailyCreates: 1,
        activeRows: 100,
        storageBytes: 10_000_000,
      },
    ).run(ctx);

    expect(result).toMatchObject({
      ok: false,
      error: { code: "RESOURCE_LIMIT" },
    });
    expect(created).toBe(false);
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

function createTestContext(): Context & { db: TransactionCountingDatabase } {
  const db = new TransactionCountingDatabase();
  const rawRepos = createRawRepos();
  return {
    now: new Date("2026-07-15T12:34:56.000Z"),
    logger,
    db,
    rawRepos,
    repos: bindAllRepos(rawRepos, createFullCtx(db)),
    config: RuntimeConfig.fromEnvironment({}),
    services: testServices(),
  };
}

const SHARE_ID = "Abcdefghijklmnopqrstu_" as ShareId;
const DELETE_PASSWORD_HASH = "fixture-password-hash" as DeletePasswordHash;

function testServices(): RuntimeServices {
  return {
    publishedAnalysisSecurity: {
      generateShareId: () => SHARE_ID,
      hashDeletePassword: async () => DELETE_PASSWORD_HASH,
      verifyDeletePassword: async () => true,
    },
  };
}

function candidate(): PublishedAnalysisCandidate {
  return {
    rulesetVersion: 3,
    ownCharacter: "LUKE",
    opponentCharacter: "CHUN_LI",
    rounds: { detected: 1, won: 1, lost: 0, unresolved: 0 },
    findings: [],
    tactics: {
      antiAir: { opportunities: 0, successes: 0, jumpInsAllowed: 0 },
      driveImpact: {
        faced: 0,
        returned: 0,
        blocked: 0,
        parried: 0,
        hit: 0,
        avoided: 0,
        unconfirmed: 0,
      },
      rawDriveRush: { faced: 0, defended: 0, hit: 0, unconfirmed: 0 },
      dashThrow: { faced: 0 },
      throwWhiff: { count: 0 },
      fastestChallenge: {
        opportunities: 0,
        strikeAttempts: 0,
        strikeLosses: 0,
        throwAttempts: 0,
        throwLosses: 0,
      },
      burnout: {
        count: 0,
        durationDeciseconds: 0,
        hpLostBp: 0,
        hpDealtBp: 0,
        selfInitiated: 0,
        forced: 0,
        mixed: 0,
        unknown: 0,
      },
    },
  };
}
