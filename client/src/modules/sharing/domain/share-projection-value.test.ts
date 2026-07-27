import { describe, expect, test } from "bun:test";
import {
  boundedInteger,
  ShareProjectionError,
  scaledInteger,
} from "./share-projection-value.js";

describe("share projection values", () => {
  test("非負の値を丸めて上限内へ収める", () => {
    expect(boundedInteger(1.6, 10, "count")).toBe(2);
    expect(boundedInteger(99, 10, "count")).toBe(10);
    expect(scaledInteger(0.12345, 10_000, 2_000, "ratio")).toBe(1235);
    expect(scaledInteger(2, 1_000, 500, "ratio")).toBe(500);
  });

  test("負数と非有限値をfield名付きで拒否する", () => {
    for (const value of [-1, Number.NaN, Number.POSITIVE_INFINITY]) {
      expect(() => boundedInteger(value, 10, "count")).toThrow(
        ShareProjectionError,
      );
      expect(() => scaledInteger(value, 10, 100, "ratio")).toThrow(
        /ratio が不正/,
      );
    }
    expect(new ShareProjectionError("invalid")).toMatchObject({
      name: "ShareProjectionError",
      message: "invalid",
    });
  });
});
