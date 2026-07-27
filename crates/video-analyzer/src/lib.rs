//! SF6 動画解析クレート。
//!
//! Python `sf6_video.py` の主要ロジック（HP 検出・ラウンド検出・アドバイス生成）を移植。
//! OCR・クリップエクスポート・HTML 生成は含まない。

pub mod advice;
pub mod context;
pub mod frame_data;
pub mod frame_features;
pub mod input_history;
pub mod input_tracker;
pub mod match_events;
pub mod pipeline;
pub mod spatial;
pub mod temporal;

pub use advice::{
    AdviceCard, AdviceKind, AdviceReport, AttributedDamageEvent, DamageBreakdown, DamageContext,
    DamageOrigin, DamageTakenEvent, EvidenceClip, InputStats, RoundSummary, TacticStats, Weakness,
};
pub use context::{AnalysisContext, PlayerContext};
pub use frame_data::{character_names, punish_options, StrikeKind};
pub use frame_features::{
    correct_hp_retroactive, drive_bar_debug_json, drive_fill_ratio,
    drive_fill_ratio_from_hud_strip, drive_gauge_read, drive_gauge_read_from_hud_strip,
    hp_bar_debug_json, hp_bar_score, hp_bar_score_from_hud_strip, hp_col_active, hp_col_orange,
    hp_col_pixel_detail_json, hp_col_yellow, hp_damage_fill, hp_damage_fill_from_hud_strip,
    hp_fill_ratio, hp_fill_ratio_from_hud_strip, hp_fill_ratio_with_quality,
    hp_fill_ratio_with_quality_from_hud_strip, hp_parallelogram, DriveGaugeRead, FrameFeatures,
    HpParallelogram, HP_ROI_P1, HP_ROI_P2, HUD_STRIP_H, HUD_STRIP_Y,
};
pub use input_history::{
    input_history_debug_json, read_input_row0_from_strip, read_input_rows, BadgeColor, BadgeMark,
    InputDir, InputRow, INPUT_STRIP_H, INPUT_STRIP_Y,
};
pub use input_tracker::{repair_row0_sequence, TrackedInput};
pub use match_events::{
    build_match_events, build_match_events_with_context, BurnoutCause, BurnoutPeriod,
    CompoundThreat, ContactEvent, DamageEvent, DefenseResponse, DefenseResponseKind,
    DefensiveActionKind, DpReachability, DriveImpactEvent, DriveImpactOutcome, DriveRushEvent,
    DriveRushOutcome, EventConfidence, GuardBreakEvent, JumpDirection, JumpEvent, JumpOutcome,
    MatchEvents, MeterState, MinusPressEvent, MinusPressOutcome, MinusSituationEvent,
    ProjectileThreat, PunishChance, PunishOrigin, PunishOutcome, PunishReachability, ReversalEvent,
    RoundInfo, TeleportContext, TeleportEvent, ThreatOutcome, ThrowActionEvent, ThrowApproach,
    ThrowEvent, ThrowOutcome,
};
pub use pipeline::{
    analyze_features, analyze_features_with_context, analyze_match, analyze_match_with_context,
    finalize_features,
};
pub use spatial::{
    refine_match_events_with_spatial, spatial_candidate_windows, ActorHint, ActorObservation,
    DistanceBand, HorizontalMotion, HorizontalOrder, MotionRegionObservation, ProjectileCandidate,
    SpatialCandidateWindow, SpatialConfig, SpatialError, SpatialExtractor, SpatialHintRange,
    SpatialHints, SpatialObservation, SpatialPoint, SpatialRect,
};
pub use temporal::{clean_drive_temporal, confirm_hp};
