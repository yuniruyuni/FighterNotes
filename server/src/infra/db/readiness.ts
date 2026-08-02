import type { QueryResultRow } from "pg";
import { SCHEMA_VERSION } from "../../models/published-analysis";
import type { ILogger } from "../logger/types";
import type { Database } from "./database";
import { sql } from "./sql";

const DEFAULT_READINESS_TIMEOUT_MS = 1_500;
const DEFAULT_STATEMENT_TIMEOUT_MS = 750;

interface CompatibilityRow extends QueryResultRow {
  compatible: boolean;
}

export interface DatabaseReadiness {
  check(): Promise<boolean>;
}

interface DatabaseReadinessOptions {
  timeoutMs?: number;
  statementTimeoutMs?: number;
}

export function createDatabaseReadiness(
  db: Database,
  logger: ILogger,
  options: DatabaseReadinessOptions = {},
): DatabaseReadiness {
  const timeoutMs = options.timeoutMs ?? DEFAULT_READINESS_TIMEOUT_MS;
  const statementTimeoutMs =
    options.statementTimeoutMs ?? DEFAULT_STATEMENT_TIMEOUT_MS;
  let inFlight: Promise<boolean> | undefined;

  function startProbe(): Promise<boolean> {
    const probe = db
      .readTransaction((tx) => inspectDatabaseReadiness(tx, statementTimeoutMs))
      .then((compatible) => {
        if (!compatible) {
          logger.warn("Database readiness check failed", {
            reason: "incompatible",
          });
        }
        return compatible;
      })
      .catch(() => {
        logger.warn("Database readiness check failed", {
          reason: "unavailable",
        });
        return false;
      });
    const tracked = probe.finally(() => {
      if (inFlight === tracked) inFlight = undefined;
    });
    inFlight = tracked;
    return tracked;
  }

  return {
    async check() {
      const probe = inFlight ?? startProbe();
      let timer: ReturnType<typeof setTimeout> | undefined;
      const timeout = new Promise<boolean>((resolve) => {
        timer = setTimeout(() => {
          logger.warn("Database readiness check failed", {
            reason: "timeout",
          });
          resolve(false);
        }, timeoutMs);
      });
      try {
        return await Promise.race([probe, timeout]);
      } finally {
        if (timer !== undefined) clearTimeout(timer);
      }
    },
  };
}

export async function inspectDatabaseReadiness(
  db: Database,
  statementTimeoutMs = DEFAULT_STATEMENT_TIMEOUT_MS,
): Promise<boolean> {
  await db.queryGet(sql`
    SELECT set_config(
      'statement_timeout', ${`${statementTimeoutMs}ms`}, true
    ) AS statement_timeout
  `);
  const expectedSchemaConstraint = `CHECK (schema_version = ${SCHEMA_VERSION})`;
  const row = await db.queryGet<CompatibilityRow>(sql`
    WITH required_columns(table_name, column_name) AS (
      VALUES
        ('published_analyses', 'id'),
        ('published_analyses', 'schema_version'),
        ('published_analyses', 'ruleset_version'),
        ('published_analyses', 'presentation_revision'),
        ('published_analyses', 'own_character'),
        ('published_analyses', 'opponent_character'),
        ('published_analyses', 'rounds_detected'),
        ('published_analyses', 'rounds_won'),
        ('published_analyses', 'rounds_lost'),
        ('published_analyses', 'rounds_unresolved'),
        ('published_analyses', 'delete_password_hash'),
        ('published_analyses', 'created_at'),
        ('published_analyses', 'expires_at'),
        ('published_analysis_findings', 'analysis_id'),
        ('published_analysis_findings', 'ordinal'),
        ('published_analysis_findings', 'kind'),
        ('published_analysis_findings', 'assessment'),
        ('published_analysis_findings', 'occurrences'),
        ('published_analysis_findings', 'severity_bp'),
        ('published_analysis_tactics', 'analysis_id'),
        ('published_analysis_tactics', 'anti_air_opportunities'),
        ('published_analysis_tactics', 'anti_air_successes'),
        ('published_analysis_tactics', 'jump_ins_allowed'),
        ('published_analysis_tactics', 'di_faced'),
        ('published_analysis_tactics', 'di_returned'),
        ('published_analysis_tactics', 'di_blocked'),
        ('published_analysis_tactics', 'di_parried'),
        ('published_analysis_tactics', 'di_hit'),
        ('published_analysis_tactics', 'di_avoided'),
        ('published_analysis_tactics', 'di_unconfirmed'),
        ('published_analysis_tactics', 'raw_drive_rushes_faced'),
        ('published_analysis_tactics', 'raw_drive_rushes_defended'),
        ('published_analysis_tactics', 'raw_drive_rushes_hit'),
        ('published_analysis_tactics', 'raw_drive_rushes_unconfirmed'),
        ('published_analysis_tactics', 'dash_throws_faced'),
        ('published_analysis_tactics', 'throw_whiffs'),
        ('published_analysis_tactics', 'minus_defense_opportunities'),
        ('published_analysis_tactics', 'fastest_strike_challenges'),
        ('published_analysis_tactics', 'fastest_strike_losses'),
        ('published_analysis_tactics', 'fastest_throw_challenges'),
        ('published_analysis_tactics', 'fastest_throw_losses'),
        ('published_analysis_tactics', 'burnout_count'),
        ('published_analysis_tactics', 'burnout_duration_deciseconds'),
        ('published_analysis_tactics', 'burnout_hp_lost_bp'),
        ('published_analysis_tactics', 'burnout_hp_dealt_bp'),
        ('published_analysis_tactics', 'burnout_self_initiated'),
        ('published_analysis_tactics', 'burnout_forced'),
        ('published_analysis_tactics', 'burnout_mixed'),
        ('published_analysis_tactics', 'burnout_unknown'),
        ('published_analysis_create_events', 'analysis_id'),
        ('published_analysis_create_events', 'created_at')
    ), present_columns AS (
      SELECT table_name, column_name
      FROM information_schema.columns
      WHERE table_schema = 'public'
    )
    SELECT (
      current_setting('transaction_read_only') = 'on'
      AND has_schema_privilege(current_user, 'public', 'USAGE')
      AND NOT EXISTS (
        SELECT table_name, column_name FROM required_columns
        EXCEPT
        SELECT table_name, column_name FROM present_columns
      )
      AND EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'public.published_analyses'::regclass
          AND conname = 'published_analyses_schema_version_check'
          AND pg_get_constraintdef(oid, true) = ${expectedSchemaConstraint}
      )
      AND has_table_privilege(current_user, 'public.published_analyses', 'SELECT')
      AND has_table_privilege(current_user, 'public.published_analyses', 'INSERT')
      AND has_table_privilege(current_user, 'public.published_analyses', 'DELETE')
      AND has_table_privilege(
        current_user, 'public.published_analysis_findings', 'SELECT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_findings', 'INSERT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_tactics', 'SELECT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_tactics', 'INSERT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_create_events', 'SELECT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_create_events', 'INSERT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_create_events', 'DELETE'
      )
    ) AS compatible
  `);
  return row?.compatible === true;
}
