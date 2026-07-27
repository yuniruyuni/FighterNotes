import { describe, expect, test } from "bun:test";
import { legacyFindingAssessment } from "./catalog";

describe("legacy finding assessment", () => {
  test("統計・確認場面・原因診断を旧rulesetの表示規則で復元する", () => {
    expect(legacyFindingAssessment(3, "burnout")).toBe("statistic");
    expect(legacyFindingAssessment(3, "big_hits")).toBe("observation");
    expect(legacyFindingAssessment(5, "early_hits")).toBe("observation");
    expect(legacyFindingAssessment(5, "lead_loss")).toBe("observation");
    expect(legacyFindingAssessment(4, "early_hits")).toBe("diagnosis");
    expect(legacyFindingAssessment(6, "anti_air")).toBe("diagnosis");
  });
});
