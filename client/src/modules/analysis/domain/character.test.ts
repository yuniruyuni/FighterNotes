import { describe, expect, test } from "bun:test";
import {
  CHARACTER_CATALOG,
  CHARACTER_IDS,
  formatCharacterId,
  isCharacterId,
} from "./character.js";

describe("character catalog", () => {
  test("catalogとID一覧が同じ一意な閉集合を表す", () => {
    expect(CHARACTER_IDS).toEqual(CHARACTER_CATALOG.map(({ id }) => id));
    expect(new Set(CHARACTER_IDS).size).toBe(CHARACTER_IDS.length);
    expect(CHARACTER_CATALOG.every(({ label }) => label.length > 0)).toBe(true);
  });

  test("既知のIDだけを受理する", () => {
    expect(isCharacterId("A_K_I")).toBe(true);
    expect(isCharacterId("YASMINE")).toBe(true);
    expect(isCharacterId("ZANGIEF")).toBe(true);
    expect(isCharacterId("juri")).toBe(false);
    expect(isCharacterId("UNKNOWN")).toBe(false);
  });

  test("表示名ではunderscoreをhyphenへ変換し、未指定を補う", () => {
    expect(formatCharacterId("CHUN_LI")).toBe("CHUN-LI");
    expect(formatCharacterId("KEN")).toBe("KEN");
    expect(formatCharacterId("")).toBe("未指定");
    expect(formatCharacterId(undefined)).toBe("未指定");
  });
});
