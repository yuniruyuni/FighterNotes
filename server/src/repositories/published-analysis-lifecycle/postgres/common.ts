import type { QueryResultRow } from "pg";
import { type SQLFragment, sql } from "../../../infra/db/sql";
import { assertNever, dateFromSQL } from "../../../infra/db/sql-helpers";
import type {
  DeletePasswordHash,
  PublishedAnalysisLifecycle,
  ShareId,
} from "../../../models/published-analysis";

export interface LifecycleRow extends QueryResultRow {
  id: string;
  delete_password_hash: string | null;
  created_at: Date | string;
  expires_at: Date | string;
}

export function lifecycleSpecToSQL(
  spec: PublishedAnalysisLifecycle.SpecData,
): SQLFragment {
  switch (spec.type) {
    case "ById":
      return sql`id = ${spec.id}`;
    case "ByIds":
      return spec.ids.length === 0
        ? sql`1=0`
        : sql`id IN (${sql.list(spec.ids)})`;
    case "ActiveAt":
      return sql`expires_at > ${spec.at}`;
    case "ExpiredAt":
      return sql`expires_at <= ${spec.at}`;
    case "CreatedAtOrBefore":
      return sql`created_at <= ${spec.cutoff}`;
    // Stryker disable next-line ConditionalExpression: SpecData is a closed discriminated union constructed by the domain model.
    default:
      return assertNever(spec);
  }
}

export function rowToLifecycle(row: LifecycleRow): PublishedAnalysisLifecycle {
  return {
    id: row.id as ShareId,
    deletePasswordHash: row.delete_password_hash as DeletePasswordHash | null,
    createdAt: dateFromSQL(row.created_at),
    expiresAt: dateFromSQL(row.expires_at),
  };
}

export function columnName(key: PublishedAnalysisLifecycle.SortKey): string {
  const columns: Record<PublishedAnalysisLifecycle.SortKey, string> = {
    createdAt: "created_at",
    expiresAt: "expires_at",
    id: "id",
  };
  return columns[key];
}
