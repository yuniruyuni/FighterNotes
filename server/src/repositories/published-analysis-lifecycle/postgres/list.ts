import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import { compToSQL } from "../../../infra/db/sql-helpers";
import type { Cursor, Page } from "../../../models/common";
import { PublishedAnalysisLifecycle } from "../../../models/published-analysis";
import {
  columnName,
  type LifecycleRow,
  lifecycleSpecToSQL,
  rowToLifecycle,
} from "./common";

export async function list(
  db: Database,
  spec: PublishedAnalysisLifecycle.Spec,
  cursor: Cursor<PublishedAnalysisLifecycle.SortKey>,
): Promise<Page<PublishedAnalysisLifecycle>> {
  const where = compToSQL(spec, lifecycleSpecToSQL);
  const sort = cursor.sort ?? PublishedAnalysisLifecycle.defaultSort;
  const orderBy = sort.keys
    .map((key) => {
      // Stryker disable next-line MethodExpression: PostgreSQL treats ASC/DESC keywords case-insensitively.
      const order = sort.order.toUpperCase();
      return `${columnName(key)} ${order}`;
    })
    .join(", ");
  const rows = await db.queryAll<LifecycleRow>(sql`
    SELECT id, delete_password_hash, created_at, expires_at
    FROM published_analyses
    WHERE ${where}
    ORDER BY ${sql.raw(orderBy)}
    LIMIT ${cursor.limit + 1}
  `);

  const hasMore = rows.length > cursor.limit;
  const items = rows.slice(0, cursor.limit).map(rowToLifecycle);
  const last = items.at(-1);
  return {
    items,
    hasMore,
    nextCursor:
      hasMore && last
        ? PublishedAnalysisLifecycle.cursor(last, sort.keys)
        : undefined,
  };
}
