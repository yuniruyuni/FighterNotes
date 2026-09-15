import { describe, expect, test } from "bun:test";
import type { OpponentMoveStat } from "~/modules/analysis/contracts.js";
import { moveAnswer, punishSummary } from "./ReportStatsSections.js";

function move(over: Partial<OpponentMoveStat>): OpponentMoveStat {
  return {
    name: "2MK",
    projectile: false,
    touches: 1,
    hits_taken: 0,
    hp_lost: 0,
    blocked: 1,
    punished: 0,
    punish_missed: 0,
    ...over,
  };
}

describe("opponent move answer column", () => {
  test("ガード区分と実測有利、確反候補から回答を組み立てる", () => {
    expect(
      moveAnswer(
        move({ guard: "low", blocked_advantage: 8, counters: ["2LP", "MP"] }),
      ),
    ).toBe("下段(しゃがみガード) / ガード後 +8F → 2LP・MP で確反");
    expect(moveAnswer(move({ guard: "overhead" }))).toBe("中段(立ちガード)");
    expect(moveAnswer(move({ blocked_advantage: 6, counters: [] }))).toBe(
      "ガード後 +6F(実測)",
    );
    // 実測が何も無い技には嘘をつかない。
    expect(moveAnswer(move({}))).toBe("-");
  });

  test("確反の内訳に距離未確認の候補を足す", () => {
    expect(punishSummary(move({ punished: 1, punish_missed: 2 }))).toBe(
      "取った 1 / 見逃し 2",
    );
    expect(punishSummary(move({ punished: 0, punish_unconfirmed: 3 }))).toBe(
      "取った 0 / 見逃し 0 / 未確認 3",
    );
    expect(punishSummary(move({ blocked: 0 }))).toBe("-");
  });
});
