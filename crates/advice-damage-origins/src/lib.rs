//! 被弾をその原因へ帰属させる。
//!
//! モジュール名は移設前と同じにしてある。`advice-report` 側が
//! `crate::…` として再輸出するため、呼び出し側の経路は変わらない。

pub use advice_model::{decisions, model, parameters};
pub use analysis_context::{context, frame_data};
pub use attack_info_vision::attack_info;
pub use hud_vision::frame_features;
pub use match_event_layer::match_events;
pub use temporal_confirm::temporal;

// 移設前は advice の平坦な名前空間から見えていた項目。経路を保つ。
pub use advice_model::model::*;
pub use advice_model::parameters::*;
pub use hud_vision::frame_features::FrameFeatures;
pub use match_event_layer::match_events::{
    AdvantageOutcome, DefensiveActionKind, EventConfidence, JumpDirection, JumpOutcome,
    MatchEvents, MinusPressOutcome, OkizemeOutcome, WhiffOutcome,
};

pub mod damage_origins;
