import { describe, expect, test } from "bun:test";
import {
  confidenceLabel,
  formatDamageContexts,
  formatHpRatio,
  formatPercent,
} from "./damage-origin-format.js";

describe("damage origin format", () => {
  test("HP比率と百分率を小数1桁まで表示する", () => {
    expect(formatHpRatio(0.1234)).toBe("12.3%");
    expect(formatPercent(100)).toBe("100%");
  });

  test("damage contextを表示順の日本語labelへ変換する", () => {
    expect(
      formatDamageContexts([
        "mashing",
        "press_while_minus",
        "guard_break",
        "reversal_punished",
        "punish_whiff",
        "burnout",
      ]),
    ).toBe(
      "守勢のボタン押し、不利フレーム中、ガード入力崩れ、リバーサル失敗、確反空振り、バーンアウト中",
    );
    expect(formatDamageContexts([])).toBe("");
  });

  test("confidence三段階を短いlabelへ変換する", () => {
    expect(confidenceLabel("high")).toBe("高");
    expect(confidenceLabel("medium")).toBe("中");
    expect(confidenceLabel("low")).toBe("低");
  });
});
