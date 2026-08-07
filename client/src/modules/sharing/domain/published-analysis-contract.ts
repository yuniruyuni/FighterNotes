import type { CharacterId } from "~/modules/analysis/contracts.js";

export const SHAREABLE_FINDING_KINDS = [
  "layered_defense",
  "teleport_defense",
  "anti_air",
  "own_jumps",
  "burnout",
  "committed_button_vs_di",
  "mashing",
  "press_while_minus",
  "throw_while_minus",
  "advantage_abandoned",
  "guard_break",
  "reversal_punished",
  "low_scaling_super",
  "punish_fail",
  "punish_missed",
  "low_conversion",
  "throw_interrupted_by_invincible",
  "throw_whiff_punished",
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

export interface PublishedSuperArtLevels {
  sa1: number;
  sa2: number;
  sa3: number;
  ca: number;
}

export interface PublishedSuperArtOutcomes {
  hit: number;
  block: number;
  noImmediateContact: number;
  punished: number;
  ko: number;
}

export type PublishedOwnSuperArtStats =
  | { availability: "unavailable" }
  | {
      availability: "complete" | "partial";
      levels: PublishedSuperArtLevels;
      outcomes: PublishedSuperArtOutcomes;
      contexts: {
        combo: number;
        punish: number;
        reversal: number;
        neutral: number;
      };
    };

export type PublishedOpponentSuperArtStats =
  | { availability: "unavailable" }
  | {
      availability: "complete" | "partial";
      levels: PublishedSuperArtLevels;
      outcomes: PublishedSuperArtOutcomes;
    };

export interface PublishedSuperArtStats {
  own: PublishedOwnSuperArtStats;
  opponent: PublishedOpponentSuperArtStats;
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
  /** ruleset v9 以降だけが持つ、公開可能な SA/CA 集計。 */
  superArts?: PublishedSuperArtStats;
}
