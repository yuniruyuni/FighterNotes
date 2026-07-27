import type { Hono } from "hono";
import { serveStatic } from "hono/bun";

export function registerStaticRoutes(app: Hono, staticDir: string) {
  app.use("/*", async (c, next) => {
    await next();
    if (c.res.status < 400) {
      const path = c.req.path;
      const contentType = c.res.headers.get("Content-Type") ?? "";
      const analyzerArtifact = path.endsWith(".js") || path.endsWith(".wasm");
      const page = contentType.includes("text/html");
      c.res.headers.set(
        "Cache-Control",
        analyzerArtifact || page
          ? "no-store, must-revalidate"
          : "public, max-age=3600",
      );
    }
  });

  app.use("/*", serveStatic({ root: staticDir }));
  app.get("*", serveStatic({ root: staticDir, path: "index.html" }));
}
