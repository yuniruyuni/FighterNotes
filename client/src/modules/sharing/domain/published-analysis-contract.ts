import type { CharacterId } from "~/modules/analysis/contracts.js";

export const SHAREABLE_FINDING_KINDS = [
  "layered_defense",
  "teleport_defense",
  "anti_air",
  "own_jumps",
  "burnout",
  "mashing",
  "press_while_minus",
  "throw_while_minus",
  "guard_break",
  "reversal_punished",
  "punish_fail",
  "punish_missed",
  "low_conversion",
  "throw_loop",
  "early_hits",
  "lead_loss",
  "big_hits",
] as const;

export const SHAREABLE_ASSESSMENTS = [
  "diagnosis",
  "observation",
  "statistic",
] as const;

export type ShareableFindingKind = (typeof SHAREABLE_FINDING_KINDS)[number];
export type ShareableAssessment = (typeof SHAREABLE_ASSESSMENTS)[number];

export interface PublishedFindingCandidate {
  kind: ShareableFindingKind;
  assessment: ShareableAssessment;
  occurrences: number;
  severityBp: number;
}

export interface PublishedTacticStats {
  antiAir: {
    opportunities: number;
    successes: number;
    jumpInsAllowed: number;
  };
  driveImpact: {
    faced: number;
    returned: number;
    blocked: number;
    parried: number;
    hit: number;
    avoided: number;
    unconfirmed: number;
  };
  rawDriveRush: {
    faced: number;
    defended: number;
    hit: number;
    unconfirmed: number;
  };
  dashThrow: { faced: number };
  throwWhiff: { count: number };
  fastestChallenge: {
    opportunities: number;
    strikeAttempts: number;
    strikeLosses: number;
    throwAttempts: number;
    throwLosses: number;
  };
  burnout: {
    count: number;
    durationDeciseconds: number;
    hpLostBp: number;
    hpDealtBp: number;
    selfInitiated: number;
    forced: number;
    mixed: number;
    unknown: number;
  };
}

export interface PublishedAnalysisCandidate {
  rulesetVersion: number;
  ownCharacter: CharacterId;
  opponentCharacter: CharacterId;
  rounds: {
    detected: number;
    won: number;
    lost: number;
    unresolved: number;
  };
  findings: PublishedFindingCandidate[];
  tactics: PublishedTacticStats;
}
