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
    for (const rulesetVersion of [9, 999]) {
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
});
