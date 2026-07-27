import { describe, expect, test } from "bun:test";
import type { AdviceCard } from "~/modules/analysis/contracts.js";
import { projectPublishedFindings } from "./published-finding-projection.js";
import { ShareProjectionError } from "./share-projection-value.js";

function card(id: string, options: Partial<AdviceCard> = {}): AdviceCard {
  return {
    id,
    title: id,
    severity: 0.125,
    description: id,
    practice: id,
    evidence: [{ frame: 1, label: id }],
    ...options,
  } as AdviceCard;
}

describe("published finding projection", () => {
  test("catalog順へ並べ、既定assessmentと整数表現へ射影する", () => {
    expect(
      projectPublishedFindings([
        card("big_hits", {
          kind: "observation",
          evidence: [
            { frame: 1, label: "a" },
            { frame: 2, label: "b" },
          ],
        }),
        card("anti_air", { kind: "diagnosis" }),
        card("burnout", { kind: undefined }),
      ]),
    ).toEqual([
      {
        kind: "anti_air",
        assessment: "diagnosis",
        occurrences: 1,
        severityBp: 1250,
      },
      {
        kind: "burnout",
        assessment: "observation",
        occurrences: 1,
        severityBp: 1250,
      },
      {
        kind: "big_hits",
        assessment: "observation",
        occurrences: 2,
        severityBp: 1250,
      },
    ]);
  });

  test("未対応・重複・証拠なしのfindingを拒否する", () => {
    expect(() => projectPublishedFindings([card("future")])).toThrow(
      "未対応の指摘種別です: future",
    );
    expect(() =>
      projectPublishedFindings([card("anti_air"), card("anti_air")]),
    ).toThrow("指摘種別が重複しています: anti_air");
    expect(() =>
      projectPublishedFindings([card("anti_air", { evidence: [] })]),
    ).toThrow("証拠のない指摘は共有できません: anti_air");
    expect(() =>
      projectPublishedFindings([card("anti_air", { evidence: [] })]),
    ).toThrow(ShareProjectionError);
  });

  test("不正なseverityをfinding ID付きで拒否する", () => {
    expect(() =>
      projectPublishedFindings([card("anti_air", { severity: Number.NaN })]),
    ).toThrow("anti_air.severity が不正です。");
  });
});
