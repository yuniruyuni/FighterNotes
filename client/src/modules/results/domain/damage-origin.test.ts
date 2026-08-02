import { describe, expect, test } from "bun:test";
import type {
  AttributedDamageEvent,
  DamageApproach,
  DamageContact,
  DamageOrigin,
  StrikeKind,
} from "~/modules/analysis/contracts.js";
import { summarizeDamageOrigins } from "./damage-origin.js";

function damage(
  sequence: number,
  round: number,
  hpDrop: number,
  origin: DamageOrigin,
  strikeKind?: StrikeKind,
  approach?: DamageApproach,
  contact?: DamageContact,
): AttributedDamageEvent {
  return {
    sequence_no: sequence,
    round_no: round,
    start_frame: sequence * 100,
    end_frame: sequence * 100 + 40,
    scene_frame: sequence * 100 - 10,
    hp_before: 1,
    hp_after: 1 - hpDrop,
    hp_drop: hpDrop,
    origin,
    confidence: origin === "unclassified" ? "low" : "high",
    ...(strikeKind ? { strike_kind: strikeKind } : {}),
    ...(approach ? { approach } : {}),
    ...(contact ? { contact, contact_confidence: "high" as const } : {}),
    contexts: [],
  };
}

describe("damage origin summary", () => {
  test("全ラウンドの被ダメージを分類し、構成比を合計100%へ配分する", () => {
    const summary = summarizeDamageOrigins(
      [
        damage(1, 1, 0.6, "throw"),
        damage(2, 1, 0.4, "strike", "low"),
        damage(3, 2, 0.5, "unclassified"),
      ],
      "all",
    );

    expect(summary.totalHpLost).toBeCloseTo(1.5);
    expect(summary.classifiedHpLost).toBeCloseTo(1);
    expect(summary.classifiedPercent).toBeCloseTo(66.666, 2);
    expect(summary.rows.map(({ key }) => key)).toEqual([
      "throw",
      "unclassified",
      "strike_low",
    ]);
    expect(
      summary.rows.reduce((sum, row) => sum + row.compositionPercent, 0),
    ).toBe(100);
  });

  test("roundを絞り、無効なdamage値を除外する", () => {
    const summary = summarizeDamageOrigins(
      [
        damage(1, 1, 0.2, "throw"),
        damage(2, 2, 0.5, "unclassified"),
        damage(3, 2, 0, "strike"),
        damage(4, 2, -0.1, "strike"),
        damage(5, 2, Number.NaN, "strike"),
      ],
      2,
    );

    expect(summary).toMatchObject({
      totalHpLost: 0.5,
      classifiedHpLost: 0,
      classifiedPercent: 0,
    });
    expect(summary.rows).toHaveLength(1);
    expect(summary.rows[0].compositionPercent).toBe(100);
  });

  test("打撃属性を個別分類し、同値の端数を決定的に配分する", () => {
    const summary = summarizeDamageOrigins(
      [
        damage(1, 1, 1, "strike", "high"),
        damage(2, 1, 1, "strike", "overhead"),
        damage(3, 1, 1, "strike", "low"),
        damage(4, 1, 1, "strike", "air"),
        damage(5, 1, 1, "strike"),
      ],
      "all",
    );

    expect(summary.rows.map(({ label }) => label)).toEqual([
      "下段",
      "空中攻撃",
      "上段",
      "打撃（属性不明）",
      "中段",
    ]);
    expect(
      summary.rows.map(({ compositionPercent }) => compositionPercent),
    ).toEqual([20, 20, 20, 20, 20]);
  });

  test("全originの表示名を契約として保持する", () => {
    const origins: DamageOrigin[] = [
      "compound_threat",
      "teleport",
      "throw",
      "drive_impact",
      "raw_drive_rush",
      "own_jump_caught",
      "opponent_jump_in",
      "projectile",
      "strike",
      "unclassified",
    ];
    const summary = summarizeDamageOrigins(
      origins.map((origin, index) => damage(index + 1, 1, 1, origin)),
      "all",
    );

    expect(
      Object.fromEntries(summary.rows.map((row) => [row.origin, row.label])),
    ).toEqual({
      compound_threat: "弾＋テレポート",
      teleport: "テレポート",
      throw: "投げ",
      drive_impact: "ドライブインパクト",
      raw_drive_rush: "生ドライブラッシュ",
      own_jump_caught: "ジャンプを狩られた",
      opponent_jump_in: "相手の飛び込み",
      projectile: "飛び道具",
      strike: "打撃（属性不明）",
      unclassified: "未分類（要確認）",
    });
  });

  test("strike以外に付いたstrike kindを分類キーへ使わない", () => {
    const summary = summarizeDamageOrigins(
      [damage(1, 1, 0.2, "throw", "low"), damage(2, 1, 0.1, "strike")],
      "all",
    );

    expect(summary.rows.map(({ key, label }) => [key, label])).toEqual([
      ["throw", "投げ"],
      ["strike", "打撃（属性不明）"],
    ]);
  });

  test("接近手段と接触種別を直交した分類として保持する", () => {
    const summary = summarizeDamageOrigins(
      [
        damage(
          1,
          1,
          0.2,
          "raw_drive_rush",
          undefined,
          "raw_drive_rush",
          "throw",
        ),
        damage(2, 1, 0.1, "raw_drive_rush"),
      ],
      "all",
    );

    expect(summary.rows.map(({ key, label }) => [key, label])).toEqual([
      ["raw_drive_rush_throw", "生ドライブラッシュ→投げ"],
      ["raw_drive_rush", "生ドライブラッシュ"],
    ]);
  });

  test("最大剰余法で異なる端数と同率端数を決定的に配分する", () => {
    const unequal = summarizeDamageOrigins(
      [
        damage(1, 1, 3, "drive_impact"),
        damage(2, 1, 2, "projectile"),
        damage(3, 1, 1, "throw"),
      ],
      "all",
    );
    expect(
      unequal.rows.map(({ compositionPercent }) => compositionPercent),
    ).toEqual([50, 33.3, 16.7]);

    const tied = summarizeDamageOrigins(
      [
        damage(1, 1, 1, "drive_impact"),
        damage(2, 1, 1, "projectile"),
        damage(3, 1, 1, "throw"),
      ],
      "all",
    );
    expect(
      tied.rows.map(({ compositionPercent }) => compositionPercent),
    ).toEqual([33.4, 33.3, 33.3]);
  });

  test("有効なdamageがなければ空の集計を返す", () => {
    expect(summarizeDamageOrigins([], "all")).toEqual({
      totalHpLost: 0,
      classifiedHpLost: 0,
      classifiedPercent: 0,
      rows: [],
    });
  });
});
