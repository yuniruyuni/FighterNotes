import type { Hono, Context as HonoContext } from "hono";
import type { Context } from "../usecases/context";
import { getPublishedAnalysisUsecase } from "../usecases/published-analysis";
import { publishedAnalysisCacheHeaders } from "./published-analysis-cache";
import {
  renderPublishedAnalysisNotFoundPage,
  renderPublishedAnalysisPage,
  renderPublishedAnalysisUnavailablePage,
} from "./published-analysis-page";
import { FixedWindowRateLimiter, requestClientKey } from "./rate-limit";

export function registerPublishedAnalysisRoutes(app: Hono, ctx: Context) {
  const { config } = ctx;
  const readLimiter = new FixedWindowRateLimiter(config.sharing.getRateLimit);

  app.get("/s/:id", async (c) => {
    const home = config.publicBaseUrl;
    const now = new Date();
    const rateLimit = readLimiter.consume(requestClientKey(c.req.raw.headers));
    if (!rateLimit.allowed) {
      ctx.logger?.warn?.("Published analysis read rate limited");
      c.header("Cache-Control", "no-store");
      c.header("Retry-After", String(rateLimit.retryAfterSeconds));
      return c.html(renderPublishedAnalysisNotFoundPage(home), 429);
    }
    if (!config.sharing.enabled) {
      c.header("Cache-Control", "no-store");
      return c.html(renderPublishedAnalysisNotFoundPage(home), 404);
    }
    const result = await getPublishedAnalysisUsecase(c.req.param("id")).run({
      ...ctx,
      now,
    });
    if (!result.ok) {
      c.header("Cache-Control", "no-store");
      if (result.error.code === "NOT_FOUND") {
        return c.html(renderPublishedAnalysisNotFoundPage(home), 404);
      }
      return c.html(renderPublishedAnalysisUnavailablePage(home), 503);
    }

    const canonical = new URL(`/s/${result.value.id}`, home);
    const image = new URL("/images/fighter-notes-ogp.jpg", home);
    const cacheHeaders = publishedAnalysisCacheHeaders(
      result.value.expiresAt,
      now,
    );
    c.header("Cache-Control", cacheHeaders.cacheControl);
    if (cacheHeaders.cloudflareCdnCacheControl) {
      c.header(
        "Cloudflare-CDN-Cache-Control",
        cacheHeaders.cloudflareCdnCacheControl,
      );
    }
    return c.html(
      renderPublishedAnalysisPage(result.value, { canonical, home, image }),
    );
  });

  const renderInvalidSharePath = (c: HonoContext) => {
    c.header("Cache-Control", "no-store");
    return c.html(
      renderPublishedAnalysisNotFoundPage(config.publicBaseUrl),
      404,
    );
  };
  app.get("/s", renderInvalidSharePath);
  app.get("/s/*", renderInvalidSharePath);
}
