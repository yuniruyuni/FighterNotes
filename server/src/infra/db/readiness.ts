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
    WITH required_columns(
      table_name, column_name, data_type, not_null
    ) AS (
      VALUES
        ('published_analyses', 'id', 'text', true),
        ('published_analyses', 'schema_version', 'smallint', true),
        ('published_analyses', 'ruleset_version', 'integer', true),
        ('published_analyses', 'presentation_revision', 'smallint', true),
        ('published_analyses', 'own_character', 'text', true),
        ('published_analyses', 'opponent_character', 'text', true),
        ('published_analyses', 'rounds_detected', 'smallint', true),
        ('published_analyses', 'rounds_won', 'smallint', true),
        ('published_analyses', 'rounds_lost', 'smallint', true),
        ('published_analyses', 'rounds_unresolved', 'smallint', true),
        ('published_analyses', 'delete_password_hash', 'text', false),
        ('published_analyses', 'logical_size_bytes', 'integer', true),
        ('published_analyses', 'created_at', 'timestamp with time zone', true),
        ('published_analyses', 'expires_at', 'timestamp with time zone', true),
        ('published_analysis_findings', 'analysis_id', 'text', true),
        ('published_analysis_findings', 'ordinal', 'smallint', true),
        ('published_analysis_findings', 'kind', 'text', true),
        ('published_analysis_findings', 'assessment', 'text', false),
        ('published_analysis_findings', 'occurrences', 'integer', true),
        ('published_analysis_findings', 'severity_bp', 'integer', true),
        ('published_analysis_tactics', 'analysis_id', 'text', true),
        ('published_analysis_tactics', 'anti_air_opportunities', 'integer', true),
        ('published_analysis_tactics', 'anti_air_successes', 'integer', true),
        ('published_analysis_tactics', 'jump_ins_allowed', 'integer', true),
        ('published_analysis_tactics', 'di_faced', 'integer', true),
        ('published_analysis_tactics', 'di_returned', 'integer', true),
        ('published_analysis_tactics', 'di_blocked', 'integer', true),
        ('published_analysis_tactics', 'di_parried', 'integer', true),
        ('published_analysis_tactics', 'di_hit', 'integer', true),
        ('published_analysis_tactics', 'di_avoided', 'integer', true),
        ('published_analysis_tactics', 'di_unconfirmed', 'integer', true),
        ('published_analysis_tactics', 'raw_drive_rushes_faced', 'integer', true),
        ('published_analysis_tactics', 'raw_drive_rushes_defended', 'integer', true),
        ('published_analysis_tactics', 'raw_drive_rushes_hit', 'integer', true),
        ('published_analysis_tactics', 'raw_drive_rushes_unconfirmed', 'integer', true),
        ('published_analysis_tactics', 'dash_throws_faced', 'integer', true),
        ('published_analysis_tactics', 'throw_whiffs', 'integer', true),
        ('published_analysis_tactics', 'minus_defense_opportunities', 'integer', true),
        ('published_analysis_tactics', 'fastest_strike_challenges', 'integer', true),
        ('published_analysis_tactics', 'fastest_strike_losses', 'integer', true),
        ('published_analysis_tactics', 'fastest_throw_challenges', 'integer', true),
        ('published_analysis_tactics', 'fastest_throw_losses', 'integer', true),
        ('published_analysis_tactics', 'burnout_count', 'integer', true),
        ('published_analysis_tactics', 'burnout_duration_deciseconds', 'integer', true),
        ('published_analysis_tactics', 'burnout_hp_lost_bp', 'integer', true),
        ('published_analysis_tactics', 'burnout_hp_dealt_bp', 'integer', true),
        ('published_analysis_tactics', 'burnout_self_initiated', 'integer', true),
        ('published_analysis_tactics', 'burnout_forced', 'integer', true),
        ('published_analysis_tactics', 'burnout_mixed', 'integer', true),
        ('published_analysis_tactics', 'burnout_unknown', 'integer', true),
        ('published_analysis_super_arts', 'analysis_id', 'text', true),
        ('published_analysis_own_super_arts', 'analysis_id', 'text', true),
        ('published_analysis_own_super_arts', 'sa1', 'integer', true),
        ('published_analysis_own_super_arts', 'sa2', 'integer', true),
        ('published_analysis_own_super_arts', 'sa3', 'integer', true),
        ('published_analysis_own_super_arts', 'ca', 'integer', true),
        ('published_analysis_own_super_arts', 'hit', 'integer', true),
        ('published_analysis_own_super_arts', 'block', 'integer', true),
        ('published_analysis_own_super_arts', 'no_immediate_contact', 'integer', true),
        ('published_analysis_own_super_arts', 'punished', 'integer', true),
        ('published_analysis_own_super_arts', 'ko', 'integer', true),
        ('published_analysis_own_super_arts', 'combo', 'integer', true),
        ('published_analysis_own_super_arts', 'punish', 'integer', true),
        ('published_analysis_own_super_arts', 'reversal', 'integer', true),
        ('published_analysis_own_super_arts', 'neutral', 'integer', true),
        ('published_analysis_opponent_super_arts', 'analysis_id', 'text', true),
        ('published_analysis_opponent_super_arts', 'sa1', 'integer', true),
        ('published_analysis_opponent_super_arts', 'sa2', 'integer', true),
        ('published_analysis_opponent_super_arts', 'sa3', 'integer', true),
        ('published_analysis_opponent_super_arts', 'ca', 'integer', true),
        ('published_analysis_opponent_super_arts', 'hit', 'integer', true),
        ('published_analysis_opponent_super_arts', 'block', 'integer', true),
        ('published_analysis_opponent_super_arts', 'no_immediate_contact', 'integer', true),
        ('published_analysis_opponent_super_arts', 'punished', 'integer', true),
        ('published_analysis_opponent_super_arts', 'ko', 'integer', true),
        ('published_analysis_create_events', 'analysis_id', 'text', true),
        ('published_analysis_create_events', 'created_at', 'timestamp with time zone', true),
        ('published_analysis_rate_limits', 'bucket', 'text', true),
        ('published_analysis_rate_limits', 'client_key_hash', 'text', true),
        ('published_analysis_rate_limits', 'window_started_at', 'timestamp with time zone', true),
        ('published_analysis_rate_limits', 'request_count', 'integer', true)
    ), present_columns AS (
      SELECT
        relation.relname::text AS table_name,
        attribute.attname::text AS column_name,
        format_type(attribute.atttypid, attribute.atttypmod) AS data_type,
        attribute.attnotnull AS not_null
      FROM pg_class AS relation
      INNER JOIN pg_namespace AS namespace
        ON namespace.oid = relation.relnamespace
      INNER JOIN pg_attribute AS attribute
        ON attribute.attrelid = relation.oid
      WHERE namespace.nspname = 'public'
        AND relation.relkind IN ('r', 'p')
        AND attribute.attnum > 0
        AND NOT attribute.attisdropped
    ), required_defaults(table_name, column_name, expression) AS (
      VALUES
        ('published_analyses', 'logical_size_bytes', '8192'),
        ('published_analysis_tactics', 'minus_defense_opportunities', '0')
    ), present_defaults AS (
      SELECT
        relation.relname::text AS table_name,
        attribute.attname::text AS column_name,
        pg_get_expr(default_value.adbin, default_value.adrelid) AS expression
      FROM pg_attrdef AS default_value
      INNER JOIN pg_class AS relation ON relation.oid = default_value.adrelid
      INNER JOIN pg_namespace AS namespace
        ON namespace.oid = relation.relnamespace
      INNER JOIN pg_attribute AS attribute
        ON attribute.attrelid = default_value.adrelid
        AND attribute.attnum = default_value.adnum
      WHERE namespace.nspname = 'public'
    ), required_constraints(
      table_name, constraint_name, constraint_type, definition
    ) AS (
      VALUES
        ('published_analyses', 'published_analyses_pkey', 'p', 'PRIMARY KEY (id)'),
        ('published_analyses', 'published_analyses_schema_version_check', 'c', ${expectedSchemaConstraint}),
        ('published_analyses', 'published_analyses_ruleset_version_check', 'c', 'CHECK (ruleset_version = ANY (ARRAY[3, 4, 5, 6, 7, 8, 9]))'),
        ('published_analyses', 'published_analyses_presentation_revision_check', 'c', 'CHECK (presentation_revision = 1)'),
        ('published_analyses', 'published_analyses_logical_size_bytes_check', 'c', 'CHECK (logical_size_bytes >= 1 AND logical_size_bytes <= 8192)'),
        ('published_analyses', 'published_analyses_check1', 'c', 'CHECK (expires_at > created_at)'),
        ('published_analysis_findings', 'published_analysis_findings_pkey', 'p', 'PRIMARY KEY (analysis_id, kind)'),
        ('published_analysis_findings', 'published_analysis_findings_analysis_id_ordinal_key', 'u', 'UNIQUE (analysis_id, ordinal)'),
        ('published_analysis_findings', 'published_analysis_findings_analysis_id_fkey', 'f', 'FOREIGN KEY (analysis_id) REFERENCES published_analyses(id) ON DELETE CASCADE'),
        ('published_analysis_tactics', 'published_analysis_tactics_pkey', 'p', 'PRIMARY KEY (analysis_id)'),
        ('published_analysis_tactics', 'published_analysis_tactics_analysis_id_fkey', 'f', 'FOREIGN KEY (analysis_id) REFERENCES published_analyses(id) ON DELETE CASCADE'),
        ('published_analysis_super_arts', 'published_analysis_super_arts_pkey', 'p', 'PRIMARY KEY (analysis_id)'),
        ('published_analysis_super_arts', 'published_analysis_super_arts_analysis_id_fkey', 'f', 'FOREIGN KEY (analysis_id) REFERENCES published_analyses(id) ON DELETE CASCADE'),
        ('published_analysis_own_super_arts', 'published_analysis_own_super_arts_pkey', 'p', 'PRIMARY KEY (analysis_id)'),
        ('published_analysis_own_super_arts', 'published_analysis_own_super_arts_analysis_id_fkey', 'f', 'FOREIGN KEY (analysis_id) REFERENCES published_analysis_super_arts(analysis_id) ON DELETE CASCADE'),
        ('published_analysis_opponent_super_arts', 'published_analysis_opponent_super_arts_pkey', 'p', 'PRIMARY KEY (analysis_id)'),
        ('published_analysis_opponent_super_arts', 'published_analysis_opponent_super_arts_analysis_id_fkey', 'f', 'FOREIGN KEY (analysis_id) REFERENCES published_analysis_super_arts(analysis_id) ON DELETE CASCADE'),
        ('published_analysis_create_events', 'published_analysis_create_events_pkey', 'p', 'PRIMARY KEY (analysis_id)'),
        ('published_analysis_rate_limits', 'published_analysis_rate_limits_pkey', 'p', 'PRIMARY KEY (bucket, client_key_hash)'),
        ('published_analysis_rate_limits', 'published_analysis_rate_limits_bucket_check', 'c', 'CHECK (bucket = ANY (ARRAY[''create''::text, ''delete''::text, ''public_read''::text]))'),
        ('published_analysis_rate_limits', 'published_analysis_rate_limits_client_key_hash_check', 'c', 'CHECK (client_key_hash ~ ''^[0-9a-f]{64}$''::text)'),
        ('published_analysis_rate_limits', 'published_analysis_rate_limits_request_count_check', 'c', 'CHECK (request_count >= 1 AND request_count <= 100001)')
    ), present_constraints AS (
      SELECT
        relation.relname::text AS table_name,
        constraint_.conname::text AS constraint_name,
        constraint_.contype::text AS constraint_type,
        pg_get_constraintdef(constraint_.oid, true) AS definition
      FROM pg_constraint AS constraint_
      INNER JOIN pg_class AS relation ON relation.oid = constraint_.conrelid
      INNER JOIN pg_namespace AS namespace
        ON namespace.oid = relation.relnamespace
      WHERE namespace.nspname = 'public'
        AND constraint_.convalidated
        AND (
          constraint_.conindid = 0
          OR EXISTS (
            SELECT 1
            FROM pg_index AS backing_index
            WHERE backing_index.indexrelid = constraint_.conindid
              AND backing_index.indisvalid
              AND backing_index.indisready
          )
        )
    ), required_indexes(table_name, index_name, columns) AS (
      VALUES
        ('published_analyses', 'published_analyses_cleanup_idx', ARRAY['expires_at', 'created_at', 'id']::text[]),
        ('published_analyses', 'published_analyses_created_at_cleanup_idx', ARRAY['created_at', 'expires_at', 'id']::text[]),
        ('published_analysis_rate_limits', 'published_analysis_rate_limits_cleanup_idx', ARRAY['window_started_at', 'bucket', 'client_key_hash']::text[])
    ), present_indexes AS (
      SELECT
        relation.relname::text AS table_name,
        index_relation.relname::text AS index_name,
        ARRAY(
          SELECT attribute.attname::text
          FROM unnest(index_.indkey::smallint[]) WITH ORDINALITY
            AS key_column(attnum, position)
          INNER JOIN pg_attribute AS attribute
            ON attribute.attrelid = index_.indrelid
            AND attribute.attnum = key_column.attnum
          WHERE key_column.position <= index_.indnkeyatts
          ORDER BY key_column.position
        ) AS columns
      FROM pg_index AS index_
      INNER JOIN pg_class AS relation ON relation.oid = index_.indrelid
      INNER JOIN pg_namespace AS namespace
        ON namespace.oid = relation.relnamespace
      INNER JOIN pg_class AS index_relation
        ON index_relation.oid = index_.indexrelid
      INNER JOIN pg_am AS access_method
        ON access_method.oid = index_relation.relam
      WHERE namespace.nspname = 'public'
        AND access_method.amname = 'btree'
        AND index_.indisvalid
        AND index_.indisready
        AND index_.indpred IS NULL
        AND index_.indexprs IS NULL
    )
    SELECT (
      current_setting('transaction_read_only') = 'on'
      AND has_schema_privilege(current_user, 'public', 'USAGE')
      AND NOT EXISTS (
        SELECT * FROM required_columns
        EXCEPT
        SELECT * FROM present_columns
      )
      AND NOT EXISTS (
        SELECT * FROM required_defaults
        EXCEPT
        SELECT * FROM present_defaults
      )
      AND NOT EXISTS (
        SELECT * FROM required_constraints
        EXCEPT
        SELECT * FROM present_constraints
      )
      AND NOT EXISTS (
        SELECT * FROM required_indexes
        EXCEPT
        SELECT * FROM present_indexes
      )
      AND has_table_privilege(current_user, 'public.published_analyses', 'SELECT')
      AND has_table_privilege(current_user, 'public.published_analyses', 'INSERT')
      AND has_table_privilege(current_user, 'public.published_analyses', 'DELETE')
      AND has_column_privilege(
        current_user, 'public.published_analyses', 'schema_version', 'UPDATE'
      )
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
        current_user, 'public.published_analysis_super_arts', 'SELECT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_super_arts', 'INSERT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_own_super_arts', 'SELECT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_own_super_arts', 'INSERT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_opponent_super_arts', 'SELECT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_opponent_super_arts', 'INSERT'
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
      AND has_table_privilege(
        current_user, 'public.published_analysis_rate_limits', 'SELECT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_rate_limits', 'INSERT'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_rate_limits', 'UPDATE'
      )
      AND has_table_privilege(
        current_user, 'public.published_analysis_rate_limits', 'DELETE'
      )
    ) AS compatible
  `);
  return row?.compatible === true;
}
