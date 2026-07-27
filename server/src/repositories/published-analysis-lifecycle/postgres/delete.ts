import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import { compToSQL } from "../../../infra/db/sql-helpers";
import type { PublishedAnalysisLifecycle } from "../../../models/published-analysis";
import { lifecycleSpecToSQL } from "./common";

export async function del(
  db: Database,
  spec: PublishedAnalysisLifecycle.Spec,
): Promise<number> {
  const where = compToSQL(spec, lifecycleSpecToSQL);
  const result = await db.queryRun(sql`
    DELETE FROM published_analyses
    WHERE ${where}
  `);
  return result.rowCount;
}
