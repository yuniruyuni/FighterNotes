import { describe, expect, test } from "bun:test";
import type {
  AdviceReport,
  OpponentMoveStat,
  TacticStats,
} from "~/modules/analysis/contracts.js";
import {
  type MatchupGuideCatalog,
  selectMatchupGuide,
} from "./matchup-guide.js";

function report(over: {
  moves?: Array<{ name: string; touches: number }>;
  stats?: Partial<TacticStats>;
  cardIds?: string[];
}): AdviceReport {
  return {
    cards: (over.cardIds ?? []).map((id) => ({ id })),
    opponent_move_stats: (over.moves ?? []).map(
      (move) =>
        ({
          name: move.name,
          projectile: false,
          touches: move.touches,
          hits_taken: 0,
          hp_lost: 0,
          blocked: 0,
          punished: 0,
          punish_missed: 0,
        }) satisfies OpponentMoveStat,
    ),
    tactic_stats: (over.stats ?? {}) as TacticStats,
  } as AdviceReport;
}

const catalog: MatchupGuideCatalog = {
  KEN: [
    {
      id: "jinrai",
      triggers: [{ kind: "move", notations: ["236LK", "236MK"] }],
      text: "迅雷の手引き",
    },
    {
      id: "rush-or-throw",
      triggers: [
        { kind: "stat", stat: "raw_drive_rushes_faced" },
        { kind: "stat", stat: "throws_taken" },
      ],
      text: "ラッシュか投げの手引き",
    },
    {
      id: "jump",
      triggers: [{ kind: "card", id: "own_jumps" }],
      text: "飛びの手引き",
    },
  ],
};

describe("matchup guide selection", () => {
  test("相手キャラ未指定(空文字)・手引きの無いキャラは空", () => {
    const observed = report({ moves: [{ name: "236LK", touches: 1 }] });
    expect(selectMatchupGuide(catalog, observed, "")).toEqual([]);
    expect(selectMatchupGuide(catalog, observed, "RYU")).toEqual([]);
  });

  test("観測された記譜の項目だけを、カタログの記載順で返す", () => {
    const observed = report({
      moves: [{ name: "236MK", touches: 1 }],
      stats: { raw_drive_rushes_faced: 1 },
    });
    expect(
      selectMatchupGuide(catalog, observed, "KEN").map((item) => item.id),
    ).toEqual(["jinrai", "rush-or-throw"]);
  });

  test("記譜が一致しない・触られていない技では出さない", () => {
    expect(
      selectMatchupGuide(
        catalog,
        report({ moves: [{ name: "236HK", touches: 3 }] }),
        "KEN",
      ),
    ).toEqual([]);
    expect(
      selectMatchupGuide(
        catalog,
        report({ moves: [{ name: "236LK", touches: 0 }] }),
        "KEN",
      ),
    ).toEqual([]);
  });

  test("stat の引き金は遭遇 1 以上。どちらか片方で足りる(OR)", () => {
    expect(
      selectMatchupGuide(
        catalog,
        report({ stats: { raw_drive_rushes_faced: 0, throws_taken: 0 } }),
        "KEN",
      ),
    ).toEqual([]);
    expect(
      selectMatchupGuide(
        catalog,
        report({ stats: { throws_taken: 1 } }),
        "KEN",
      ).map((item) => item.id),
    ).toEqual(["rush-or-throw"]);
    // 旧レポートで field 自体が無ければ観測なしとして扱う。
    expect(selectMatchupGuide(catalog, report({}), "KEN")).toEqual([]);
  });

  test("カードの引き金は id の一致で判定する", () => {
    expect(
      selectMatchupGuide(
        catalog,
        report({ cardIds: ["anti_air", "own_jumps"] }),
        "KEN",
      ).map((item) => item.id),
    ).toEqual(["jump"]);
    expect(
      selectMatchupGuide(catalog, report({ cardIds: ["anti_air"] }), "KEN"),
    ).toEqual([]);
  });

  test("opponent_move_stats を持たない旧レポートでも落ちない", () => {
    const legacy = report({});
    legacy.opponent_move_stats = undefined;
    expect(selectMatchupGuide(catalog, legacy, "KEN")).toEqual([]);
  });
});
