import { describe, expect, test } from "bun:test";
import { RuntimeConfig } from "../../config";
import type { Database } from "../../infra/db";
import type { ILogger } from "../../infra/logger/types";
import type {
  DeletePasswordHash,
  ShareId,
} from "../../models/published-analysis";
import { createRawRepos } from "../../repositories";
import {
  bindAllRepos,
  createFullCtx,
} from "../../repositories/common/capability";
import type { Context } from "../context";
import type { RuntimeServices } from "../services";
import { deletePublishedAnalysisUsecase } from "./delete";

const ID = "Abcdefghijklmnopqrstu_" as ShareId;
const DELETE_PASSWORD = "fighter-notes-delete-key";
const DELETE_PASSWORD_HASH =
  "$argon2id$v=19$m=19456,t=2,p=1$afqgXENr3y/WCxW5FclnyO6NDY/hIjW2oVS12hgu3b8$Tn12OEC62ylqoD4wLt+6ou9Hq7medNra44FzjO9DlRM" as DeletePasswordHash;

describe("deletePublishedAnalysisUsecase", () => {
  test("ByIdでhashを読み、検証後に汎用deleteを実行する", async () => {
    const ctx = createTestContext();
    ctx.rawRepos.publishedAnalysisLifecycle.list = async (_ctx, spec) => {
      expect(spec).toMatchObject({ type: "ById", id: ID });
      return {
        items: [
          {
            id: ID,
            deletePasswordHash: DELETE_PASSWORD_HASH,
            createdAt: ctx.now,
            expiresAt: new Date("2026-08-15T00:00:00.000Z"),
          },
        ],
        hasMore: false,
      };
    };
    ctx.rawRepos.publishedAnalysisLifecycle.delete = async (_ctx, spec) => {
      expect(spec).toMatchObject({ type: "ById", id: ID });
      return 1;
    };

    const result = await deletePublishedAnalysisUsecase(
      ID,
      DELETE_PASSWORD,
    ).run(ctx);

    expect(result).toEqual({ ok: true, value: { deleted: true } });
    expect(ctx.db.readTransactions).toBe(1);
    expect(ctx.db.writeTransactions).toBe(1);
    expect(ctx.logs.join(" ")).not.toContain(ID);
    expect(ctx.logs.join(" ")).not.toContain(DELETE_PASSWORD);
  });

  test("誤った削除キーは同じNOT_FOUNDにしてdeleteしない", async () => {
    const ctx = createTestContext();
    let deleted = false;
    ctx.rawRepos.publishedAnalysisLifecycle.list = async () => ({
      items: [
        {
          id: ID,
          deletePasswordHash: DELETE_PASSWORD_HASH,
          createdAt: ctx.now,
          expiresAt: new Date("2026-08-15T00:00:00.000Z"),
        },
      ],
      hasMore: false,
    });
    ctx.rawRepos.publishedAnalysisLifecycle.delete = async () => {
      deleted = true;
      return 1;
    };

    const result = await deletePublishedAnalysisUsecase(
      ID,
      "wrong-delete-password",
    ).run(ctx);

    expect(result).toMatchObject({
      ok: false,
      error: { code: "NOT_FOUND" },
    });
    expect(deleted).toBe(false);
    expect(ctx.db.writeTransactions).toBe(0);
  });

  test("不正なIDと短い削除キーはDBへ進まない", async () => {
    const ctx = createTestContext();

    const result = await deletePublishedAnalysisUsecase(
      "not-an-id",
      "short",
    ).run(ctx);

    expect(result).toMatchObject({
      ok: false,
      error: { code: "NOT_FOUND" },
    });
    expect(ctx.db.readTransactions).toBe(0);
    expect(ctx.db.writeTransactions).toBe(0);
  });
});

class TransactionCountingDatabase implements Database {
  readTransactions = 0;
  writeTransactions = 0;

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
    this.writeTransactions += 1;
    return fn(this);
  }

  async readTransaction<T>(fn: (tx: Database) => Promise<T>): Promise<T> {
    this.readTransactions += 1;
    return fn(this);
  }

  async close(): Promise<void> {}
}

function createTestContext(): Context & {
  db: TransactionCountingDatabase;
  logs: string[];
} {
  const db = new TransactionCountingDatabase();
  const logs: string[] = [];
  const logger: ILogger = {
    debug(message) {
      logs.push(message);
    },
    info(message) {
      logs.push(message);
    },
    warn(message) {
      logs.push(message);
    },
    error(message) {
      logs.push(message);
    },
    child() {
      return this;
    },
  };
  const rawRepos = createRawRepos();
  return {
    now: new Date("2026-07-15T12:34:56.000Z"),
    logger,
    db,
    logs,
    rawRepos,
    repos: bindAllRepos(rawRepos, createFullCtx(db)),
    config: RuntimeConfig.fromEnvironment({}),
    services: testServices(),
  };
}

function testServices(): RuntimeServices {
  return {
    publishedAnalysisSecurity: {
      generateShareId: () => ID,
      hashDeletePassword: async () => DELETE_PASSWORD_HASH,
      verifyDeletePassword: async (password, hash) =>
        password === DELETE_PASSWORD && hash === DELETE_PASSWORD_HASH,
    },
  };
}
