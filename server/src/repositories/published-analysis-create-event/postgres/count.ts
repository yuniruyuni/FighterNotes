import type { QueryResultRow } from "pg";
import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import { compToSQL } from "../../../infra/db/sql-helpers";
import type { PublishedAnalysisCreateEvent } from "../../../models/published-analysis";
import { createEventSpecToSQL } from "./common";

interface CountRow extends QueryResultRow {
  count: string;
}

export async function count(
  db: Database,
  spec: PublishedAnalysisCreateEvent.Spec,
): Promise<number> {
  const where = compToSQL(spec, createEventSpecToSQL);
  const row = await db.queryGet<CountRow>(sql`
    SELECT count(*)::bigint AS count
    FROM published_analysis_create_events
    WHERE ${where}
  `);
  // Stryker disable next-line OptionalChaining: PostgreSQL COUNT(*) without GROUP BY always returns exactly one row.
  return Number(row?.count ?? 0);
}
