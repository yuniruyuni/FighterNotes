import type { Database } from "../../../infra/db/database";
import { sql } from "../../../infra/db/sql";
import type { PersistablePublishedAnalysis } from "../../../models/published-analysis";

export async function create(
  db: Database,
  analysis: PersistablePublishedAnalysis,
): Promise<void> {
  const { content } = analysis;
  await db.queryRun(sql`
    INSERT INTO published_analyses (
      id, schema_version, ruleset_version, presentation_revision,
      own_character, opponent_character,
      rounds_detected, rounds_won, rounds_lost, rounds_unresolved,
      delete_password_hash, logical_size_bytes, created_at, expires_at
    ) VALUES (
      ${analysis.id}, ${content.schemaVersion}, ${content.rulesetVersion},
      ${content.presentationRevision}, ${content.ownCharacter},
      ${content.opponentCharacter}, ${content.rounds.detected},
      ${content.rounds.won}, ${content.rounds.lost},
      ${content.rounds.unresolved}, ${analysis.deletePasswordHash},
      ${analysis.logicalSizeBytes}, ${analysis.createdAt}, ${analysis.expiresAt}
    )
  `);

  if (content.findings.length > 0) {
    const values = content.findings.map(
      (finding, ordinal) => sql`(
        ${analysis.id}, ${ordinal}, ${finding.kind}, ${finding.assessment},
        ${finding.occurrences}, ${finding.severityBp}
      )`,
    );
    await db.queryRun(sql`
      INSERT INTO published_analysis_findings (
        analysis_id, ordinal, kind, assessment, occurrences, severity_bp
      ) VALUES ${sql.join(values, ", ")}
    `);
  }

  const tactics = content.tactics;
  await db.queryRun(sql`
    INSERT INTO published_analysis_tactics (
      analysis_id,
      anti_air_opportunities, anti_air_successes, jump_ins_allowed,
      di_faced, di_returned, di_blocked, di_parried, di_hit, di_avoided,
      di_unconfirmed,
      raw_drive_rushes_faced, raw_drive_rushes_defended,
      raw_drive_rushes_hit, raw_drive_rushes_unconfirmed,
      dash_throws_faced, throw_whiffs,
      minus_defense_opportunities,
      fastest_strike_challenges, fastest_strike_losses,
      fastest_throw_challenges, fastest_throw_losses,
      burnout_count, burnout_duration_deciseconds,
      burnout_hp_lost_bp, burnout_hp_dealt_bp,
      burnout_self_initiated, burnout_forced, burnout_mixed, burnout_unknown
    ) VALUES (
      ${analysis.id},
      ${tactics.antiAir.opportunities}, ${tactics.antiAir.successes},
      ${tactics.antiAir.jumpInsAllowed},
      ${tactics.driveImpact.faced}, ${tactics.driveImpact.returned},
      ${tactics.driveImpact.blocked}, ${tactics.driveImpact.parried},
      ${tactics.driveImpact.hit}, ${tactics.driveImpact.avoided},
      ${tactics.driveImpact.unconfirmed},
      ${tactics.rawDriveRush.faced}, ${tactics.rawDriveRush.defended},
      ${tactics.rawDriveRush.hit}, ${tactics.rawDriveRush.unconfirmed},
      ${tactics.dashThrow.faced}, ${tactics.throwWhiff.count},
      ${tactics.fastestChallenge.opportunities},
      ${tactics.fastestChallenge.strikeAttempts},
      ${tactics.fastestChallenge.strikeLosses},
      ${tactics.fastestChallenge.throwAttempts},
      ${tactics.fastestChallenge.throwLosses},
      ${tactics.burnout.count}, ${tactics.burnout.durationDeciseconds},
      ${tactics.burnout.hpLostBp}, ${tactics.burnout.hpDealtBp},
      ${tactics.burnout.selfInitiated}, ${tactics.burnout.forced},
      ${tactics.burnout.mixed}, ${tactics.burnout.unknown}
    )
  `);
}
