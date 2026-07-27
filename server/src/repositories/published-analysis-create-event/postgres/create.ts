import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import type { PublishedAnalysisCreateEvent } from "../../../models/published-analysis";

export async function create(
  db: Database,
  event: PublishedAnalysisCreateEvent,
): Promise<void> {
  await db.queryRun(sql`
    INSERT INTO published_analysis_create_events (analysis_id, created_at)
    VALUES (${event.analysisId}, ${event.createdAt})
  `);
}
