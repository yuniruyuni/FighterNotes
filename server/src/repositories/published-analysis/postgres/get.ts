import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import { compToSQL } from "../../../infra/db/sql-helpers";
import type { PublishedAnalysis } from "../../../models/published-analysis";
import {
  type AnalysisRow,
  type FindingRow,
  hydratePublishedAnalysis,
  publishedAnalysisSpecToSQL,
} from "./common";

export async function get(
  db: Database,
  spec: PublishedAnalysis.Spec,
): Promise<PublishedAnalysis | null> {
  const where = compToSQL(spec, publishedAnalysisSpecToSQL);
  const row = await db.queryGet<AnalysisRow>(sql`
    SELECT
      a.id, a.schema_version, a.ruleset_version,
      a.presentation_revision, a.own_character, a.opponent_character,
      a.rounds_detected, a.rounds_won, a.rounds_lost,
      a.rounds_unresolved,
      a.created_at, a.expires_at,
      t.anti_air_opportunities, t.anti_air_successes, t.jump_ins_allowed,
      t.di_faced, t.di_returned, t.di_blocked, t.di_parried, t.di_hit,
      t.di_avoided, t.di_unconfirmed,
      t.raw_drive_rushes_faced, t.raw_drive_rushes_defended,
      t.raw_drive_rushes_hit, t.raw_drive_rushes_unconfirmed,
      t.dash_throws_faced, t.throw_whiffs,
      t.minus_defense_opportunities,
      t.fastest_strike_challenges, t.fastest_strike_losses,
      t.fastest_throw_challenges, t.fastest_throw_losses,
      t.burnout_count, t.burnout_duration_deciseconds,
      t.burnout_hp_lost_bp, t.burnout_hp_dealt_bp,
      t.burnout_self_initiated, t.burnout_forced, t.burnout_mixed,
      t.burnout_unknown,
      s.analysis_id AS super_art_analysis_id,
      s.own_available, s.opponent_available,
      s.own_sa1, s.own_sa2, s.own_sa3, s.own_ca,
      s.own_hit, s.own_block, s.own_no_immediate_contact,
      s.own_punished, s.own_ko,
      s.own_combo, s.own_punish, s.own_reversal, s.own_neutral,
      s.opponent_sa1, s.opponent_sa2, s.opponent_sa3, s.opponent_ca,
      s.opponent_hit, s.opponent_block,
      s.opponent_no_immediate_contact,
      s.opponent_punished, s.opponent_ko
    FROM published_analyses a
    INNER JOIN published_analysis_tactics t ON t.analysis_id = a.id
    LEFT JOIN published_analysis_super_arts s ON s.analysis_id = a.id
    WHERE ${where}
    LIMIT 1
  `);
  if (!row) return null;

  const findings = await db.queryAll<FindingRow>(sql`
    SELECT kind, assessment, occurrences, severity_bp
    FROM published_analysis_findings
    WHERE analysis_id = ${row.id}
    ORDER BY ordinal
  `);
  return hydratePublishedAnalysis(row, findings);
}
