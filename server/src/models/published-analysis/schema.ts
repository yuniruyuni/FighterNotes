import { z } from "zod";
import {
  CHARACTER_IDS,
  FINDING_ASSESSMENTS,
  FINDING_KINDS,
  legacyFindingAssessment,
  MAX_COUNT,
  MAX_DURATION_DECISECONDS,
  MAX_HP_BP,
  MAX_PUBLISHED_ANALYSIS_BYTES,
  MAX_ROUNDS,
  MAX_SEVERITY_BP,
  SUPPORTED_RULESET_VERSIONS,
} from "./catalog";

const countSchema = z.number().int().min(0).max(MAX_COUNT);
const roundCountSchema = z.number().int().min(0).max(MAX_ROUNDS);

const roundStatsSchema = z.strictObject({
  detected: roundCountSchema,
  won: roundCountSchema,
  lost: roundCountSchema,
  unresolved: roundCountSchema,
});

const tacticStatsSchema = z.strictObject({
  antiAir: z.strictObject({
    opportunities: countSchema,
    successes: countSchema,
    jumpInsAllowed: countSchema,
  }),
  driveImpact: z.strictObject({
    faced: countSchema,
    returned: countSchema,
    blocked: countSchema,
    parried: countSchema,
    hit: countSchema,
    avoided: countSchema,
    unconfirmed: countSchema,
  }),
  rawDriveRush: z.strictObject({
    faced: countSchema,
    defended: countSchema,
    hit: countSchema,
    unconfirmed: countSchema,
  }),
  dashThrow: z.strictObject({ faced: countSchema }),
  throwWhiff: z.strictObject({ count: countSchema }),
  fastestChallenge: z.strictObject({
    // ruleset v3/v4 の保存候補には存在しない。正規化後は 0 として扱う。
    opportunities: countSchema.default(0),
    strikeAttempts: countSchema,
    strikeLosses: countSchema,
    throwAttempts: countSchema,
    throwLosses: countSchema,
  }),
  burnout: z.strictObject({
    count: countSchema,
    durationDeciseconds: z.number().int().min(0).max(MAX_DURATION_DECISECONDS),
    hpLostBp: z.number().int().min(0).max(MAX_HP_BP),
    hpDealtBp: z.number().int().min(0).max(MAX_HP_BP),
    selfInitiated: countSchema,
    forced: countSchema,
    mixed: countSchema,
    unknown: countSchema,
  }),
});

const superArtLevelsSchema = z.strictObject({
  sa1: countSchema,
  sa2: countSchema,
  sa3: countSchema,
  ca: countSchema,
});

const superArtOutcomesSchema = z.strictObject({
  hit: countSchema,
  block: countSchema,
  noImmediateContact: countSchema,
  punished: countSchema,
  ko: countSchema,
});

const unavailableSuperArtStatsSchema = z.strictObject({
  availability: z.literal("unavailable"),
});

const ownObservedSuperArtStatsShape = {
  levels: superArtLevelsSchema,
  outcomes: superArtOutcomesSchema,
  contexts: z.strictObject({
    combo: countSchema,
    punish: countSchema,
    reversal: countSchema,
    neutral: countSchema,
  }),
};

const ownSuperArtStatsSchema = z
  .discriminatedUnion("availability", [
    unavailableSuperArtStatsSchema,
    z.strictObject({
      availability: z.literal("complete"),
      ...ownObservedSuperArtStatsShape,
    }),
    z.strictObject({
      availability: z.literal("partial"),
      ...ownObservedSuperArtStatsShape,
    }),
  ])
  .superRefine(requireObservedPartial);

const opponentObservedSuperArtStatsShape = {
  levels: superArtLevelsSchema,
  outcomes: superArtOutcomesSchema,
};

const opponentSuperArtStatsSchema = z
  .discriminatedUnion("availability", [
    unavailableSuperArtStatsSchema,
    z.strictObject({
      availability: z.literal("complete"),
      ...opponentObservedSuperArtStatsShape,
    }),
    z.strictObject({
      availability: z.literal("partial"),
      ...opponentObservedSuperArtStatsShape,
    }),
  ])
  .superRefine(requireObservedPartial);

type SuperArtLevelCounts = {
  sa1: number;
  sa2: number;
  sa3: number;
  ca: number;
};

