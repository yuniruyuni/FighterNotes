//! 試合イベントの形と、それを組み立てるための共通の見方。
//!
//! どの抽出器も、同じイベント型・同じ閾値・同じ時系列の読み方の上に載る。
//! 上流の観測にも個々の抽出にも依存しない土台なので、独立させてある。
//!
//!   - `model`      : イベントとその集合の形
//!   - `parameters` : 抽出の閾値
//!   - `timeline`   : フレーム列を状態・ゲームフレーム・epoch として読む
//!   - `runs`       : 同一状態が続く区間の切り出し
//!
//! モジュール名は移設前と同じにしてある。上位が `crate::match_events::…`
//! として再輸出するため、呼び出し側の経路は変わらない。

pub use analysis_context::{context, frame_data};
pub use attack_info_vision::attack_info;
pub use hud_vision::frame_features::FrameFeatures;
pub use hud_vision::{frame_features, round_start};
pub use input_vision::{input_history, input_tracker};
pub use meter_tracker::MeterTimeline;
pub use temporal_confirm::temporal;

pub mod model;
pub mod parameters;
pub mod runs;
pub mod threats;
pub mod timeline;

// 移設前は match_events の平坦な名前空間から見えていた項目。経路を保つ。
pub use model::*;
pub use parameters::*;
pub use threats::{
    CompoundThreat, DefenseResponse, DefenseResponseKind, DpReachability, ProjectileThreat,
    TeleportContext, TeleportEvent, ThreatOutcome,
};
pub use timeline::*;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
