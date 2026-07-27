import type {
  DeletePasswordHash,
  FindingKind,
  PublishedAnalysisCandidate,
  ShareId,
} from "../../models/published-analysis";
import {
  createPersistablePublishedAnalysis,
  createPublishedAnalysisContent,
  FINDING_KINDS,
  legacyFindingAssessment,
  MAX_COUNT,
  MAX_DURATION_DECISECONDS,
  MAX_HP_BP,
  MAX_SEVERITY_BP,
} from "../../models/published-analysis";

export const DELETE_PASSWORD = "fighter-notes-delete-key";
export const DELETE_PASSWORD_HASH =
  "$argon2id$v=19$m=19456,t=2,p=1$afqgXENr3y/WCxW5FclnyO6NDY/hIjW2oVS12hgu3b8$Tn12OEC62ylqoD4wLt+6ou9Hq7medNra44FzjO9DlRM" as DeletePasswordHash;
export const FIXTURE_ID = "Abcdefghijklmnopqrstu_" as ShareId;

export function candidate(maximum = false): PublishedAnalysisCandidate {
  const count = maximum ? MAX_COUNT : 2;
  const kinds: readonly FindingKind[] = maximum
    ? FINDING_KINDS
    : ["anti_air", "big_hits"];
  return {
    rulesetVersion: 3,
    ownCharacter: "LUKE",
    opponentCharacter: "CHUN_LI",
    rounds: maximum
      ? { detected: 255, won: 85, lost: 85, unresolved: 85 }
      : { detected: 2, won: 1, lost: 1, unresolved: 0 },
    findings: kinds.map((kind) => ({
      kind,
      assessment: legacyFindingAssessment(3, kind),
      occurrences: count,
      severityBp: maximum ? MAX_SEVERITY_BP : 1200,
    })),
    tactics: {
      antiAir: {
        opportunities: count,
        successes: count,
        jumpInsAllowed: count,
      },
      driveImpact: {
        faced: count,
        returned: count,
        blocked: count,
        parried: count,
        hit: count,
        avoided: count,
        unconfirmed: count,
      },
      rawDriveRush: {
        faced: count,
        defended: count,
        hit: count,
        unconfirmed: count,
      },
      dashThrow: { faced: count },
      throwWhiff: { count },
      fastestChallenge: {
        opportunities: count,
        strikeAttempts: count,
        strikeLosses: count,
        throwAttempts: count,
        throwLosses: count,
      },
      burnout: {
        count,
        durationDeciseconds: maximum ? MAX_DURATION_DECISECONDS : 123,
        hpLostBp: maximum ? MAX_HP_BP : 1200,
        hpDealtBp: maximum ? MAX_HP_BP : 800,
        selfInitiated: count,
        forced: count,
        mixed: count,
        unknown: count,
      },
    },
  };
}

export function persistableAnalysis(options?: {
  id?: ShareId;
  now?: Date;
  retentionDays?: number;
  maximum?: boolean;
}) {
  const content = createPublishedAnalysisContent(candidate(options?.maximum));
  if (!content.ok) throw new Error("published analysis fixture is invalid");
  return createPersistablePublishedAnalysis({
    id: options?.id ?? FIXTURE_ID,
    content: content.value,
    deletePasswordHash: DELETE_PASSWORD_HASH,
    now: options?.now ?? new Date("2026-07-13T00:00:00.000Z"),
    retentionDays: options?.retentionDays ?? 365,
  }).analysis;
}

export function createInput(analysis = candidate()) {
  return { analysis, deletePassword: DELETE_PASSWORD };
}
