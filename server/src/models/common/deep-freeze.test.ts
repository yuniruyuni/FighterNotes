import { describe, expect, test } from "bun:test";
import { deepFreeze } from "./deep-freeze";

describe("deepFreeze", () => {
  test("objectとarrayを末端までfreezeする", () => {
    const value = {
      nested: { enabled: true },
      items: [{ count: 1 }],
    };

    expect(deepFreeze(value)).toBe(value);
    expect(Object.isFrozen(value)).toBe(true);
    expect(Object.isFrozen(value.nested)).toBe(true);
    expect(Object.isFrozen(value.items)).toBe(true);
    expect(Object.isFrozen(value.items[0])).toBe(true);
  });

  test("浅くfreeze済みの親でも子をfreezeする", () => {
    const child = { enabled: true };
    const value = Object.freeze({ child });

    expect(deepFreeze(value)).toBe(value);
    expect(Object.isFrozen(child)).toBe(true);
  });

  test("循環参照を一度ずつ辿ってfreezeする", () => {
    const value: { child: { parent?: unknown } } = { child: {} };
    value.child.parent = value;

    expect(deepFreeze(value)).toBe(value);
    expect(Object.isFrozen(value)).toBe(true);
    expect(Object.isFrozen(value.child)).toBe(true);
  });

  test("nullとprimitiveをそのまま返す", () => {
    for (const value of [null, undefined, true, 1, "value"] as const) {
      expect(deepFreeze(value)).toBe(value);
    }
  });
});
