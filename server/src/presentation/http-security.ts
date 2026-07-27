import { secureHeaders } from "hono/secure-headers";

export function securityHeaders() {
  return secureHeaders({
    xFrameOptions: "SAMEORIGIN",
    xContentTypeOptions: "nosniff",
    xXssProtection: "1; mode=block",
    referrerPolicy: "strict-origin-when-cross-origin",
    contentSecurityPolicy: {
      defaultSrc: ["'self'"],
      // The analyzer needs WASM, blob-backed video/Workers, and local assets.
      scriptSrc: ["'self'", "'wasm-unsafe-eval'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      fontSrc: ["'none'"],
      imgSrc: ["'self'", "data:", "https:"],
      mediaSrc: ["'self'", "blob:"],
      connectSrc: ["'self'", "blob:", "data:"],
      workerSrc: ["'self'", "blob:"],
      frameAncestors: ["'none'"],
    },
  });
}

export function allowedOrigins(
  requestUrl: string,
  publicBaseUrl: URL,
): Set<string> {
  return new Set([new URL(requestUrl).origin, publicBaseUrl.origin]);
}
