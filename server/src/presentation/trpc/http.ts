import { trpcServer } from "@hono/trpc-server";
import type { Hono } from "hono";
import { bodyLimit } from "hono/body-limit";
import { ZodError } from "zod";
import type { Context } from "../../usecases/context";
import { allowedOrigins } from "../http-security";
import { FixedWindowRateLimiter, requestClientKey } from "../rate-limit";
import { appRouter } from "./routers";

const SHARE_MUTATIONS = new Set([
  "publishedAnalysis.create",
  "publishedAnalysis.delete",
]);

export function registerTrpcHttpRoutes(app: Hono, ctx: Context) {
  const mutationLimiter = new FixedWindowRateLimiter(
    ctx.config.sharing.createRateLimit,
  );

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
      const shareMutation = procedures.some((procedure) =>
        SHARE_MUTATIONS.has(procedure),
      );
      if (shareMutation) {
        if (procedures.length !== 1) {
          return c.json(
            { error: "batch share mutation is not supported" },
            400,
          );
        }
        const decision = mutationLimiter.consume(
          requestClientKey(c.req.raw.headers),
        );
        if (!decision.allowed) {
          ctx.logger?.warn?.("Published analysis mutation rate limited");
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
