import { describe, expect, test } from "bun:test";
import {
  type PublishedAnalysisCandidate,
  publishedAnalysisCandidateSchema,
} from "./schema";

type CandidateFinding = PublishedAnalysisCandidate["findings"][number];
type CandidateInput = Omit<PublishedAnalysisCandidate, "findings"> & {
  findings: Array<
    Omit<CandidateFinding, "assessment"> & {
      assessment?: CandidateFinding["assessment"];
    }
  >;
};

function candidate(): CandidateInput {
  return {
    rulesetVersion: 3,
    ownCharacter: "LUKE",
    opponentCharacter: "CHUN_LI",
    rounds: { detected: 1, won: 1, lost: 0, unresolved: 0 },
    findings: [
      {
        kind: "anti_air",
        assessment: "diagnosis",
        occurrences: 1,
        severityBp: 1200,
      },
    ],
    tactics: {
      antiAir: { opportunities: 1, successes: 1, jumpInsAllowed: 0 },
      driveImpact: {
        faced: 0,
        returned: 0,
        blocked: 0,
        parried: 0,
        hit: 0,
        avoided: 0,
        unconfirmed: 0,
      },
      rawDriveRush: { faced: 0, defended: 0, hit: 0, unconfirmed: 0 },
      dashThrow: { faced: 0 },
      throwWhiff: { count: 0 },
      fastestChallenge: {
        opportunities: 0,
        strikeAttempts: 0,
        strikeLosses: 0,
        throwAttempts: 0,
        throwLosses: 0,
      },
      burnout: {
        count: 0,
        durationDeciseconds: 0,
        hpLostBp: 0,
        hpDealtBp: 0,
        selfInitiated: 0,
        forced: 0,
        mixed: 0,
        unknown: 0,
      },
    },
  };
}

function issues(value: unknown) {
  const result = publishedAnalysisCandidateSchema.safeParse(value);
  if (result.success) throw new Error("expected an invalid candidate");
  return result.error.issues.map(({ code, path, message }) => ({
    code,
    path,
    message,
  }));
}

