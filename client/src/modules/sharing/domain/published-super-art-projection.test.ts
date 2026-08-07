import { describe, expect, test } from "bun:test";
import type { TacticStats } from "~/modules/analysis/contracts.js";
import { syntheticTacticStats } from "~/test-support/analysis.js";
import { projectPublishedSuperArts } from "./published-super-art-projection.js";
import { ShareProjectionError } from "./share-projection-value.js";

/**
 * SA/CA集計は ruleset v9 以降で必須になる項目で、`TacticStats` 上は任意。
 * 共通fixtureは持たないため、ここで観測できた状態の最小形を組み立てる。
 */
function stats(overrides: Partial<TacticStats> = {}): TacticStats {
  return syntheticTacticStats({
    super_art_stats_complete: false,
    opponent_super_art_stats_complete: false,
    sa1_used: 0,
    sa2_used: 0,
    sa3_used: 0,
    ca_used: 0,
    super_hits: 0,
    super_blocked: 0,
    super_no_immediate_contact: 0,
    super_punished: 0,
    super_kos: 0,
    super_combo_uses: 0,
    super_punish_uses: 0,
    super_reversal_uses: 0,
    super_neutral_uses: 0,
    opponent_sa1_used: 0,
    opponent_sa2_used: 0,
    opponent_sa3_used: 0,
    opponent_ca_used: 0,
    opponent_super_hits: 0,
    opponent_super_blocked: 0,
    opponent_super_no_immediate_contact: 0,
    opponent_super_punished: 0,
    opponent_super_kos: 0,
    ...overrides,
  });
}

