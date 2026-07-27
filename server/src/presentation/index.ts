import { Hono } from "hono";
import { compress } from "hono/compress";
import type { Context } from "../usecases/context";
import { securityHeaders } from "./http-security";
import { registerPublishedAnalysisRoutes } from "./published-analysis-routes";
import { registerStaticRoutes } from "./static-routes";
import { registerTrpcHttpRoutes } from "./trpc/http";

export function createApp(ctx: Context) {
  const app = new Hono();

  app.use(compress());
  app.use(securityHeaders());
  app.get("/health", (c) => c.json({ status: "ok" }));

  registerPublishedAnalysisRoutes(app, ctx);
  registerTrpcHttpRoutes(app, ctx);
  registerStaticRoutes(app, ctx.config.staticDir);

  return app;
}
