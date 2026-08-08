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

export type DamageApproach = "raw_drive_rush";

export type DamageContact = "throw" | "strike" | "drive_impact" | "projectile";

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
  /** 帰属した中央表示の攻撃連係数。実際のヒット数ではない。 */
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
  approach?: DamageApproach;
  contact?: DamageContact;
  contact_confidence?: "low" | "medium" | "high";
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
  /** 被ダメージが直接の結果である指摘だけが持つ、失った体力の合計。 */
  hp_lost?: number | null;
  description: string;
  practice: string;
  evidence: EvidenceClip[];
}

export type EvidenceRequirement =
  | "own_hp"
  | "opponent_hp"
  | "own_drive"
  | "opponent_drive"
  | "own_super"
  | "opponent_super"
  | "own_input"
  | "opponent_input"
  | "frame_meter"
  | "contacts"
  | "punishes"
  | "spatial"
  | "own_attack_info"
  | "opponent_attack_info";

export interface SuppressedAdviceCard {
  id: string;
  title: string;
  missing_requirements: EvidenceRequirement[];
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
  /** 確定ラウンド内の試合フレーム数。以下のHUD/入力系coverageの共通分母。 */
  detector_match_frames?: number;
  own_hp_reliable_frames?: number;
  opponent_hp_reliable_frames?: number;
  own_drive_reliable_frames?: number;
  opponent_drive_reliable_frames?: number;
  own_super_reliable_frames?: number;
  opponent_super_reliable_frames?: number;
  own_super_end_reliable?: boolean;
  opponent_super_end_reliable?: boolean;
  own_input_observed_frames?: number;
  opponent_input_observed_frames?: number;
  own_input_repaired_frames?: number;
  opponent_input_repaired_frames?: number;
  own_meter_mapped_frames?: number;
  opponent_meter_mapped_frames?: number;
  /** 空間解析は全試合ではなく候補区間だけを分母にする。 */
  spatial_candidate_frames?: number;
  spatial_sampled_frames?: number;
  spatial_usable_frames?: number;
  own_spatial_observed_frames?: number;
  opponent_spatial_observed_frames?: number;
  attack_damage_events?: number;
  attack_damage_linked?: number;
  attack_damage_consistent?: number;
  attack_damage_mismatched?: number;
  attack_damage_unverified?: number;
  own_attack_damage_events?: number;
  own_attack_damage_usable?: number;
  opponent_attack_damage_events?: number;
  opponent_attack_damage_usable?: number;
  /** ruleset v9以降は解析器が依存関係と閾値を解決して付与する。 */
  availability?: AnalysisAvailability;
}

export type EvidenceAvailability =
  | "available"
  | "unavailable"
  | "not_applicable";

export interface AnalysisAvailability {
  own_hp: EvidenceAvailability;
  opponent_hp: EvidenceAvailability;
  own_drive: EvidenceAvailability;
  opponent_drive: EvidenceAvailability;
  own_super: EvidenceAvailability;
  opponent_super: EvidenceAvailability;
  own_input: EvidenceAvailability;
  opponent_input: EvidenceAvailability;
  own_meter: EvidenceAvailability;
  opponent_meter: EvidenceAvailability;
  contacts: EvidenceAvailability;
  punishes: EvidenceAvailability;
  spatial: EvidenceAvailability;
  own_attack_info: EvidenceAvailability;
  opponent_attack_info: EvidenceAvailability;
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
  disadvantage_decisions: number;
  disadvantage_top_option_percent: number;
  advantage_decisions: number;
  advantage_top_option_percent: number;
  okizeme_decisions: number;
  okizeme_top_option_percent: number;
  throws_faced: number;
  throws_taken: number;
  throws_teched: number;
  throws_reversal_escaped: number;
  knockdowns_scored: number;
  okizeme_meaty: number;
  okizeme_pressured: number;
  okizeme_neutral: number;
  knockdowns_taken: number;
  okizeme_faced_meaty: number;
  own_di_used: number;
  own_di_hit: number;
  own_di_blocked: number;
  own_di_parried: number;
  own_di_countered: number;
  own_di_whiffed: number;
  own_di_unconfirmed: number;
  own_raw_drive_rushes: number;
  own_raw_drive_rush_hits: number;
  own_raw_drive_rush_defended: number;
  drive_spent_on_impacts: number;
  drive_spent_on_rushes: number;
  drive_damage_from_impacts: number;
  drive_damage_from_rushes: number;
  drive_spend_samples: number;
  whiffs: number;
  whiffs_punished: number;
  opponent_whiffs: number;
  opponent_whiffs_punished: number;
  advantage_opportunities: number;
  advantage_continued: number;
  advantage_abandoned: number;
  advantage_turns_lost: number;
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
  /** ruleset v8以前では省略。trueは全件集計、falseは検出件数に応じpartialまたはunavailable。 */
  super_art_stats_complete?: boolean;
  /** ruleset v8以前では省略。trueは全件集計、falseは検出件数に応じpartialまたはunavailable。 */
  opponent_super_art_stats_complete?: boolean;
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
  /** ruleset v8以前の保存済みレポートでは省略。 */
  suppressed_cards?: SuppressedAdviceCard[];
  round_summaries: RoundSummary[];
  input_stats: InputStats | null;
  tactic_stats: TacticStats;
  coverage?: AnalysisCoverage;
  analysis_warnings?: string[];
}
