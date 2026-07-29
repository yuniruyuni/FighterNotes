import { describe, expect, test } from "bun:test";
import type {
  AdviceReport,
  AnalysisContext,
  TacticStats,
} from "~/modules/analysis/contracts.js";
import {
  type AnalysisHistoryRecord,
  aggregateMatchups,
  createAnalysisHistoryRecord,
  defensiveResponseBias,
  rate,
} from "./history.js";

const emptyTactics = (): TacticStats => ({
  anti_air_opportunities: 0,
  anti_air_successes: 0,
  jump_ins_allowed: 0,
  di_faced: 0,
  di_returned: 0,
  di_blocked: 0,
  di_parried: 0,
  di_hit: 0,
  di_avoided: 0,
  di_unconfirmed: 0,
  raw_drive_rushes_faced: 0,
  raw_drive_rushes_defended: 0,
  raw_drive_rushes_hit: 0,
  raw_drive_rushes_unconfirmed: 0,
  dash_throws_faced: 0,
  throw_whiffs: 0,
  minus_defense_opportunities: 0,
  fastest_strike_challenges: 0,
  fastest_strike_losses: 0,
  fastest_throw_challenges: 0,
  fastest_throw_losses: 0,
  burnout_count: 0,
  burnout_seconds: 0,
  burnout_hp_lost: 0,
  burnout_hp_dealt: 0,
  burnout_self_initiated: 0,
  burnout_forced: 0,
  burnout_mixed: 0,
  burnout_unknown: 0,
});

function record(
  id: string,
  opponentCharacter: string,
  tactics: Partial<TacticStats>,
  rulesetVersion = 2,
): AnalysisHistoryRecord {
  return {
    id,
    createdAt: "2026-07-12T00:00:00Z",
    rulesetVersion,
    ownCharacter: "LUKE",
    opponentCharacter,
    rounds: 3,
    tactics: { ...emptyTactics(), ...tactics },
  };
}

