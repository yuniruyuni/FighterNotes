// 解析レポート（wasm finish() の JSON）の型定義。
// Rust 側 advice.rs の AdviceReport 系と対応する。

export interface DamageTakenEvent {
  frame: number;
  hp_drop: number;
  own_hp_before: number;
  own_hp_after: number;
  meter_state: string | null;
}

export type DamageOrigin =
  | "compound_threat"
  | "teleport"
  | "throw"
  | "drive_impact"
  | "raw_drive_rush"
  | "own_jump_caught"
  | "opponent_jump_in"
  | "projectile"
  | "strike"
  | "unclassified";

export type StrikeKind = "high" | "overhead" | "low" | "air";

export type DamageContext =
  | "mashing"
  | "press_while_minus"
  | "guard_break"
  | "reversal_punished"
  | "punish_whiff"
  | "burnout";

export type AttackDamageConsistency = "consistent" | "mismatch" | "unverified";

export interface DamageAttackEvidence {
  victim: number;
  attacker: number;
  damage_start_frame: number;
  sequence_start_frame: number;
  sequence_end_frame: number;
  combo_damage: number;
  sequence_count: number;
  final_scaling_percent: number;
  starter_attribute?: "upper" | "middle" | "lower" | "throw";
  final_attribute: "upper" | "middle" | "lower" | "throw";
  complete: boolean;
  recovered_from_max: boolean;
  confidence: "low" | "medium" | "high";
  hp_consistency: AttackDamageConsistency;
}

export interface AttributedDamageEvent {
  sequence_no: number;
  round_no: number;
  start_frame: number;
  end_frame: number;
  scene_frame: number;
  hp_before: number;
  hp_after: number;
  hp_drop: number;
  origin: DamageOrigin;
  confidence: "low" | "medium" | "high";
  strike_kind?: StrikeKind;
  strike_kind_confidence?: "low" | "medium" | "high";
  contexts: DamageContext[];
  attack_evidence?: DamageAttackEvidence;
}

export interface DamageBreakdown {
  attribution_version: number;
  total_hp_lost: number;
  classified_hp_lost: number;
  events: AttributedDamageEvent[];
}

export interface EvidenceClip {
  frame: number;
  label: string;
  /** 区間クリップの終端フレーム（lead_loss 等。省略時は frame の ±固定窓） */
  end_frame?: number;
}

export interface AdviceCard {
  id: string;
  /** 原因診断・事実確認・統計。ruleset v3 の保存済みレポートでは省略。 */
  kind?: "diagnosis" | "observation" | "statistic";
  /** カードの因果帰属に使った証拠の確度。 */
  confidence?: "low" | "medium" | "high";
  title: string;
  severity: number;
  description: string;
  practice: string;
  evidence: EvidenceClip[];
}

export interface RoundSummary {
  round_no: number;
  start_frame: number;
  end_frame: number;
  won: boolean | null;
  own_hp_end: number;
  opp_hp_end: number;
  own_hp_lost: number;
  opp_hp_lost: number;
  own_hits_taken: number;
  early_hit: boolean;
  own_burnouts: number;
  detection_confidence?: "high" | "medium";
}

export interface AnalysisCoverage {
  match_frames: number;
  analyzed_match_frames: number;
  input_segments: number;
  analyzed_input_segments: number;
  attack_damage_events?: number;
  attack_damage_linked?: number;
  attack_damage_consistent?: number;
  attack_damage_mismatched?: number;
}

export interface InputStats {
  total_inputs: number;
  minutes: number;
  jumps: number;
  jumps_per_min: number;
  jump_got_hit: number;
  jump_landed: number;
  throw_attempts: number;
  throw_hits: number;
  button_presses: number;
  auto_presses: number;
  auto_ratio: number;
  di_presses: number;
  crouch_ratio: number;
}

export interface TacticStats {
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
  burnout_seconds: number;
  burnout_hp_lost: number;
  burnout_hp_dealt: number;
  burnout_self_initiated: number;
  burnout_forced: number;
  burnout_mixed: number;
  burnout_unknown: number;
  /** ruleset v6 以前の保存済みレポートでは省略。 */
  sa1_used?: number;
  sa2_used?: number;
  sa3_used?: number;
  ca_used?: number;
  super_hits?: number;
  super_blocked?: number;
  super_no_immediate_contact?: number;
  super_punished?: number;
  super_kos?: number;
  super_combo_uses?: number;
  super_punish_uses?: number;
  super_reversal_uses?: number;
  super_neutral_uses?: number;
  super_damage_samples?: number;
  super_reported_combo_damage?: number;
  super_reported_marginal_damage?: number;
  super_low_scaling_uses?: number;
  opponent_sa1_used?: number;
  opponent_sa2_used?: number;
  opponent_sa3_used?: number;
  opponent_ca_used?: number;
  opponent_super_hits?: number;
  opponent_super_blocked?: number;
  opponent_super_no_immediate_contact?: number;
  opponent_super_punished?: number;
  opponent_super_kos?: number;
  super_gauge_end?: number;
  opponent_super_gauge_end?: number;
}

export interface AdviceReport {
  ruleset_version: number;
  analyzer_build_id?: string;
  total_frames: number;
  rounds_detected: number;
  damage_taken_events: DamageTakenEvent[];
  /** ruleset v6 以前に保存された解析結果では省略。 */
  damage_breakdown?: DamageBreakdown;
  weaknesses: Array<{
    category: string;
    description: string;
    frequency: number;
  }>;
  practice_items: string[];
  summary: string;
  cards: AdviceCard[];
  round_summaries: RoundSummary[];
  input_stats: InputStats | null;
  tactic_stats: TacticStats;
  coverage?: AnalysisCoverage;
  analysis_warnings?: string[];
}
