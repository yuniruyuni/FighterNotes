export const CHARACTER_IDS = [
  "A_K_I",
  "AKUMA",
  "ALEX",
  "BLANKA",
  "C_VIPER",
  "CAMMY",
  "CHUN_LI",
  "DEE_JAY",
  "DHALSIM",
  "E_HONDA",
  "ED",
  "ELENA",
  "GUILE",
  "INGRID",
  "JAMIE",
  "JP",
  "JURI",
  "KEN",
  "KIMBERLY",
  "LILY",
  "LUKE",
  "M_BISON",
  "MAI",
  "MANON",
  "MARISA",
  "RASHID",
  "RYU",
  "SAGAT",
  "TERRY",
  "YASMINE",
  "ZANGIEF",
] as const;

export type CharacterId = (typeof CHARACTER_IDS)[number];

export const FINDING_KINDS = [
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
  "whiff_punished",
  "throw_loop",
  "early_hits",
  "lead_loss",
  "big_hits",
] as const;

export type FindingKind = (typeof FINDING_KINDS)[number];

export const FINDING_ASSESSMENTS = [
  "diagnosis",
  "observation",
  "statistic",
] as const;

export type FindingAssessment = (typeof FINDING_ASSESSMENTS)[number];

export const SCHEMA_VERSION = 1 as const;
export const PRESENTATION_REVISION = 1 as const;
export const SUPPORTED_RULESET_VERSIONS = [
  3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
] as const;

/** ruleset v5以前は共有payloadに判定区分が無いため、当時の表示規則を復元する。 */
export function legacyFindingAssessment(
  rulesetVersion: number,
  kind: FindingKind,
): FindingAssessment {
  if (kind === "burnout") return "statistic";
  if (kind === "big_hits") return "observation";
  if (rulesetVersion >= 5 && (kind === "early_hits" || kind === "lead_loss")) {
    return "observation";
  }
  return "diagnosis";
}

export const MAX_PUBLISHED_ANALYSIS_BYTES = 8 * 1024;
export const TARGET_PUBLISHED_ANALYSIS_BYTES = 4 * 1024;
export const MAX_COUNT = 65_535;
export const MAX_ROUNDS = 255;
export const MAX_SEVERITY_BP = 1_000_000;
export const MAX_DURATION_DECISECONDS = 864_000;
export const MAX_HP_BP = 1_000_000;
