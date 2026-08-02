import { describe, expect, test } from "bun:test";
import type { QueryResultRow } from "pg";
import type { Database } from "../../../infra/db";
import { PublishedAnalysisLifecycle } from "../../../models/published-analysis";
import { deleteBatch } from "./delete-batch";

describe("PublishedAnalysisLifecycle deleteBatch", () => {
  test("1 statementでlimit付きlock・delete・hasMore判定を行う", async () => {
    let statement = "";
    let parameters: unknown[] = [];
    const db = {
      async queryGet<T extends QueryResultRow>(fragment: {
        query: string;
        params: unknown[];
      }): Promise<T> {
        statement = fragment.query;
        parameters = fragment.params;
        return { deleted: 500, has_more: true } as unknown as T;
      },
    } as Database;
    const now = new Date("2026-07-15T12:34:56.000Z");
    const cutoff = new Date("2026-06-15T12:34:56.000Z");

    expect(
      await deleteBatch(
        db,
        PublishedAnalysisLifecycle.ExpiredAt(now).or(
          PublishedAnalysisLifecycle.CreatedAtOrBefore(cutoff),
        ),
        500,
      ),
    ).toEqual({ deleted: 500, hasMore: true });
    expect(statement).toContain("WITH candidates AS MATERIALIZED");
    expect(statement).toContain(
      "ORDER BY expires_at ASC, created_at ASC, id ASC",
    );
    expect(statement).toContain("FOR UPDATE SKIP LOCKED");
    expect(statement).toContain("DELETE FROM published_analyses");
    expect(parameters).toEqual([now, cutoff, 501, 500, 500]);
  });

  test("範囲外limitをquery前に拒否する", async () => {
    const db = {} as Database;
    await expect(
      deleteBatch(db, PublishedAnalysisLifecycle.ExpiredAt(new Date()), 0),
    ).rejects.toThrow("limit must be from 1 to 10000");
  });
});
