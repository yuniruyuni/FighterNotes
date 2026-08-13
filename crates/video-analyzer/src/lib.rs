//! SF6 動画解析クレート。
//!
//! Python `sf6_video.py` の主要ロジック（HP 検出・ラウンド検出・アドバイス生成）を移植。
//! OCR・クリップエクスポート・HTML 生成は含まない。

// 画素読み取りは独立した crate へ切り出してある。呼び出し側の経路を
// 変えないよう、モジュールごとここで再輸出する。
pub use advice_report::advice;
pub use analysis_context::{context, frame_data};
pub use attack_info_vision::attack_info;
pub use hud_vision::{frame_features, round_start};
pub use input_vision::{input_history, input_tracker};
pub use match_event_layer::match_events;
pub use spatial_refine::spatial;
pub use temporal_confirm::temporal;

pub mod pipeline;

pub use advice::{
    AdviceCard, AdviceKind, AdviceReport, AttributedDamageEvent, DamageBreakdown, DamageContext,
    DamageOrigin, DamageTakenEvent, EvidenceClip, InputStats, RoundSummary, TacticStats, Weakness,
};
pub use attack_info::{
    attack_info_debug_json, build_attack_sequences, read_attack_info,
    read_attack_info_from_meter_strip, AttackAttribute, AttackInfoFrameInspection,
    AttackInfoObservation, AttackInfoRoi, AttackInfoRois, AttackInfoSide, AttackInfoSideInspection,
    AttackInfoSideRois, AttackInfoTracker, AttackSequence, AttackSequenceStep,
};
pub use context::{AnalysisContext, PlayerContext};
pub use frame_data::{character_names, punish_options, StrikeKind};
pub use frame_features::{
    correct_hp_retroactive, drive_bar_debug_json, drive_fill_ratio,
    drive_fill_ratio_from_hud_strip, drive_gauge_read, drive_gauge_read_from_hud_strip,
    hp_bar_debug_json, hp_bar_score, hp_bar_score_from_hud_strip, hp_col_active, hp_col_orange,
    hp_col_pixel_detail_json, hp_col_yellow, hp_damage_fill, hp_damage_fill_from_hud_strip,
    hp_fill_ratio, hp_fill_ratio_from_hud_strip, hp_fill_ratio_with_quality,
    hp_fill_ratio_with_quality_from_hud_strip, hp_parallelogram, hp_score_decision_table,
    hp_score_roi_in_strip, super_gauge_debug_json, super_gauge_read,
    super_gauge_read_from_hud_strip, DriveGaugeRead, FrameFeatures, HpParallelogram,
    SuperGaugeRead, HP_ROI_P1, HP_ROI_P2, HUD_STRIP_H, HUD_STRIP_Y,
};
pub use input_history::{
    input_history_debug_json, read_input_row0_from_strip, read_input_rows, BadgeColor, BadgeMark,
    InputDir, InputRow, INPUT_STRIP_H, INPUT_STRIP_Y,
};
pub use input_tracker::{repair_row0_sequence, TrackedInput};
pub use match_events::{
    build_match_events, build_match_events_with_context,
    build_match_events_with_context_and_attack_info,
    build_match_events_with_context_and_fight_markers,
    build_match_events_with_context_and_fight_markers_and_attack_info, AttackDamageConsistency,
    AttackEvidence, BurnoutCause, BurnoutPeriod, CompoundThreat, ContactEvent,
    DamageAttackEvidence, DamageEvent, DefenseResponse, DefenseResponseKind, DefensiveActionKind,
    DpReachability, DriveImpactEvent, DriveImpactOutcome, DriveRushEvent, DriveRushOutcome,
    EventConfidence, GuardBreakEvent, JumpDirection, JumpEvent, JumpOutcome, MatchEvents,
    MeterState, MinusPressEvent, MinusPressOutcome, MinusSituationEvent, ProjectileThreat,
    PunishChance, PunishOrigin, PunishOutcome, PunishReachability, ReversalEvent, RoundInfo,
    SuperArtAttackEvidence, SuperArtContext, SuperArtEvent, SuperArtOutcome, TeleportContext,
    TeleportEvent, ThreatOutcome, ThrowActionEvent, ThrowApproach, ThrowEvent, ThrowOutcome,
};
pub use pipeline::{
    analyze_features, analyze_features_with_context, analyze_match, analyze_match_with_context,
    finalize_features, finalize_features_with_fight_markers,
};
pub use round_start::{
    detect_fight_markers, fight_score_from_hud_strip, FightMarker, FightObservation,
    FIGHT_PATCH_HEIGHT, FIGHT_PATCH_WIDTH, FIGHT_PATCH_X, FIGHT_PATCH_Y, FIGHT_SAMPLE_INTERVAL,
};
pub use spatial::{
    refine_match_events_with_spatial, spatial_candidate_windows, ActorHint, ActorObservation,
    DistanceBand, HorizontalMotion, HorizontalOrder, MotionRegionObservation, ProjectileCandidate,
    SpatialCandidateWindow, SpatialConfig, SpatialError, SpatialExtractor, SpatialHintRange,
    SpatialHints, SpatialObservation, SpatialPoint, SpatialRect,
};
pub use temporal::{
    clean_drive_temporal, clean_super_temporal, confirm_hp, confirm_hp_with_fight_markers,
};
