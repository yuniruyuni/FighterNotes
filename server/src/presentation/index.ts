import { Hono } from "hono";
import { compress } from "hono/compress";
import { createDatabaseReadiness } from "../infra/db";
import type { Context } from "../usecases/context";
import { securityHeaders } from "./http-security";
import { requestLog } from "./middleware/request-log";
import { registerPublishedAnalysisRoutes } from "./published-analysis-routes";
import { registerStaticRoutes } from "./static-routes";
import { registerTrpcHttpRoutes } from "./trpc/http";

export function createApp(ctx: Context) {
  const app = new Hono();
  const readiness = createDatabaseReadiness(ctx.db, ctx.logger);

  // いちばん外側に置く。内側で何が起きても 1 行は残る。
  app.use(requestLog(ctx.logger));
  app.use(compress());
  app.use(securityHeaders());
  app.get("/health", (c) => {
    c.header("Cache-Control", "no-store");
    return c.json({ status: "ok" });
  });
  app.get("/ready", async (c) => {
    c.header("Cache-Control", "no-store");
    if (await readiness.check()) return c.json({ status: "ready" });
    return c.json({ status: "unavailable" }, 503);
  });

  registerPublishedAnalysisRoutes(app, ctx);
  registerTrpcHttpRoutes(app, ctx);
  registerStaticRoutes(app, ctx.config.staticDir);

  return app;
}
