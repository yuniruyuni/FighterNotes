CREATE TABLE IF NOT EXISTS published_analyses (
  id TEXT PRIMARY KEY
    CHECK (id ~ '^[A-Za-z0-9_-]{22}$'),
  schema_version SMALLINT NOT NULL
    CHECK (schema_version = 1),
  ruleset_version INTEGER NOT NULL
    CHECK (ruleset_version IN (3, 4, 5, 6, 7, 8)),
  presentation_revision SMALLINT NOT NULL
    CHECK (presentation_revision IN (1)),
  own_character TEXT NOT NULL
    CHECK (own_character IN (
      'A_K_I', 'AKUMA', 'ALEX', 'BLANKA', 'C_VIPER', 'CAMMY',
      'CHUN_LI', 'DEE_JAY', 'DHALSIM', 'E_HONDA', 'ED', 'ELENA',
      'GUILE', 'INGRID', 'JAMIE', 'JP', 'JURI', 'KEN', 'KIMBERLY',
      'LILY', 'LUKE', 'M_BISON', 'MAI', 'MANON', 'MARISA', 'RASHID',
      'RYU', 'SAGAT', 'TERRY', 'ZANGIEF'
    )),
  opponent_character TEXT NOT NULL
    CHECK (opponent_character IN (
      'A_K_I', 'AKUMA', 'ALEX', 'BLANKA', 'C_VIPER', 'CAMMY',
      'CHUN_LI', 'DEE_JAY', 'DHALSIM', 'E_HONDA', 'ED', 'ELENA',
      'GUILE', 'INGRID', 'JAMIE', 'JP', 'JURI', 'KEN', 'KIMBERLY',
      'LILY', 'LUKE', 'M_BISON', 'MAI', 'MANON', 'MARISA', 'RASHID',
      'RYU', 'SAGAT', 'TERRY', 'ZANGIEF'
    )),
  rounds_detected SMALLINT NOT NULL
    CHECK (rounds_detected BETWEEN 0 AND 255),
  rounds_won SMALLINT NOT NULL
    CHECK (rounds_won BETWEEN 0 AND 255),
  rounds_lost SMALLINT NOT NULL
    CHECK (rounds_lost BETWEEN 0 AND 255),
  rounds_unresolved SMALLINT NOT NULL
    CHECK (rounds_unresolved BETWEEN 0 AND 255),
  -- Legacy rollout column. Keep it nullable until all revisions that know
  -- about it are outside the rollback window.
  delete_token_hash BYTEA
    CHECK (octet_length(delete_token_hash) = 32),
  -- New shares store only an Argon2id PHC string. Nullable keeps shares
  -- created before password-based deletion compatible until their expiry.
  delete_password_hash TEXT
    CHECK (char_length(delete_password_hash) BETWEEN 64 AND 512),
  created_at TIMESTAMPTZ NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  CHECK (rounds_won + rounds_lost + rounds_unresolved = rounds_detected),
  CHECK (expires_at > created_at)
);
-- 既存環境のCREATE TABLE制約も現在のrulesetへ更新する。
ALTER TABLE published_analyses
  DROP CONSTRAINT IF EXISTS published_analyses_ruleset_version_check;
ALTER TABLE published_analyses
  ADD CONSTRAINT published_analyses_ruleset_version_check
  CHECK (ruleset_version IN (3, 4, 5, 6, 7, 8));
CREATE INDEX IF NOT EXISTS published_analyses_expires_at_idx
  ON published_analyses (expires_at);
GRANT SELECT, INSERT, DELETE ON published_analyses TO fighter_app;

