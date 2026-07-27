import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import type { TransactionLock } from "../../../models/common";

export async function acquire(
  db: Database,
  lock: TransactionLock,
): Promise<void> {
  await db.queryRun(sql`
    SELECT pg_advisory_xact_lock(
      ${lock.namespace}::integer,
      ${lock.id}::integer
    )
  `);
}
