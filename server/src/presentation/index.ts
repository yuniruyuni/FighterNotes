import { Hono } from "hono";
import { compress } from "hono/compress";
import { createDatabaseReadiness } from "../infra/db";
import type { Context } from "../usecases/context";
import { securityHeaders } from "./http-security";
import { registerPublishedAnalysisRoutes } from "./published-analysis-routes";
import { registerStaticRoutes } from "./static-routes";
import { registerTrpcHttpRoutes } from "./trpc/http";

export function createApp(ctx: Context) {
  const app = new Hono();
  const readiness = createDatabaseReadiness(ctx.db, ctx.logger);

  app.use(compress());
  app.use(securityHeaders());
  app.get("/health", (c) => c.json({ status: "ok" }));
  app.get("/ready", async (c) => {
    if (await readiness.check()) return c.json({ status: "ready" });
    return c.json({ status: "unavailable" }, 503);
  });

  registerPublishedAnalysisRoutes(app, ctx);
  registerTrpcHttpRoutes(app, ctx);
  registerStaticRoutes(app, ctx.config.staticDir);

  return app;
}
