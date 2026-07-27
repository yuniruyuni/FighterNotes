import type { QueryResultRow } from "pg";
import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import { compToSQL } from "../../../infra/db/sql-helpers";
import type { PublishedAnalysisStorageUsage } from "../../../models/published-analysis";

interface StorageUsageRow extends QueryResultRow {
  bytes: string;
}

export async function get(
  db: Database,
  spec: PublishedAnalysisStorageUsage.Spec,
): Promise<PublishedAnalysisStorageUsage | null> {
  const where = compToSQL(spec, (value) => {
    switch (value.type) {
      case "Current":
        return sql.empty();
    }
  });
  const row = await db.queryGet<StorageUsageRow>(sql`
    SELECT (
      pg_total_relation_size('published_analyses'::regclass) +
      pg_total_relation_size('published_analysis_findings'::regclass) +
      pg_total_relation_size('published_analysis_tactics'::regclass) +
      pg_total_relation_size('published_analysis_create_events'::regclass)
    )::bigint AS bytes
    WHERE ${where}
  `);
  return row ? { bytes: Number(row.bytes) } : null;
}
