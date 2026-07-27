import { describe, expect, test } from "bun:test";
import { fail, isFail } from "./fail";

describe("Fail", () => {
  test("code・message・任意detailsをbrand付きfailureへ閉じ込める", () => {
    const details = { paths: ["rounds"] };
    const value = fail("INVALID_INPUT", "invalid payload", details);

    expect(value).toMatchObject({
      code: "INVALID_INPUT",
      message: "invalid payload",
      details,
    });
    expect(isFail(value)).toBe(true);
    expect(isFail(fail("NOT_FOUND", "missing"))).toBe(true);

    const brand = Reflect.ownKeys(value).find(
      (key): key is symbol => typeof key === "symbol",
    );
    if (!brand) throw new Error("Fail brand is missing");
    expect(isFail({ ...value, [brand]: false })).toBe(false);
  });

  test("構造が似ていてもbrandを持たない値をfailureとみなさない", () => {
    for (const value of [
      null,
      undefined,
      "error",
      1,
      { code: "INVALID_INPUT", message: "invalid payload" },
    ]) {
      expect(isFail(value)).toBe(false);
    }
  });
});
