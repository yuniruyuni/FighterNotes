import { trpcServer } from "@hono/trpc-server";
import type { Hono } from "hono";
import { bodyLimit } from "hono/body-limit";
import { ZodError } from "zod";
import type { Context } from "../../usecases/context";
import type {
  RateLimitDecision,
  SharingRateLimitBucket,
} from "../../usecases/services";
import { allowedOrigins } from "../http-security";
import { requestClientKey } from "../rate-limit";
import { appRouter } from "./routers";

const SHARE_MUTATIONS = new Map<string, SharingRateLimitBucket>([
  ["publishedAnalysis.create", "create"],
  ["publishedAnalysis.delete", "delete"],
]);

export function registerTrpcHttpRoutes(app: Hono, ctx: Context) {
  app.use(
    "/api/trpc/*",
    bodyLimit({
      maxSize: 12 * 1024,
      onError: (c) => c.json({ error: "request body too large" }, 413),
    }),
  );

  app.use("/api/trpc/*", async (c, next) => {
    if (c.req.method === "POST") {
      const contentType = c.req.header("content-type") ?? "";
      const mediaType = contentType.split(";", 1)[0]?.trim().toLowerCase();
      if (mediaType !== "application/json") {
        return c.json({ error: "application/json required" }, 415);
      }
      const origin = c.req.header("origin");
      if (
        origin &&
        !allowedOrigins(c.req.url, ctx.config.publicBaseUrl).has(origin)
      ) {
        return c.json({ error: "origin not allowed" }, 403);
      }
      const procedures = trpcProcedures(c.req.path);
      const shareMutation = procedures
        .map((procedure) => SHARE_MUTATIONS.get(procedure))
        .find((bucket) => bucket !== undefined);
      if (shareMutation) {
        if (procedures.length !== 1) {
          return c.json(
            { error: "batch share mutation is not supported" },
            400,
          );
        }
        if (shareMutation === "create" && !ctx.config.sharing.enabled) {
          await next();
          c.header("Cache-Control", "no-store");
          return;
        }
        const limit =
          shareMutation === "create"
            ? ctx.config.sharing.createRateLimit
            : ctx.config.sharing.deleteRateLimit;
        let decision: RateLimitDecision;
        try {
          decision = await ctx.services.sharingRateLimit.consume(
            shareMutation,
            requestClientKey(
              c.req.raw.headers,
              ctx.config.sharing.trustCloudflareConnectingIp,
            ),
            limit,
          );
        } catch {
          ctx.logger.warn("Published analysis rate limit unavailable", {
            bucket: shareMutation,
          });
          return c.json({ error: "service unavailable" }, 503);
        }
        if (!decision.allowed) {
          ctx.logger.warn("Published analysis request rate limited", {
            bucket: shareMutation,
          });
          c.header("Retry-After", String(decision.retryAfterSeconds));
          return c.json({ error: "rate limit exceeded" }, 429);
        }
      }
    }
    await next();
    c.header("Cache-Control", "no-store");
  });

  app.use(
    "/api/trpc/*",
    trpcServer({
      router: appRouter,
      createContext: () =>
        ({ ...ctx, now: new Date() }) as unknown as Record<string, unknown>,
      onError: ({ error, path }) => {
        if (
          path !== "publishedAnalysis.create" ||
          !(error.cause instanceof ZodError)
        ) {
          return;
        }
        ctx.logger?.warn?.("Published analysis input rejected", {
          procedure: path,
          issues: error.cause.issues.slice(0, 32).map((issue) => ({
            code: issue.code,
            path: issue.path.join(".") || "$",
          })),
        });
      },
    }),
  );
}

function trpcProcedures(path: string): string[] {
  const prefix = "/api/trpc/";
  if (!path.startsWith(prefix)) return [];
  try {
    return decodeURIComponent(path.slice(prefix.length)).split(",");
  } catch {
    return [];
  }
}
