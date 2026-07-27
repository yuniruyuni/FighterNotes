import type { QueryResultRow } from "pg";
import { type SQLFragment, sql } from "../../../infra/db/sql";
import { assertNever } from "../../../infra/db/sql-helpers";
import type {
  CharacterId,
  FindingAssessment,
  FindingKind,
  PublishedAnalysis,
  PublishedAnalysisContent,
  ShareId,
} from "../../../models/published-analysis";
import {
  createPublishedAnalysisContent,
  PRESENTATION_REVISION,
  SCHEMA_VERSION,
} from "../../../models/published-analysis";

export interface AnalysisRow extends QueryResultRow {
  id: string;
  schema_version: number;
  ruleset_version: number;
  presentation_revision: number;
  own_character: string;
  opponent_character: string;
  rounds_detected: number;
  rounds_won: number;
  rounds_lost: number;
  rounds_unresolved: number;
  created_at: Date | string;
  expires_at: Date | string;
  anti_air_opportunities: number;
  anti_air_successes: number;
  jump_ins_allowed: number;
  di_faced: number;
  di_returned: number;
  di_blocked: number;
  di_parried: number;
  di_hit: number;
  di_avoided: number;
  di_unconfirmed: number;
  raw_drive_rushes_faced: number;
  raw_drive_rushes_defended: number;
  raw_drive_rushes_hit: number;
  raw_drive_rushes_unconfirmed: number;
  dash_throws_faced: number;
  throw_whiffs: number;
  minus_defense_opportunities: number;
  fastest_strike_challenges: number;
  fastest_strike_losses: number;
  fastest_throw_challenges: number;
  fastest_throw_losses: number;
  burnout_count: number;
  burnout_duration_deciseconds: number;
  burnout_hp_lost_bp: number;
  burnout_hp_dealt_bp: number;
  burnout_self_initiated: number;
  burnout_forced: number;
  burnout_mixed: number;
  burnout_unknown: number;
}

export interface FindingRow extends QueryResultRow {
  kind: string;
  assessment: string | null;
  occurrences: number;
  severity_bp: number;
}

export function publishedAnalysisSpecToSQL(
  spec: PublishedAnalysis.SpecData,
): SQLFragment {
  switch (spec.type) {
    case "ById":
      return sql`a.id = ${spec.id}`;
    case "ActiveAt":
      return sql`a.expires_at > ${spec.at}`;
    // Stryker disable next-line ConditionalExpression: SpecData is a closed discriminated union constructed by the domain model.
    default:
      return assertNever(spec);
  }
}

export function hydratePublishedAnalysis(
  row: AnalysisRow,
  findingRows: FindingRow[],
): PublishedAnalysis | null {
  if (
    row.schema_version !== SCHEMA_VERSION ||
    row.presentation_revision !== PRESENTATION_REVISION
  ) {
    return null;
  }

  const candidate = {
    rulesetVersion: row.ruleset_version,
    ownCharacter: row.own_character as CharacterId,
    opponentCharacter: row.opponent_character as CharacterId,
    rounds: {
      detected: row.rounds_detected,
      won: row.rounds_won,
      lost: row.rounds_lost,
      unresolved: row.rounds_unresolved,
    },
    findings: findingRows.map((finding) => ({
      kind: finding.kind as FindingKind,
      ...(finding.assessment === null
        ? {}
        : { assessment: finding.assessment as FindingAssessment }),
      occurrences: finding.occurrences,
      severityBp: finding.severity_bp,
    })),
    tactics: {
      antiAir: {
        opportunities: row.anti_air_opportunities,
        successes: row.anti_air_successes,
        jumpInsAllowed: row.jump_ins_allowed,
      },
      driveImpact: {
        faced: row.di_faced,
        returned: row.di_returned,
        blocked: row.di_blocked,
        parried: row.di_parried,
        hit: row.di_hit,
        avoided: row.di_avoided,
        unconfirmed: row.di_unconfirmed,
      },
      rawDriveRush: {
        faced: row.raw_drive_rushes_faced,
        defended: row.raw_drive_rushes_defended,
        hit: row.raw_drive_rushes_hit,
        unconfirmed: row.raw_drive_rushes_unconfirmed,
      },
      dashThrow: { faced: row.dash_throws_faced },
      throwWhiff: { count: row.throw_whiffs },
      fastestChallenge: {
        opportunities: row.minus_defense_opportunities,
        strikeAttempts: row.fastest_strike_challenges,
        strikeLosses: row.fastest_strike_losses,
        throwAttempts: row.fastest_throw_challenges,
        throwLosses: row.fastest_throw_losses,
      },
      burnout: {
        count: row.burnout_count,
        durationDeciseconds: row.burnout_duration_deciseconds,
        hpLostBp: row.burnout_hp_lost_bp,
        hpDealtBp: row.burnout_hp_dealt_bp,
        selfInitiated: row.burnout_self_initiated,
        forced: row.burnout_forced,
        mixed: row.burnout_mixed,
        unknown: row.burnout_unknown,
      },
    },
  };
  const content = createPublishedAnalysisContent(candidate);
  if (!content.ok) return null;
  return {
    id: row.id as ShareId,
    content: content.value as PublishedAnalysisContent,
    createdAt: toDate(row.created_at),
    expiresAt: toDate(row.expires_at),
  };
}

function toDate(value: Date | string): Date {
  return value instanceof Date ? value : new Date(value);
}
