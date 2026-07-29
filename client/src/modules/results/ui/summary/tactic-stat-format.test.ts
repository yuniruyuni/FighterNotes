import { describe, expect, test } from "bun:test";
import {
  appendUnconfirmedCandidates,
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
});
