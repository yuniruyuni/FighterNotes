//! 攻め側の抽出。攻撃情報の裏取り、投げ・DI などの行動、SA、ジャンプ、バーンアウト。
//!
//! 抽出器どうしは互いを参照しない。確定済みの観測とイベント型だけを
//! 受け取り、意味のあるイベントを返す。組み立ての順序は上位が持つ。
//!
//! モジュール名は移設前と同じにしてある。`match-event-layer` 側が
//! `crate::…` として再輸出するため、経路は変わらない。

pub use analysis_context::{context, frame_data};
pub use attack_info_vision::attack_info;
pub use hud_vision::{frame_features, round_start};
pub use input_vision::input_history::InputDir;
pub use input_vision::input_tracker::TrackedInput;
pub use input_vision::{input_history, input_tracker};
// crate 全体を glob で借りると、依存側が持つ crate 直下の項目まで
// 引き込んでしまう（ソースを変換する解析ツールが注入するモジュールと
// 名前が衝突する）。必要なモジュールと型だけを明示する。
pub use match_event_model::model::*;
pub use match_event_model::parameters::*;
pub use match_event_model::threats::{
    CompoundThreat, DefenseResponse, DefenseResponseKind, DpReachability, ProjectileThreat,
    TeleportContext, TeleportEvent, ThreatOutcome,
};
pub use match_event_model::timeline::*;
pub use match_event_model::FrameFeatures;
pub use match_event_model::{model, parameters, runs, threats, timeline};
pub use meter_tracker::MeterTimeline;
pub use temporal_confirm::temporal;

pub mod actions;
pub mod attack_evidence;
pub mod burnouts;
pub mod jumps;
pub mod super_arts;