describe("publishedAnalysisCandidateSchema refinements", () => {
  test("support外rulesetのfieldと理由を返す", () => {
    for (const rulesetVersion of [2, 16, 999]) {
      expect(issues({ ...candidate(), rulesetVersion })).toEqual([
        {
          code: "custom",
          path: ["rulesetVersion"],
          message: "unsupported ruleset version",
        },
      ]);
    }
  });

  test("ruleset v3からv8を引き続き受理する", () => {
    for (const rulesetVersion of [3, 4, 5, 6, 7, 8]) {
      const value = candidate();
      value.rulesetVersion = rulesetVersion;
      expect(publishedAnalysisCandidateSchema.safeParse(value).success).toBe(
        true,
      );
    }
  });

  test("round合計不一致のfieldと理由を返す", () => {
    expect(
      issues({
        ...candidate(),
        rounds: { detected: 1, won: 1, lost: 1, unresolved: 0 },
      }),
    ).toEqual([
      {
        code: "custom",
        path: ["rounds"],
        message: "round totals do not match detected rounds",
      },
    ]);
  });

  test("重複findingのindexと理由を返す", () => {
    const value = candidate();
    value.findings.push({ ...value.findings[0] });
    expect(issues(value)).toEqual([
      {
        code: "custom",
        path: ["findings", 1, "kind"],
        message: "duplicate finding kind",
      },
    ]);
  });

  test("ruleset v6ではassessmentを要求し、指定済みなら受理する", () => {
    const value = candidate();
    value.rulesetVersion = 6;
    value.findings[0].assessment = undefined;
    expect(issues(value)).toEqual([
      {
        code: "custom",
        path: ["findings", 0, "assessment"],
        message: "assessment is required for ruleset v6 and later",
      },
    ]);

    value.findings[0].assessment = "diagnosis";
    expect(publishedAnalysisCandidateSchema.safeParse(value).success).toBe(
      true,
    );
  });

  test("ruleset v9以降にSA/CA集計を必須化し、旧rulesetの形を維持する", () => {
    for (const rulesetVersion of [3, 4, 5, 6, 7, 8]) {
      expect(
        publishedAnalysisCandidateSchema.safeParse({
          ...candidate(),
          rulesetVersion,
        }).success,
      ).toBe(true);
    }

    const aggregate = {
      own: { availability: "unavailable" as const },
      opponent: { availability: "unavailable" as const },
    };
    // v9で導入した必須化は、以降のrulesetにも同じ形で適用する。
    for (const rulesetVersion of [9, 10, 11, 12, 13, 14, 15]) {
      const required = candidate();
      required.rulesetVersion = rulesetVersion;
      expect(issues(required)).toEqual([
        {
          code: "custom",
          path: ["superArts"],
          message: "super art aggregates are required for ruleset v9 and later",
        },
      ]);
      expect(
        publishedAnalysisCandidateSchema.safeParse({
          ...required,
          superArts: aggregate,
        }).success,
      ).toBe(true);
    }

    const value = candidate();
    value.rulesetVersion = 9;
    expect(
      publishedAnalysisCandidateSchema.safeParse({
        ...value,
        superArts: {
          own: {
            availability: "partial",
            levels: { sa1: 1, sa2: 0, sa3: 0, ca: 0 },
            outcomes: {
              hit: 1,
              block: 0,
              noImmediateContact: 0,
              punished: 0,
              ko: 0,
            },
            contexts: { combo: 1, punish: 0, reversal: 0, neutral: 0 },
          },
          opponent: { availability: "unavailable" },
        },
      }).success,
    ).toBe(true);
    expect(issues({ ...candidate(), superArts: aggregate })).toContainEqual({
      code: "custom",
      path: ["superArts"],
      message: "super art aggregates are not valid before ruleset v9",
    });
  });

  test("unavailableに0値を付けず、completeは全集計を必須にする", () => {
    const value = {
      ...candidate(),
      rulesetVersion: 9,
      superArts: {
        own: { availability: "unavailable", levels: { sa1: 0 } },
        opponent: { availability: "complete" },
      },
    };
    const paths = issues(value).map((issue) => issue.path.join("."));
    expect(paths).toContain("superArts.own");
    expect(paths).toContain("superArts.opponent.levels");
    expect(paths).toContain("superArts.opponent.outcomes");
  });

  test("completeは0回を受理し、partialは検出済み使用を必須にする", () => {
    const zeroLevels = { sa1: 0, sa2: 0, sa3: 0, ca: 0 };
    const zeroOutcomes = {
      hit: 0,
      block: 0,
      noImmediateContact: 0,
      punished: 0,
      ko: 0,
    };
    const zeroContexts = { combo: 0, punish: 0, reversal: 0, neutral: 0 };
    const base = { ...candidate(), rulesetVersion: 9 };

    expect(
      publishedAnalysisCandidateSchema.safeParse({
        ...base,
        superArts: {
          own: {
            availability: "complete",
            levels: zeroLevels,
            outcomes: zeroOutcomes,
            contexts: zeroContexts,
          },
          opponent: {
            availability: "complete",
            levels: zeroLevels,
            outcomes: zeroOutcomes,
          },
        },
      }).success,
    ).toBe(true);

    expect(
      issues({
        ...base,
        superArts: {
          own: {
            availability: "partial",
            levels: zeroLevels,
            outcomes: zeroOutcomes,
            contexts: zeroContexts,
          },
          opponent: {
            availability: "partial",
            levels: zeroLevels,
            outcomes: zeroOutcomes,
          },
        },
      }),
    ).toEqual([
      {
        code: "custom",
        path: ["superArts", "own", "levels"],
        message: "partial super art aggregates require an observed use",
      },
      {
        code: "custom",
        path: ["superArts", "opponent", "levels"],
        message: "partial super art aggregates require an observed use",
      },
    ]);

    // どのlevelの1回でも「観測できた使用」として partial を成立させる。
    // 特定のlevelだけ数えていると、検出済みの使用を捨てることになる。
    for (const level of ["sa1", "sa2", "sa3", "ca"] as const) {
      expect(
        publishedAnalysisCandidateSchema.safeParse({
          ...base,
          superArts: {
            own: {
              availability: "partial",
              levels: { ...zeroLevels, [level]: 1 },
              outcomes: zeroOutcomes,
              contexts: zeroContexts,
            },
            opponent: {
              availability: "partial",
              levels: { ...zeroLevels, [level]: 1 },
              outcomes: zeroOutcomes,
            },
          },
        }).success,
      ).toBe(true);
    }
  });

  test("v9 aggregate内のdamage・gauge・frame・入力・自由文をstrictに拒否する", () => {
    const aggregate = {
      own: {
        availability: "complete",
        levels: { sa1: 1, sa2: 0, sa3: 0, ca: 0 },
        outcomes: {
          hit: 1,
          block: 0,
          noImmediateContact: 0,
          punished: 0,
          ko: 0,
        },
        contexts: { combo: 1, punish: 0, reversal: 0, neutral: 0 },
        super_damage_samples: 1,
        super_reported_combo_damage: 2500,
        super_reported_marginal_damage: 1000,
        super_low_scaling_uses: 1,
        super_gauge_end: 1.5,
        opponent_super_gauge_end: 2,
        frame: 120,
        input: "236236P",
        note: "free text",
      },
      opponent: { availability: "unavailable" },
    };
    const paths = issues({
      ...candidate(),
      rulesetVersion: 9,
      superArts: aggregate,
    }).map((issue) => issue.path.join("."));
    expect(paths).toContain("superArts.own");
  });
});
