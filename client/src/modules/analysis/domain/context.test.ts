import { describe, expect, test } from "bun:test";
import { createAnalysisContext, resolveAnalysisContext } from "./context.js";

describe("analysis context", () => {
  test("自分が2Pなら自キャラと相手キャラをP1/P2へ正規化する", () => {
    expect(createAnalysisContext("p2", "BLANKA", "DHALSIM")).toEqual({
      ownSide: "p2",
      p1: { character: "DHALSIM" },
      p2: { character: "BLANKA" },
    });
  });

  test("従来のownChar文字列を受け付ける", () => {
    expect(resolveAnalysisContext("p1", "KEN")).toEqual({
      ownSide: "p1",
      p1: { character: "KEN" },
      p2: {},
    });
  });

  test("省略したmetadataを空のplayer contextへ正規化する", () => {
    expect(createAnalysisContext("p1")).toEqual({
      ownSide: "p1",
      p1: {},
      p2: {},
    });
    expect(resolveAnalysisContext("p2")).toEqual({
      ownSide: "p2",
      p1: {},
      p2: {},
    });
  });

  test("control typeとbattle versionを欠落させない", () => {
    expect(
      createAnalysisContext("p1", "KEN", "DHALSIM", {
        ownControlType: "classic",
        opponentControlType: "modern",
        battleVersion: "2026.06",
      }),
    ).toEqual({
      ownSide: "p1",
      p1: { character: "KEN", controlType: "classic" },
      p2: { character: "DHALSIM", controlType: "modern" },
      battleVersion: "2026.06",
    });
  });

  test("明示contextでも解析引数のownSideを正とする", () => {
    expect(
      resolveAnalysisContext("p2", {
        ownSide: "p1",
        p1: { character: "DHALSIM" },
        p2: { character: "BLANKA" },
      }).ownSide,
    ).toBe("p2");
  });

  test("sideと任意metadataを正規化する", () => {
    expect(
      resolveAnalysisContext("P2", {
        p1: { character: "  KEN  ", controlType: " " },
        p2: undefined,
        battleVersion: "  2026.07  ",
      }),
    ).toEqual({
      ownSide: "p2",
      p1: { character: "KEN" },
      p2: {},
      battleVersion: "2026.07",
    });
    expect(createAnalysisContext("unknown").ownSide).toBe("p1");
  });
});
