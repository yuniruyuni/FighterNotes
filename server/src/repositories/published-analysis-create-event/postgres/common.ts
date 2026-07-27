import type { QueryResultRow } from "pg";
import { type SQLFragment, sql } from "../../../infra/db/sql";
import { assertNever, dateFromSQL } from "../../../infra/db/sql-helpers";
import type {
  PublishedAnalysisCreateEvent,
  ShareId,
} from "../../../models/published-analysis";

export interface CreateEventRow extends QueryResultRow {
  analysis_id: string;
  created_at: Date | string;
}

export function createEventSpecToSQL(
  spec: PublishedAnalysisCreateEvent.SpecData,
): SQLFragment {
  switch (spec.type) {
    case "ByAnalysisId":
      return sql`analysis_id = ${spec.analysisId}`;
    case "CreatedAtOrAfter":
      return sql`created_at >= ${spec.start}`;
    case "CreatedBefore":
      return sql`created_at < ${spec.cutoff}`;
    // Stryker disable next-line ConditionalExpression: SpecData is a closed discriminated union constructed by the domain model.
    default:
      return assertNever(spec);
  }
}

export function rowToCreateEvent(
  row: CreateEventRow,
): PublishedAnalysisCreateEvent {
  return {
    analysisId: row.analysis_id as ShareId,
    createdAt: dateFromSQL(row.created_at),
  };
}
