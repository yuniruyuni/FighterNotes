import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import { compToSQL } from "../../../infra/db/sql-helpers";
import type { PublishedAnalysisCreateEvent } from "../../../models/published-analysis";
import { createEventSpecToSQL } from "./common";

export async function del(
  db: Database,
  spec: PublishedAnalysisCreateEvent.Spec,
): Promise<number> {
  const where = compToSQL(spec, createEventSpecToSQL);
  const result = await db.queryRun(sql`
    DELETE FROM published_analysis_create_events
    WHERE ${where}
  `);
  return result.rowCount;
}
