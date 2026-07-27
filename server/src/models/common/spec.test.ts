import { describe, expect, test } from "bun:test";
import { and, defineSpecs, isCompLogical, not, or } from "./spec";

const specs = defineSpecs({
  ById: (id: string) => ({ id }),
  Active: (active: boolean) => ({ active }),
});

describe("specification composition", () => {
  test("factoryが型付きleafとchainableな合成メソッドを作る", () => {
    const leaf = specs.ById("analysis-1");
    expect(leaf).toMatchObject({ type: "ById", id: "analysis-1" });
    expect(isCompLogical(leaf)).toBe(false);

    expect(leaf.and(specs.Active(true))).toMatchObject({
      type: "and",
      children: [
        { type: "ById", id: "analysis-1" },
        { type: "Active", active: true },
      ],
    });
    const disjunction = leaf.or(specs.Active(false));
    expect(disjunction).toMatchObject({
      type: "or",
      children: [
        { type: "ById", id: "analysis-1" },
        { type: "Active", active: false },
      ],
    });
    expect(disjunction.not()).toMatchObject({
      type: "not",
      child: { type: "or" },
    });
  });

  test("関数形式でもAND / OR / NOTを合成できる", () => {
    const conjunction = and(specs.ById("a"), specs.Active(true));
    expect(conjunction).toMatchObject({
      type: "and",
      children: [
        { type: "ById", id: "a" },
        { type: "Active", active: true },
      ],
    });
    const disjunction = or(specs.ById("a"), specs.ById("b"));
    expect(disjunction).toMatchObject({
      type: "or",
      children: [
        { type: "ById", id: "a" },
        { type: "ById", id: "b" },
      ],
    });
    const negation = not(specs.Active(false));
    expect(negation).toMatchObject({
      type: "not",
      child: { type: "Active", active: false },
    });

    expect(isCompLogical(conjunction)).toBe(true);
    expect(isCompLogical(disjunction)).toBe(true);
    expect(isCompLogical(negation)).toBe(true);
  });

  test("logical node以外のruntime値を拒否する", () => {
    for (const value of [
      null,
      undefined,
      "and",
      1,
      {},
      { type: "" },
      { type: "xor" },
    ]) {
      expect(isCompLogical(value as never)).toBe(false);
    }
  });
});
