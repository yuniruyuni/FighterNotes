//! 確定済みの観測列から、助言が扱う意味的なイベントを組み立てる。
//!
//! ラウンド分割、被弾のまとまり、ジャンプ・投げ・無敵技・Drive Impact の
//! 成否、ダウンと起き攻め、空振りと差し返しなど、「何が起きたか」を
//! ここで確定させる。「だから何をすべきか」は上位の助言層が扱う。
//!
//! モジュール名は移設前と同じにしてある。`video-analyzer` 側が
//! `crate::match_events` として再輸出するため、呼び出し側の経路は変わらない。

pub use analysis_context::{context, frame_data};
pub use attack_info_vision::attack_info;
pub use hud_vision::{frame_features, round_start};
pub use input_vision::{input_history, input_tracker};
pub use temporal_confirm::temporal;

pub mod match_events;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