CREATE TABLE IF NOT EXISTS published_analysis_findings (
  analysis_id TEXT NOT NULL
    REFERENCES published_analyses (id) ON DELETE CASCADE,
  ordinal SMALLINT NOT NULL
    CHECK (ordinal BETWEEN 0 AND 20),
  kind TEXT NOT NULL
    CHECK (kind IN (
      'layered_defense', 'teleport_defense', 'anti_air', 'own_jumps',
      'burnout', 'committed_button_vs_di', 'mashing',
      'press_while_minus', 'throw_while_minus',
      'guard_break', 'reversal_punished', 'low_scaling_super',
      'punish_fail', 'punish_missed',
      'low_conversion', 'throw_interrupted_by_invincible',
      'throw_whiff_punished', 'throw_loop',
      'early_hits', 'lead_loss', 'big_hits'
    )),
  -- ruleset v3-v5の既存行はNULL。読み出し時に当時の規則から復元する。
  -- ruleset v6以降はapplication schemaが必須化する。
  assessment TEXT
    CHECK (assessment IN ('diagnosis', 'observation', 'statistic')),
  occurrences INTEGER NOT NULL
    CHECK (occurrences BETWEEN 1 AND 65535),
  severity_bp INTEGER NOT NULL
    CHECK (severity_bp BETWEEN 0 AND 1000000),
  PRIMARY KEY (analysis_id, kind),
  UNIQUE (analysis_id, ordinal)
);
-- CREATE TABLE IF NOT EXISTSだけでは既存環境へ列が追加されないため、
-- 同じschema fileを再適用できる形でオンライン追加する。
ALTER TABLE published_analysis_findings
  ADD COLUMN IF NOT EXISTS assessment TEXT
  CHECK (assessment IN ('diagnosis', 'observation', 'statistic'));
-- 新しいruleset v6カードを既存環境でも保存できるよう、閉じたIDとordinalを更新する。
ALTER TABLE published_analysis_findings
  DROP CONSTRAINT IF EXISTS published_analysis_findings_kind_check;
ALTER TABLE published_analysis_findings
  ADD CONSTRAINT published_analysis_findings_kind_check
  CHECK (kind IN (
    'layered_defense', 'teleport_defense', 'anti_air', 'own_jumps',
    'burnout', 'committed_button_vs_di', 'mashing',
    'press_while_minus', 'throw_while_minus',
    'guard_break', 'reversal_punished', 'low_scaling_super',
    'punish_fail', 'punish_missed',
    'low_conversion', 'throw_interrupted_by_invincible',
    'throw_whiff_punished', 'throw_loop',
    'early_hits', 'lead_loss', 'big_hits'
  ));
ALTER TABLE published_analysis_findings
  DROP CONSTRAINT IF EXISTS published_analysis_findings_ordinal_check;
ALTER TABLE published_analysis_findings
  ADD CONSTRAINT published_analysis_findings_ordinal_check
  CHECK (ordinal BETWEEN 0 AND 20);
GRANT SELECT, INSERT ON published_analysis_findings TO fighter_app;