describe("published super art projection", () => {
  /**
   * 全ラウンドのゲージ観測被覆を満たした側だけが complete で、0回を確定できる。
   */
  test("completeは0回でも全集計を保持する", () => {
    const result = projectPublishedSuperArts(
      stats({
        super_art_stats_complete: true,
        opponent_super_art_stats_complete: true,
      }),
    );

    expect(result.own).toMatchObject({
      availability: "complete",
      levels: { sa1: 0, sa2: 0, sa3: 0, ca: 0 },
    });
    expect(result.opponent).toMatchObject({ availability: "complete" });
  });

  /**
   * 被覆が足りなくても検出できた使用があれば、下限として partial で公開する。
   */
  test("被覆不足でも検出済み使用があればpartialにする", () => {
    const result = projectPublishedSuperArts(
      stats({
        super_art_stats_complete: false,
        opponent_super_art_stats_complete: false,
        sa2_used: 1,
        opponent_ca_used: 2,
      }),
    );

    expect(result.own.availability).toBe("partial");
    expect(result.opponent.availability).toBe("partial");
  });

  /**
   * 被覆も検出材料も無い側は件数を持たない。0を入れると「使わなかった」と
   * 読めてしまうため、availability 以外の項目を出さないことまで確認する。
   */
  test("材料が無い側はavailabilityだけを公開する", () => {
    const result = projectPublishedSuperArts(
      stats({
        super_art_stats_complete: false,
        opponent_super_art_stats_complete: false,
      }),
    );

    expect(result.own).toEqual({ availability: "unavailable" });
    expect(result.opponent).toEqual({ availability: "unavailable" });
  });

  /**
   * どのlevelの使用でもpartialへ引き上がる。特定のlevelだけ見ていると
   * 検出済みの使用を落としてunavailableにしてしまう。
   */
  test("どのlevelの使用でもpartialへ引き上げる", () => {
    for (const level of [
      "sa1_used",
      "sa2_used",
      "sa3_used",
      "ca_used",
    ] as const) {
      const result = projectPublishedSuperArts(
        stats({
          super_art_stats_complete: false,
          [level]: 1,
        }),
      );
      expect(result.own.availability).toBe("partial");
    }
  });

  /**
   * 利用文脈は自分側だけの公開契約。相手側へ出すと観測範囲を超えた
   * 推定を公開することになる。
   */
  test("利用文脈を自分側だけに付ける", () => {
    const result = projectPublishedSuperArts(
      stats({
        super_art_stats_complete: true,
        opponent_super_art_stats_complete: true,
        super_combo_uses: 3,
        super_punish_uses: 2,
        super_reversal_uses: 1,
        super_neutral_uses: 4,
      }),
    );

    expect(result.own).toMatchObject({
      contexts: { combo: 3, punish: 2, reversal: 1, neutral: 4 },
    });
    expect(result.opponent).not.toHaveProperty("contexts");
  });

  /**
   * 自分側と相手側で読む統計を取り違えると、相手の集計が自分の欄へ出る。
   * 値を非対称にして、どちらの側からも取り違えを検出できるようにする。
   */
  test("自分側と相手側の集計を取り違えない", () => {
    const result = projectPublishedSuperArts(
      stats({
        super_art_stats_complete: true,
        opponent_super_art_stats_complete: true,
        sa1_used: 1,
        sa2_used: 2,
        sa3_used: 3,
        ca_used: 4,
        super_hits: 5,
        super_blocked: 6,
        super_no_immediate_contact: 7,
        super_punished: 8,
        super_kos: 9,
        opponent_sa1_used: 11,
        opponent_sa2_used: 12,
        opponent_sa3_used: 13,
        opponent_ca_used: 14,
        opponent_super_hits: 15,
        opponent_super_blocked: 16,
        opponent_super_no_immediate_contact: 17,
        opponent_super_punished: 18,
        opponent_super_kos: 19,
      }),
    );

    expect(result.own).toMatchObject({
      levels: { sa1: 1, sa2: 2, sa3: 3, ca: 4 },
      outcomes: {
        hit: 5,
        block: 6,
        noImmediateContact: 7,
        punished: 8,
        ko: 9,
      },
    });
    expect(result.opponent).toMatchObject({
      levels: { sa1: 11, sa2: 12, sa3: 13, ca: 14 },
      outcomes: {
        hit: 15,
        block: 16,
        noImmediateContact: 17,
        punished: 18,
        ko: 19,
      },
    });
  });

  /**
   * 拒否したときは、どの項目が壊れていたかを field path で名指しする。
   * 名前が違うと、共有できない原因を追えない。
   */
  test("壊れている項目をfield pathで名指しする", () => {
    const complete = {
      super_art_stats_complete: true,
      opponent_super_art_stats_complete: true,
    } as const;
    // completeness は真偽値、件数は数値。別々の理由として報告する。
    const completeness: Array<[keyof TacticStats, string]> = [
      ["super_art_stats_complete", "superArts.own.availability"],
      ["opponent_super_art_stats_complete", "superArts.opponent.availability"],
    ];
    for (const [key, field] of completeness) {
      expect(() =>
        projectPublishedSuperArts(stats({ ...complete, [key]: undefined })),
      ).toThrow(`${field} が不正です。`);
    }

    const cases: Array<[keyof TacticStats, string]> = [
      ["sa1_used", "superArts.own.levels.sa1"],
      ["sa2_used", "superArts.own.levels.sa2"],
      ["sa3_used", "superArts.own.levels.sa3"],
      ["ca_used", "superArts.own.levels.ca"],
      ["super_hits", "superArts.own.outcomes.hit"],
      ["super_blocked", "superArts.own.outcomes.block"],
      [
        "super_no_immediate_contact",
        "superArts.own.outcomes.noImmediateContact",
      ],
      ["super_punished", "superArts.own.outcomes.punished"],
      ["super_kos", "superArts.own.outcomes.ko"],
      ["super_combo_uses", "superArts.own.contexts.combo"],
      ["super_punish_uses", "superArts.own.contexts.punish"],
      ["super_reversal_uses", "superArts.own.contexts.reversal"],
      ["super_neutral_uses", "superArts.own.contexts.neutral"],
      ["opponent_sa1_used", "superArts.opponent.levels.sa1"],
      ["opponent_sa2_used", "superArts.opponent.levels.sa2"],
      ["opponent_sa3_used", "superArts.opponent.levels.sa3"],
      ["opponent_ca_used", "superArts.opponent.levels.ca"],
      ["opponent_super_hits", "superArts.opponent.outcomes.hit"],
      ["opponent_super_blocked", "superArts.opponent.outcomes.block"],
      [
        "opponent_super_no_immediate_contact",
        "superArts.opponent.outcomes.noImmediateContact",
      ],
      ["opponent_super_punished", "superArts.opponent.outcomes.punished"],
      ["opponent_super_kos", "superArts.opponent.outcomes.ko"],
    ];

    for (const [key, field] of cases) {
      expect(() =>
        projectPublishedSuperArts(stats({ ...complete, [key]: undefined })),
      ).toThrow(`${field} が数値ではありません。`);
    }
  });

  /**
   * 欠測を completeness や件数として黙って通すと、観測できていない値を
   * 公開してしまう。両方とも型で弾く。
   */
  test("completenessと件数の型が壊れていれば公開しない", () => {
    expect(() =>
      projectPublishedSuperArts(
        stats({
          super_art_stats_complete: undefined as unknown as boolean,
        }),
      ),
    ).toThrow(ShareProjectionError);

    expect(() =>
      projectPublishedSuperArts(
        stats({
          super_art_stats_complete: true,
          sa1_used: undefined as unknown as number,
        }),
      ),
    ).toThrow(ShareProjectionError);

    expect(() =>
      projectPublishedSuperArts(
        stats({
          super_art_stats_complete: true,
          opponent_super_art_stats_complete: "yes" as unknown as boolean,
        }),
      ),
    ).toThrow(ShareProjectionError);
  });
});
