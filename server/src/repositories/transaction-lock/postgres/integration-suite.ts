import { describe, expect, test } from "bun:test";
import type { PgDatabase } from "../../../infra/db";
import { sql } from "../../../infra/db";
import { PUBLISHED_ANALYSIS_CREATE_LOCK } from "../../../models/published-analysis";
import { createDbWriteCtx } from "../../common/capability";
import { TransactionLockRepository } from ".";

export function registerTransactionLockRepositoryIntegrationTests(
  database: () => PgDatabase,
): void {
  describe("TransactionLockRepository acquire", () => {
    const repository = new TransactionLockRepository();
    const lock = PUBLISHED_ANALYSIS_CREATE_LOCK;

    test("transaction中は同じlockを排他しcommit後に解放する", async () => {
      await database().transaction(async (transaction) => {
        await repository.acquire(createDbWriteCtx(transaction), lock);
        const competing = await database().queryGet<{ acquired: boolean }>(sql`
          SELECT pg_try_advisory_xact_lock(
            ${lock.namespace}::integer,
            ${lock.id}::integer
          ) AS acquired
        `);
        expect(competing?.acquired).toBe(false);
      });

      const afterCommit = await database().queryGet<{ acquired: boolean }>(sql`
        SELECT pg_try_advisory_xact_lock(
          ${lock.namespace}::integer,
          ${lock.id}::integer
        ) AS acquired
      `);
      expect(afterCommit?.acquired).toBe(true);
    });
  });
}
