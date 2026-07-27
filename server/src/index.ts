import { parseBatchCommand, runBatchCommand } from "./batch";
import { RuntimeConfig } from "./config";
import { createContext } from "./context";
import { closeDatabase, initDatabase } from "./infra/db";
import { createLogger } from "./infra/logger";
import { createApp } from "./presentation";

const logger = createLogger();
const batchCommand = parseBatchCommand(process.argv);
const config = RuntimeConfig.fromEnvironment();
const db = initDatabase(logger, config.database, {
  applicationName: batchCommand ? `fighter-${batchCommand}` : "fighter-runtime",
});
const ctx = createContext(db, logger, { config });

async function shutdown() {
  await closeDatabase(db);
  process.exit(0);
}

if (batchCommand) {
  try {
    if (!(await runBatchCommand(batchCommand, ctx))) process.exitCode = 1;
  } finally {
    await closeDatabase(db);
  }
} else {
  const { port } = config;
  const app = createApp(ctx);

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);

  Bun.serve({
    hostname: "0.0.0.0",
    port,
    fetch: app.fetch,
    idleTimeout: 120,
  });
  logger.info(`Server running on http://localhost:${port}`);
}