/**
 * `partial` は「観測できた使用が最低1回ある」ことを表すので、1回も無いなら
 * `unavailable` でなければならない。levels は union 上 `partial` と `complete`
 * で必須なため、`partial` へ絞り込んだ時点で必ず存在する。
 */
function requireObservedPartial(
  value:
    | { availability: "complete" | "unavailable"; levels?: SuperArtLevelCounts }
    | { availability: "partial"; levels: SuperArtLevelCounts },
  ctx: z.RefinementCtx,
): void {
  if (
    value.availability === "partial" &&
    value.levels.sa1 + value.levels.sa2 + value.levels.sa3 + value.levels.ca < 1
  ) {
    ctx.addIssue({
      code: "custom",
      path: ["levels"],
      message: "partial super art aggregates require an observed use",
    });
  }
}

const superArtStatsSchema = z.strictObject({
  own: ownSuperArtStatsSchema,
  opponent: opponentSuperArtStatsSchema,
});

const findingSchema = z.strictObject({
  kind: z.enum(FINDING_KINDS),
  // ruleset v3-v5の保存候補には存在しない。旧規則は全体transformで復元する。
  assessment: z.enum(FINDING_ASSESSMENTS).optional(),
  occurrences: z.number().int().min(1).max(MAX_COUNT),
  severityBp: z.number().int().min(0).max(MAX_SEVERITY_BP),
});

export const publishedAnalysisCandidateSchema = z
  .strictObject({
    rulesetVersion: z
      .number()
      .int()
      .refine(
        (value) =>
          (SUPPORTED_RULESET_VERSIONS as readonly number[]).includes(value),
        "unsupported ruleset version",
      ),
    ownCharacter: z.enum(CHARACTER_IDS),
    opponentCharacter: z.enum(CHARACTER_IDS),
    rounds: roundStatsSchema,
    findings: z.array(findingSchema).max(FINDING_KINDS.length),
    tactics: tacticStatsSchema,
    superArts: superArtStatsSchema.optional(),
  })
  .superRefine((value, ctx) => {
    const supportedRuleset = (
      SUPPORTED_RULESET_VERSIONS as readonly number[]
    ).includes(value.rulesetVersion);
    const { detected, won, lost, unresolved } = value.rounds;
    if (won + lost + unresolved !== detected) {
      ctx.addIssue({
        code: "custom",
        path: ["rounds"],
        message: "round totals do not match detected rounds",
      });
    }

    const seen = new Set<string>();
    value.findings.forEach((finding, index) => {
      if (seen.has(finding.kind)) {
        ctx.addIssue({
          code: "custom",
          path: ["findings", index, "kind"],
          message: "duplicate finding kind",
        });
      }
      seen.add(finding.kind);
      if (value.rulesetVersion >= 6 && finding.assessment === undefined) {
        ctx.addIssue({
          code: "custom",
          path: ["findings", index, "assessment"],
          message: "assessment is required for ruleset v6 and later",
        });
      }
    });

    if (
      supportedRuleset &&
      value.rulesetVersion >= 9 &&
      value.superArts === undefined
    ) {
      ctx.addIssue({
        code: "custom",
        path: ["superArts"],
        message: "super art aggregates are required for ruleset v9 and later",
      });
    }
    if (
      supportedRuleset &&
      value.rulesetVersion < 9 &&
      value.superArts !== undefined
    ) {
      ctx.addIssue({
        code: "custom",
        path: ["superArts"],
        message: "super art aggregates are not valid before ruleset v9",
      });
    }

    // Stryker disable all: Schema-valid payloads are already bounded below the 4 KiB target; this guard protects future schema growth.
    if (serializedByteLength(value) > MAX_PUBLISHED_ANALYSIS_BYTES) {
      ctx.addIssue({
        code: "custom",
        message: "published analysis exceeds the size limit",
      });
    }
    // Stryker restore all
  })
  .transform((value) => ({
    ...value,
    findings: value.findings.map((finding) => ({
      ...finding,
      assessment:
        finding.assessment ??
        legacyFindingAssessment(value.rulesetVersion, finding.kind),
    })),
  }));

export type PublishedAnalysisCandidate = z.infer<
  typeof publishedAnalysisCandidateSchema
>;

export function serializedByteLength(value: unknown): number {
  return new TextEncoder().encode(JSON.stringify(value)).byteLength;
}
