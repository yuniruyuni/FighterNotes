import { describe, expect, test } from "bun:test";
import { syntheticTacticStats } from "~/test-support/analysis.js";
import {
  appendUnconfirmedCandidates,
  driveSpendBreakdown,
  formatDriveEfficiency,
  formatTacticCount,
  formatTacticRateWithCount,
} from "./tactic-stat-format.js";

describe("戦術統計の表示", () => {
  test("確認済みの機会がない状態をハイフンにしない", () => {
    expect(formatTacticCount(0, 0)).toBe("確認なし");
    expect(formatTacticRateWithCount(0, 0)).toBe("確認なし");
  });

  test("未確認候補と確認済みの結果を区別する", () => {
    expect(formatTacticCount(0, 0, 2)).toBe("未確認 2 件");
    expect(formatTacticCount(1, 3, 2)).toBe("1 / 3");
    expect(formatTacticRateWithCount(1, 3, 2)).toBe("33% (1/3)・未確認 2 件");
    expect(appendUnconfirmedCandidates("被弾 1 回", 2)).toBe(
      "被弾 1 回 / 未確認候補 2 件",
    );
  });

  /**
   * 解析側はゲージ全量に対する比で消費を実測している。利用者が普段数えて
   * いるのは本数なので、6本へ直してから見せる。
   */
  test("実測した消費量を本数と1本あたりの与ダメージへ直す", () => {
    const stats = syntheticTacticStats({
      // 2本（2/6 ≈ 0.333）使って 24% 与えた。
      drive_spent_on_impacts: 1 / 3,
      drive_damage_from_impacts: 0.24,
      drive_spend_samples: 2,
    });

    expect(formatDriveEfficiency(stats)).toBe("12.0%");
    expect(driveSpendBreakdown(stats)).toBe("DI 2.0本→24%");
  });

  test("消費と与ダメージを費目別に並べる", () => {
    const stats = syntheticTacticStats({
      drive_spent_on_impacts: 1 / 6,
      drive_damage_from_impacts: 0.2,
      drive_spent_on_rushes: 1 / 6,
      drive_damage_from_rushes: 0,
      drive_spend_samples: 2,
    });

    expect(driveSpendBreakdown(stats)).toBe(
      "DI 1.0本→20% / 生ラッシュ 1.0本→0%",
    );
  });

  /**
   * 実測できた行動が無いときに 0% と出すと「使ったが無駄だった」に読める。
   * 測れていないことと区別する。
   */
  test("実測できた行動が無ければ率を出さない", () => {
    const stats = syntheticTacticStats({ drive_spend_samples: 0 });

    expect(formatDriveEfficiency(stats)).toBe("確認なし");
    expect(driveSpendBreakdown(stats)).toBe(
      "ゲージ消費を実測できた行動がありません",
    );
  });
});
