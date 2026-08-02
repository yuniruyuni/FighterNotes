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
      own_sa.analysis_id AS own_super_art_analysis_id,
      opponent_sa.analysis_id AS opponent_super_art_analysis_id,
      own_sa.complete AS own_super_art_complete,
      opponent_sa.complete AS opponent_super_art_complete,
      own_sa.sa1 AS own_sa1, own_sa.sa2 AS own_sa2,
      own_sa.sa3 AS own_sa3, own_sa.ca AS own_ca,
      own_sa.hit AS own_hit, own_sa.block AS own_block,
      own_sa.no_immediate_contact AS own_no_immediate_contact,
      own_sa.punished AS own_punished, own_sa.ko AS own_ko,
      own_sa.combo AS own_combo, own_sa.punish AS own_punish,
      own_sa.reversal AS own_reversal, own_sa.neutral AS own_neutral,
      opponent_sa.sa1 AS opponent_sa1,
      opponent_sa.sa2 AS opponent_sa2,
      opponent_sa.sa3 AS opponent_sa3,
      opponent_sa.ca AS opponent_ca,
      opponent_sa.hit AS opponent_hit,
      opponent_sa.block AS opponent_block,
      opponent_sa.no_immediate_contact AS opponent_no_immediate_contact,
      opponent_sa.punished AS opponent_punished,
      opponent_sa.ko AS opponent_ko
    FROM published_analyses a
    INNER JOIN published_analysis_tactics t ON t.analysis_id = a.id
    LEFT JOIN published_analysis_super_arts s ON s.analysis_id = a.id
    LEFT JOIN published_analysis_own_super_arts own_sa
      ON own_sa.analysis_id = s.analysis_id
    LEFT JOIN published_analysis_opponent_super_arts opponent_sa
      ON opponent_sa.analysis_id = s.analysis_id
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
