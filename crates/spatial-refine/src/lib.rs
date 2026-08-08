//! 画面上の位置関係によるイベントの再評価。
//!
//! HUD だけでは決まらない事柄——どちらが仕掛けたのか、その技は届く距離に
//! あったのか——を、動きのある領域の観測から詰める。イベント層が「怪しい」
//! と印を付けた区間だけを対象にする。
//!
//! モジュール名は移設前と同じにしてある。`video-analyzer` 側が
//! `crate::spatial` として再輸出するため、呼び出し側の経路は変わらない。

pub use analysis_context::{context, frame_data};
pub use match_event_layer::match_events;

pub mod spatial;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;

// 移設前は video-analyzer の crate root にあった再輸出。深い階層の
// モジュールがここを経由して型を参照しているため、同じ形で残す。
pub use analysis_context::context::{AnalysisContext, PlayerContext};
pub use match_event_layer::match_events::*;
pub use spatial::*;
