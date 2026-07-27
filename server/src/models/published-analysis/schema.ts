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
  })
  .superRefine((value, ctx) => {
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
