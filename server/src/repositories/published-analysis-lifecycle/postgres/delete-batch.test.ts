import { describe, expect, test } from "bun:test";
import type { QueryResultRow } from "pg";
import type { Database } from "../../../infra/db";
import {
  deleteCreatedAtOrBeforeBatch,
  deleteExpiredBatch,
} from "./delete-batch";

describe("PublishedAnalysisLifecycle cleanup batches", () => {
  test("期限切れをexpires_at index順の1 statementでlock・deleteする", async () => {
    const fixture = queryFixture();
    const now = new Date("2026-07-15T12:34:56.000Z");

    expect(await deleteExpiredBatch(fixture.db, now, 500)).toEqual({
      deleted: 500,
      hasMore: true,
    });
    expect(fixture.statement()).toContain("WITH candidates AS MATERIALIZED");
    expect(fixture.statement()).toContain("WHERE expires_at <=");
    expect(fixture.statement()).toContain(
      "ORDER BY expires_at ASC, created_at ASC, id ASC",
    );
    expect(fixture.statement()).toContain("FOR UPDATE SKIP LOCKED");
    expect(fixture.statement()).toContain("DELETE FROM published_analyses");
    expect(fixture.parameters()).toEqual([now, 501, 500, 500]);
  });

  test("retention超過をcreated_at index順の別statementで削除する", async () => {
    const fixture = queryFixture();
    const cutoff = new Date("2026-06-15T12:34:56.000Z");

    expect(await deleteCreatedAtOrBeforeBatch(fixture.db, cutoff, 250)).toEqual(
      { deleted: 500, hasMore: true },
    );
    expect(fixture.statement()).toContain("WHERE created_at <=");
    expect(fixture.statement()).toContain(
      "ORDER BY created_at ASC, expires_at ASC, id ASC",
    );
    expect(fixture.parameters()).toEqual([cutoff, 251, 250, 250]);
  });

  /**
   * limit は境界そのもの。1件も消さない batch と、1回で上限を超える batch の
   * どちらも cleanup の前提を壊すので、両端で拒否と受理を固定する。
   */
  test("範囲外limitをquery前に拒否する", async () => {
    for (const invalid of [0, -1, 10_001, 1.5, Number.NaN]) {
      expect(() =>
        deleteExpiredBatch({} as Database, new Date(), invalid),
      ).toThrow("limit must be from 1 to 10000");
      expect(() =>
        deleteCreatedAtOrBeforeBatch({} as Database, new Date(), invalid),
      ).toThrow("limit must be from 1 to 10000");
    }

    for (const valid of [1, 10_000]) {
      const fixture = queryFixture();
      expect(
        await deleteExpiredBatch(fixture.db, new Date(), valid),
      ).toBeDefined();
      expect(fixture.parameters()).toEqual([
        expect.any(Date),
        valid + 1,
        valid,
        valid,
      ]);
    }
  });

  /**
   * batch の結果行が返らないのは、statement が想定どおり動いていない
   * ということ。0件削除として続けると、cleanup が静かに空回りする。
   */
  test("結果行が返らない場合は失敗として扱う", async () => {
    const db = {
      async queryGet() {
        return undefined;
      },
    } as unknown as Database;

    expect(deleteExpiredBatch(db, new Date(), 500)).rejects.toThrow(
      "Lifecycle delete batch returned no row",
    );
  });
});

function queryFixture(): {
  readonly db: Database;
  readonly statement: () => string;
  readonly parameters: () => unknown[];
} {
  let statement = "";
  let parameters: unknown[] = [];
  return {
    statement: () => statement,
    parameters: () => parameters,
    db: {
      async queryGet<T extends QueryResultRow>(fragment: {
        query: string;
        params: unknown[];
      }): Promise<T> {
        statement = fragment.query;
        parameters = fragment.params;
        return { deleted: 500, has_more: true } as unknown as T;
      },
    } as Database,
  };
}
