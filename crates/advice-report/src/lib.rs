//! 試合イベントから、次に何を直すべきかを組み立てる。
//!
//! 「何が起きたか」はイベント層で確定している。ここが決めるのは、それを
//! どう受け止めるか——どの指摘を出し、どれを優先し、どの場面を根拠として
//! 見せるか。
//!
//! モジュール名は移設前と同じにしてある。`video-analyzer` 側が
//! `crate::advice` として再輸出するため、呼び出し側の経路は変わらない。

pub use analysis_context::{context, frame_data};
pub use attack_info_vision::attack_info;
pub use hud_vision::{frame_features, round_start};
pub use match_event_layer::match_events;
pub use temporal_confirm::temporal;

pub mod advice;

// 移設前は video-analyzer の crate root にあった再輸出。深い階層の
// モジュールがここを経由して型を参照しているため、同じ形で残す。
pub use advice::*;
pub use analysis_context::context::{AnalysisContext, PlayerContext};
pub use match_event_layer::match_events::*;