describe("analysis history aggregation", () => {
  test("ファイル名を含まない動画識別子と2P視点を永続化用レコードへ正規化する", async () => {
    const context: AnalysisContext = {
      ownSide: "p2",
      p1: { character: "CHUN_LI" },
      p2: { character: "LUKE" },
    };
    const video = new File([new Uint8Array(1024)], "private-match-name.mp4", {
      lastModified: 42,
    });
    const history = await createAnalysisHistoryRecord(
      video,
      context,
      {
        ruleset_version: 6,
        rounds_detected: 3,
        tactic_stats: emptyTactics(),
      } as AdviceReport,
      new Date("2026-07-22T00:00:00.000Z"),
    );

    expect(history).toMatchObject({
      createdAt: "2026-07-22T00:00:00.000Z",
      rulesetVersion: 6,
      ownCharacter: "LUKE",
      opponentCharacter: "CHUN_LI",
      rounds: 3,
    });
    expect(history.id).toMatch(/^v2:[0-9a-f]{64}$/);
    expect(history.id).not.toContain(video.name);

    const renamedHistory = await createAnalysisHistoryRecord(
      new File([new Uint8Array(1024)], "renamed.mp4", {
        lastModified: 42,
      }),
      context,
      {
        ruleset_version: 6,
        rounds_detected: 3,
        tactic_stats: emptyTactics(),
      } as AdviceReport,
      new Date("2026-07-22T00:00:00.000Z"),
    );
    expect(renamedHistory.id).toBe(history.id);
  });

  test("1P視点とキャラクター未指定を永続化用レコードへ反映する", async () => {
    const history = await createAnalysisHistoryRecord(
      { size: 1, lastModified: 2 },
      { ownSide: "p1", p1: {}, p2: { character: "KEN" } },
      {
        ruleset_version: 6,
        rounds_detected: 1,
        tactic_stats: emptyTactics(),
      } as AdviceReport,
      new Date("2026-07-22T00:00:00.000Z"),
    );

    expect(history).toMatchObject({
      ownCharacter: "未指定",
      opponentCharacter: "KEN",
    });
    expect(history.id).toMatch(/^v2:[0-9a-f]{64}$/);

    const missingOpponent = await createAnalysisHistoryRecord(
      { size: 1, lastModified: 2 },
      { ownSide: "p1", p1: { character: "LUKE" }, p2: {} },
      {
        ruleset_version: 6,
        rounds_detected: 1,
        tactic_stats: emptyTactics(),
      } as AdviceReport,
      new Date("2026-07-22T00:00:00.000Z"),
    );
    expect(missingOpponent.opponentCharacter).toBe("未指定");
    expect(missingOpponent.id).toMatch(/^v2:[0-9a-f]{64}$/);
    expect(missingOpponent.id).not.toBe(history.id);
  });

  test("同じ組み合わせだけを合算する", () => {
    const summaries = aggregateMatchups(
      [
        record("a", "CHUN_LI", {
          anti_air_opportunities: 2,
          anti_air_successes: 1,
        }),
        record("b", "CHUN_LI", {
          anti_air_opportunities: 3,
          anti_air_successes: 2,
        }),
        record("c", "DHALSIM", { di_faced: 2, di_returned: 1 }),
      ],
      2,
    );
    expect(summaries).toHaveLength(2);
    expect(summaries[0]).toMatchObject({
      opponentCharacter: "CHUN_LI",
      matches: 2,
      antiAirOpportunities: 5,
      antiAirSuccesses: 3,
      rawRushesDefended: 0,
    });
  });

  test("全集約fieldを複数レコードから加算する", () => {
    const populated: Partial<TacticStats> = {
      anti_air_opportunities: 1,
      anti_air_successes: 2,
      di_faced: 3,
      di_returned: 4,
      di_unconfirmed: 5,
      raw_drive_rushes_faced: 6,
      raw_drive_rushes_defended: 7,
      raw_drive_rushes_hit: 8,
      raw_drive_rushes_unconfirmed: 9,
      minus_defense_opportunities: 10,
      fastest_strike_challenges: 11,
      fastest_strike_losses: 12,
      fastest_throw_challenges: 13,
      fastest_throw_losses: 14,
      burnout_count: 15,
      burnout_seconds: 16,
      burnout_hp_lost: 17,
      burnout_hp_dealt: 18,
    };
    const [summary] = aggregateMatchups(
      [record("a", "KEN", populated), record("b", "KEN", populated)],
      2,
    );

    expect(summary).toEqual({
      key: "LUKE\u0000KEN",
      ownCharacter: "LUKE",
      opponentCharacter: "KEN",
      matches: 2,
      rounds: 6,
      antiAirOpportunities: 2,
      antiAirSuccesses: 4,
      diFaced: 6,
      diReturned: 8,
      diUnconfirmed: 10,
      rawRushesFaced: 12,
      rawRushesDefended: 14,
      rawRushesHit: 16,
      rawRushesUnconfirmed: 18,
      minusDefenseOpportunities: 20,
      fastestStrikeChallenges: 22,
      fastestStrikeLosses: 24,
      fastestThrowChallenges: 26,
      fastestThrowLosses: 28,
      burnoutCount: 30,
      burnoutSeconds: 32,
      burnoutHpLost: 34,
      burnoutHpDealt: 36,
    });
  });

  test("異なるルール世代を混ぜない", () => {
    const summaries = aggregateMatchups(
      [record("a", "CHUN_LI", {}, 1), record("b", "CHUN_LI", {}, 2)],
      2,
    );
    expect(summaries[0].matches).toBe(1);
  });

  test("同数試合の組み合わせを相手キャラクター名で安定して並べる", () => {
    const summaries = aggregateMatchups(
      [record("a", "KEN", {}), record("b", "CHUN_LI", {})],
      2,
    );
    expect(summaries.map(({ opponentCharacter }) => opponentCharacter)).toEqual(
      ["CHUN_LI", "KEN"],
    );
  });

  test("分母ゼロは率を断定しない", () => {
    expect(rate(0, 0)).toBe("-");
    expect(rate(3, 4)).toBe("75%");
    expect(rate(2, 3)).toBe("67%");
  });

  test("複数試合でも単発の読み負けではなく回答偏重だけを返す", () => {
    const [summary] = aggregateMatchups(
      [
        record("a", "CHUN_LI", {
          minus_defense_opportunities: 3,
          fastest_strike_challenges: 2,
          fastest_strike_losses: 1,
        }),
        record("b", "CHUN_LI", {
          minus_defense_opportunities: 5,
          fastest_strike_challenges: 4,
          fastest_strike_losses: 2,
        }),
      ],
      2,
    );
    expect(defensiveResponseBias(summary)).toEqual({
      action: "strike",
      opportunities: 8,
      selections: 6,
      losses: 3,
      selectionPercent: 75,
    });

    const [singleLoss] = aggregateMatchups(
      [
        record("c", "CHUN_LI", {
          minus_defense_opportunities: 4,
          fastest_strike_challenges: 3,
          fastest_strike_losses: 1,
        }),
      ],
      2,
    );
    expect(defensiveResponseBias(singleLoss)).toBeNull();
  });

  test("機会不足を除外し、複数の偏りから損失数と選択数で代表を選ぶ", () => {
    const [base] = aggregateMatchups([record("a", "KEN", {})], 2);
    expect(
      defensiveResponseBias({ ...base, minusDefenseOpportunities: 3 }),
    ).toBeNull();

    expect(
      defensiveResponseBias({
        ...base,
        minusDefenseOpportunities: 10,
        fastestStrikeChallenges: 7,
        fastestStrikeLosses: 2,
        fastestThrowChallenges: 8,
        fastestThrowLosses: 3,
      }),
    ).toEqual({
      action: "throw",
      opportunities: 10,
      selections: 8,
      losses: 3,
      selectionPercent: 80,
    });

    expect(
      defensiveResponseBias({
        ...base,
        minusDefenseOpportunities: 10,
        fastestStrikeChallenges: 8,
        fastestStrikeLosses: 2,
        fastestThrowChallenges: 7,
        fastestThrowLosses: 2,
      })?.action,
    ).toBe("strike");
  });

  test("回答偏重の成立境界と代表選択のtie breakを固定する", () => {
    const [base] = aggregateMatchups([record("a", "KEN", {})], 2);
    expect(
      defensiveResponseBias({
        ...base,
        minusDefenseOpportunities: 4,
        fastestStrikeChallenges: 3,
        fastestStrikeLosses: 2,
      })?.action,
    ).toBe("strike");
    expect(
      defensiveResponseBias({
        ...base,
        minusDefenseOpportunities: 3,
        fastestStrikeChallenges: 3,
        fastestStrikeLosses: 2,
      }),
    ).toBeNull();
    expect(
      defensiveResponseBias({
        ...base,
        minusDefenseOpportunities: 5,
        fastestStrikeChallenges: 3,
        fastestStrikeLosses: 2,
      }),
    ).toBeNull();
    expect(
      defensiveResponseBias({
        ...base,
        minusDefenseOpportunities: 10,
        fastestStrikeChallenges: 7,
        fastestStrikeLosses: 2,
      })?.action,
    ).toBe("strike");
    expect(
      defensiveResponseBias({
        ...base,
        minusDefenseOpportunities: 10,
        fastestStrikeChallenges: 7,
        fastestStrikeLosses: 2,
        fastestThrowChallenges: 7,
        fastestThrowLosses: 3,
      })?.action,
    ).toBe("throw");
    expect(
      defensiveResponseBias({
        ...base,
        minusDefenseOpportunities: 10,
        fastestStrikeChallenges: 7,
        fastestStrikeLosses: 2,
        fastestThrowChallenges: 8,
        fastestThrowLosses: 2,
      })?.action,
    ).toBe("throw");
  });
});
