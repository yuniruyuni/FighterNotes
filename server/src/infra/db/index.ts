import type { DatabaseSettings } from "../../config";
import type { ILogger } from "../logger/types";
import type { Database } from "./database";
import { PgDatabase } from "./pg-client";

export type { Database } from "./database";
export { PgDatabase } from "./pg-client";
export { type SQLFragment, sql } from "./sql";

interface DatabaseOptions {
  applicationName?: string;
}

export function initDatabase(
  logger: ILogger,
  settings: DatabaseSettings,
  options: DatabaseOptions = {},
): PgDatabase {
  const log = logger.child("Database");
  log.info(
    `Connecting to PostgreSQL at ${settings.host}:${settings.port}/${settings.database}...`,
  );
  const database = new PgDatabase({
    host: settings.host,
    port: settings.port,
    user: settings.user,
    password: settings.password,
    database: settings.database,
    max: settings.max,
    connectionTimeoutMillis: settings.connectionTimeoutMillis,
    idleTimeoutMillis: settings.idleTimeoutMillis,
    statement_timeout: settings.statementTimeoutMillis,
    lock_timeout: settings.lockTimeoutMillis,
    idle_in_transaction_session_timeout:
      settings.idleInTransactionSessionTimeoutMillis,
    query_timeout: settings.statementTimeoutMillis + 1_000,
    application_name: options.applicationName ?? "fighter-runtime",
  });
  log.info("Database ready");
  return database;
}

export async function closeDatabase(database: Database): Promise<void> {
  await database.close();
}
