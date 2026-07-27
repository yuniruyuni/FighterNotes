import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import { compToSQL } from "../../../infra/db/sql-helpers";
import type { PublishedAnalysisCreateEvent } from "../../../models/published-analysis";
import {
  type CreateEventRow,
  createEventSpecToSQL,
  rowToCreateEvent,
} from "./common";

export async function get(
  db: Database,
  spec: PublishedAnalysisCreateEvent.Spec,
): Promise<PublishedAnalysisCreateEvent | null> {
  const where = compToSQL(spec, createEventSpecToSQL);
  const row = await db.queryGet<CreateEventRow>(sql`
    SELECT analysis_id, created_at
    FROM published_analysis_create_events
    WHERE ${where}
    LIMIT 1
  `);
  return row ? rowToCreateEvent(row) : null;
}
