import type { QueryResultRow } from "pg";
import type { Database } from "../../../infra/db/database";
import { type SQLFragment, sql } from "../../../infra/db/sql";
import { compToSQL } from "../../../infra/db/sql-helpers";
import type { PublishedAnalysisLifecycle } from "../../../models/published-analysis";
import type { LifecycleDeleteBatchResult } from "../repository";
import { lifecycleSpecToSQL } from "./common";

interface DeleteBatchRow extends QueryResultRow {
  deleted: number;
  has_more: boolean;
}

export async function deleteBatch(
  db: Database,
  spec: PublishedAnalysisLifecycle.Spec,
  limit: number,
): Promise<LifecycleDeleteBatchResult> {
  const row = await db.queryGet<DeleteBatchRow>(deleteBatchSQL(spec, limit));
  if (!row) throw new Error("Lifecycle delete batch returned no row");
  return { deleted: row.deleted, hasMore: row.has_more };
}

export function deleteBatchSQL(
  spec: PublishedAnalysisLifecycle.Spec,
  limit: number,
): SQLFragment {
  if (!Number.isSafeInteger(limit) || limit < 1 || limit > 10_000) {
    throw new Error("Lifecycle delete batch limit must be from 1 to 10000");
  }
  const where = compToSQL(spec, lifecycleSpecToSQL);
  return sql`
    WITH candidates AS MATERIALIZED (
      SELECT id, expires_at, created_at
      FROM published_analyses
      WHERE ${where}
      ORDER BY expires_at ASC, created_at ASC, id ASC
      LIMIT ${limit + 1}
      FOR UPDATE SKIP LOCKED
    ), selected AS (
      SELECT id
      FROM candidates
      ORDER BY expires_at ASC, created_at ASC, id ASC
      LIMIT ${limit}
    ), deleted AS (
      DELETE FROM published_analyses AS analysis
      USING selected
      WHERE analysis.id = selected.id
      RETURNING analysis.id
    )
    SELECT
      (SELECT count(*)::integer FROM deleted) AS deleted,
      (SELECT count(*) FROM candidates) > ${limit} AS has_more
  `;
}
