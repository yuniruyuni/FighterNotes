import { describe, expect, test } from "bun:test";
import type { Database } from "./database";
import { PostgresSharingRateLimit } from "./shared-rate-limit";
import type { SQLFragment } from "./sql";

describe("PostgresSharingRateLimit", () => {
  test("uses an atomic upsert and never sends the plaintext client key", async () => {
    let captured: SQLFragment | undefined;
    const db = {
      async queryGet(fragment: SQLFragment) {
        captured = fragment;
        return { allowed: false, retry_after_seconds: "42" };
      },
    } as Database;
    const limiter = new PostgresSharingRateLimit(db);

    expect(await limiter.consume("create", "203.0.113.10", 10)).toEqual({
      allowed: false,
      retryAfterSeconds: 42,
    });
    expect(captured?.query).toContain("ON CONFLICT");
    expect(captured?.query).toContain("LEAST");
    expect(captured?.params).not.toContain("203.0.113.10");
    expect(captured?.params).toContain(
      "631f08140b24b7274d12df3c37a1a80ce5876dafd7007d772e0114fddf88b682",
    );
  });

  test("fails closed when the store returns no decision", async () => {
    const db = { queryGet: async () => null } as unknown as Database;
    await expect(
      new PostgresSharingRateLimit(db).consume("public_read", "unknown", 120),
    ).rejects.toThrow("returned no row");
  });
});
