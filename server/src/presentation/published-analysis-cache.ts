const FRESH_SECONDS = 15;

export interface PublishedAnalysisCacheHeaders {
  cacheControl: string;
  cloudflareCdnCacheControl?: string;
}

export function publishedAnalysisCacheHeaders(
  expiresAt: Date,
  now: Date,
): PublishedAnalysisCacheHeaders {
  const remainingSeconds = Math.floor(
    (expiresAt.getTime() - now.getTime()) / 1_000,
  );
  if (remainingSeconds <= 0) {
    return { cacheControl: "no-store" };
  }

  const freshSeconds = Math.min(FRESH_SECONDS, remainingSeconds);
  const policy = `public, max-age=${freshSeconds}, must-revalidate, stale-if-error=0`;

  return {
    cacheControl: policy,
    cloudflareCdnCacheControl: policy,
  };
}