CREATE TABLE IF NOT EXISTS published_analysis_tactics (
  analysis_id TEXT PRIMARY KEY
    REFERENCES published_analyses (id) ON DELETE CASCADE,
  anti_air_opportunities INTEGER NOT NULL CHECK (anti_air_opportunities BETWEEN 0 AND 65535),
  anti_air_successes INTEGER NOT NULL CHECK (anti_air_successes BETWEEN 0 AND 65535),
  jump_ins_allowed INTEGER NOT NULL CHECK (jump_ins_allowed BETWEEN 0 AND 65535),
  di_faced INTEGER NOT NULL CHECK (di_faced BETWEEN 0 AND 65535),
  di_returned INTEGER NOT NULL CHECK (di_returned BETWEEN 0 AND 65535),
  di_blocked INTEGER NOT NULL CHECK (di_blocked BETWEEN 0 AND 65535),
  di_parried INTEGER NOT NULL CHECK (di_parried BETWEEN 0 AND 65535),
  di_hit INTEGER NOT NULL CHECK (di_hit BETWEEN 0 AND 65535),
  di_avoided INTEGER NOT NULL CHECK (di_avoided BETWEEN 0 AND 65535),
  di_unconfirmed INTEGER NOT NULL CHECK (di_unconfirmed BETWEEN 0 AND 65535),
  raw_drive_rushes_faced INTEGER NOT NULL CHECK (raw_drive_rushes_faced BETWEEN 0 AND 65535),
  raw_drive_rushes_defended INTEGER NOT NULL CHECK (raw_drive_rushes_defended BETWEEN 0 AND 65535),
  raw_drive_rushes_hit INTEGER NOT NULL CHECK (raw_drive_rushes_hit BETWEEN 0 AND 65535),
  raw_drive_rushes_unconfirmed INTEGER NOT NULL CHECK (raw_drive_rushes_unconfirmed BETWEEN 0 AND 65535),
  dash_throws_faced INTEGER NOT NULL CHECK (dash_throws_faced BETWEEN 0 AND 65535),
  throw_whiffs INTEGER NOT NULL CHECK (throw_whiffs BETWEEN 0 AND 65535),
  -- Existing ruleset v3-v5 rows did not store this denominator. Backfill them
  -- as zero while all new ruleset v6 inserts provide the measured value.
  minus_defense_opportunities INTEGER NOT NULL DEFAULT 0 CHECK (minus_defense_opportunities BETWEEN 0 AND 65535),
  fastest_strike_challenges INTEGER NOT NULL CHECK (fastest_strike_challenges BETWEEN 0 AND 65535),
  fastest_strike_losses INTEGER NOT NULL CHECK (fastest_strike_losses BETWEEN 0 AND 65535),
  fastest_throw_challenges INTEGER NOT NULL CHECK (fastest_throw_challenges BETWEEN 0 AND 65535),
  fastest_throw_losses INTEGER NOT NULL CHECK (fastest_throw_losses BETWEEN 0 AND 65535),
  burnout_count INTEGER NOT NULL CHECK (burnout_count BETWEEN 0 AND 65535),
  burnout_duration_deciseconds INTEGER NOT NULL CHECK (burnout_duration_deciseconds BETWEEN 0 AND 864000),
  burnout_hp_lost_bp INTEGER NOT NULL CHECK (burnout_hp_lost_bp BETWEEN 0 AND 1000000),
  burnout_hp_dealt_bp INTEGER NOT NULL CHECK (burnout_hp_dealt_bp BETWEEN 0 AND 1000000),
  burnout_self_initiated INTEGER NOT NULL CHECK (burnout_self_initiated BETWEEN 0 AND 65535),
  burnout_forced INTEGER NOT NULL CHECK (burnout_forced BETWEEN 0 AND 65535),
  burnout_mixed INTEGER NOT NULL CHECK (burnout_mixed BETWEEN 0 AND 65535),
  burnout_unknown INTEGER NOT NULL CHECK (burnout_unknown BETWEEN 0 AND 65535)
);
GRANT SELECT, INSERT ON published_analysis_tactics TO fighter_app;

-- Successful creation events are independent from result-row cleanup. This
-- keeps the UTC daily quota monotonic throughout each day.
CREATE TABLE IF NOT EXISTS published_analysis_create_events (
  analysis_id TEXT PRIMARY KEY
    CHECK (analysis_id ~ '^[A-Za-z0-9_-]{22}$'),
  created_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS published_analysis_create_events_created_at_idx
  ON published_analysis_create_events (created_at);
-- The scheduled cleanup job intentionally reuses the runtime DB role. DELETE
-- is needed to prune old quota events; ownership and DDL remain migration-only.
GRANT SELECT, INSERT, DELETE ON published_analysis_create_events TO fighter_app;

-- One row per client/bucket keeps fixed-window counters durable across Cloud
-- Run instances and revision changes without retaining a plaintext IP address.
CREATE TABLE IF NOT EXISTS published_analysis_rate_limits (
  bucket TEXT NOT NULL
    CHECK (bucket IN ('create', 'delete', 'public_read')),
  client_key_hash TEXT NOT NULL
    CHECK (client_key_hash ~ '^[0-9a-f]{64}$'),
  window_started_at TIMESTAMPTZ NOT NULL,
  request_count INTEGER NOT NULL
    CHECK (request_count BETWEEN 1 AND 100001),
  PRIMARY KEY (bucket, client_key_hash)
);
CREATE INDEX IF NOT EXISTS published_analysis_rate_limits_window_idx
  ON published_analysis_rate_limits (window_started_at);
-- UPDATE is limited to the shared counters. Ownership, DELETE and DDL stay
-- migration-only until bounded pruning is introduced.
GRANT SELECT, INSERT, UPDATE
  ON published_analysis_rate_limits TO fighter_app;
