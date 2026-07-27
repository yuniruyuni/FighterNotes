import { describe, expect, test } from "bun:test";
import { publishedAnalysisCacheHeaders } from "./published-analysis-cache";

const now = new Date("2026-07-15T00:00:00.000Z");

describe("published analysis cache policy", () => {
  test("通常の公開結果はbrowserとedgeの両方を15秒だけ保存する", () => {
    const headers = publishedAnalysisCacheHeaders(
      new Date("2026-08-14T00:00:00.000Z"),
      now,
    );
    expect(headers).toEqual({
      cacheControl: "public, max-age=15, must-revalidate, stale-if-error=0",
      cloudflareCdnCacheControl:
        "public, max-age=15, must-revalidate, stale-if-error=0",
    });
    expect(headers.cloudflareCdnCacheControl).not.toContain(
      "stale-while-revalidate",
    );
  });

  test("残り有効時間が15秒未満ならその秒数までにする", () => {
    expect(
      publishedAnalysisCacheHeaders(new Date("2026-07-15T00:00:14.900Z"), now),
    ).toEqual({
      cacheControl: "public, max-age=14, must-revalidate, stale-if-error=0",
      cloudflareCdnCacheControl:
        "public, max-age=14, must-revalidate, stale-if-error=0",
    });
  });

  test("期限切れだけは保存しない", () => {
    for (const expiresAt of [
      new Date("2026-07-15T00:00:00.000Z"),
      new Date("2026-07-14T23:59:59.000Z"),
    ]) {
      expect(publishedAnalysisCacheHeaders(expiresAt, now)).toEqual({
        cacheControl: "no-store",
      });
    }
  });
});
