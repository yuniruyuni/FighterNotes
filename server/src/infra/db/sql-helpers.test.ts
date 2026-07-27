import { describe, expect, test } from "bun:test";
import { and, defineSpecs, not, or } from "../../models/common";
import { sql } from "./sql";
import { compToSQL } from "./sql-helpers";

const specs = defineSpecs({
  ById: (id: string) => ({ id }),
  ActiveAt: (at: Date) => ({ at }),
});

type SpecData = { type: "ById"; id: string } | { type: "ActiveAt"; at: Date };

function convert(spec: SpecData) {
  switch (spec.type) {
    case "ById":
      return sql`id = ${spec.id}`;
    case "ActiveAt":
      return sql`expires_at > ${spec.at}`;
  }
}

describe("compToSQL", () => {
  test("nested AND / OR / NOTをparameterized SQLへ変換する", () => {
    const at = new Date("2026-07-15T00:00:00.000Z");
    const spec = specs
      .ById("a")
      .and(specs.ActiveAt(at).or(specs.ById("b").not()));
    const fragment = compToSQL(spec, convert);

    expect(fragment.query).toBe(
      "(id = ? AND (expires_at > ? OR NOT (id = ?)))",
    );
    expect(fragment.params).toEqual(["a", at, "b"]);
  });

  test("empty ANDはtrue、empty ORはfalseになる", () => {
    expect(compToSQL(and<SpecData>(), convert)).toEqual({
      query: "1=1",
      params: [],
    });
    expect(compToSQL(or<SpecData>(), convert)).toEqual({
      query: "1=0",
      params: [],
    });
    expect(compToSQL(not(and<SpecData>()), convert)).toEqual({
      query: "NOT (1=1)",
      params: [],
    });
  });
});
