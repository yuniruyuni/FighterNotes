import { describe, expect, test } from "bun:test";
import { FixedWindowRateLimiter, requestClientKey } from "./rate-limit";

describe("share create rate limit", () => {
  test("同じキーを固定窓内の上限で拒否し、窓の終了後に再開する", () => {
    const limiter = new FixedWindowRateLimiter(2);
    expect(limiter.consume("client", 1_000).allowed).toBe(true);
    expect(limiter.consume("client", 2_000).allowed).toBe(true);
    expect(limiter.consume("client", 3_000)).toEqual({
      allowed: false,
      retryAfterSeconds: 58,
    });
    expect(limiter.consume("client", 61_000).allowed).toBe(true);
  });

  test("Cloudflareが上書きする接続元IPだけを信頼する", () => {
    expect(
      requestClientKey(
        new Headers({
          "CF-Connecting-IP": "203.0.113.10",
          "X-Forwarded-For": "198.51.100.2, 198.51.100.3",
        }),
      ),
    ).toBe("203.0.113.10");
    expect(
      requestClientKey(
        new Headers({ "X-Forwarded-For": "198.51.100.2, 198.51.100.3" }),
      ),
    ).toBe("unknown");
    expect(requestClientKey(new Headers())).toBe("unknown");
  });
});
