import type { QueryResultRow } from "pg";
import type { Database } from "../../../infra/db/database";
import { type SQLFragment, sql } from "../../../infra/db/sql";
import type { LifecycleDeleteBatchResult } from "../repository";

interface DeleteBatchRow extends QueryResultRow {
  deleted: number;
  has_more: boolean;
}

interface CleanupOrdering {
  readonly boundaryColumn: "expires_at" | "created_at";
  readonly trailingColumn: "created_at" | "expires_at";
}

const EXPIRED_ORDERING: CleanupOrdering = {
  boundaryColumn: "expires_at",
  trailingColumn: "created_at",
};
const CREATED_AT_ORDERING: CleanupOrdering = {
  boundaryColumn: "created_at",
  trailingColumn: "expires_at",
};

export function deleteExpiredBatch(
  db: Database,
  at: Date,
  limit: number,
): Promise<LifecycleDeleteBatchResult> {
  return deleteBatch(db, deleteExpiredBatchSQL(at, limit));
}

export function deleteCreatedAtOrBeforeBatch(
  db: Database,
  cutoff: Date,
  limit: number,
): Promise<LifecycleDeleteBatchResult> {
  return deleteBatch(db, deleteCreatedAtOrBeforeBatchSQL(cutoff, limit));
}

async function deleteBatch(
  db: Database,
  statement: SQLFragment,
): Promise<LifecycleDeleteBatchResult> {
  const row = await db.queryGet<DeleteBatchRow>(statement);
  if (!row) throw new Error("Lifecycle delete batch returned no row");
  return { deleted: row.deleted, hasMore: row.has_more };
}

export function deleteExpiredBatchSQL(at: Date, limit: number): SQLFragment {
  return deleteBatchSQL(EXPIRED_ORDERING, at, limit);
}

export function deleteCreatedAtOrBeforeBatchSQL(
  cutoff: Date,
  limit: number,
): SQLFragment {
  return deleteBatchSQL(CREATED_AT_ORDERING, cutoff, limit);
}

function deleteBatchSQL(
  ordering: CleanupOrdering,
  boundary: Date,
  limit: number,
): SQLFragment {
  if (!Number.isSafeInteger(limit) || limit < 1 || limit > 10_000) {
    throw new Error("Lifecycle delete batch limit must be from 1 to 10000");
  }
  const boundaryColumn = sql.raw(ordering.boundaryColumn);
  const trailingColumn = sql.raw(ordering.trailingColumn);
  return sql`
    WITH candidates AS MATERIALIZED (
      SELECT id, ${boundaryColumn}, ${trailingColumn}
      FROM published_analyses
      WHERE ${boundaryColumn} <= ${boundary}
      ORDER BY ${boundaryColumn} ASC, ${trailingColumn} ASC, id ASC
      LIMIT ${limit + 1}
      FOR UPDATE SKIP LOCKED
    ), selected AS (
      SELECT id
      FROM candidates
      ORDER BY ${boundaryColumn} ASC, ${trailingColumn} ASC, id ASC
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
