//! 指摘の形と、その手前の共通の見方。
//!
//! どんな指摘を出すかを決める検出器や、それを数え上げる統計は、いずれも
//! ここで定義した型と閾値の上に載る。上流の観測にも下流の組み立てにも
//! 依存しない土台なので、独立させてある。
//!
//!   - `model`      : 指摘カード・レポート・戦術統計の形
//!   - `parameters` : 指摘の閾値と ruleset 版数
//!   - `decisions`  : 状況ラベル × 選択肢という共通の見方への射影
//!
//! モジュール名は移設前と同じにしてある。`advice-report` 側が
//! `crate::advice::…` として再輸出するため、呼び出し側の経路は変わらない。

pub use analysis_context::{context, frame_data};
pub use match_event_layer::{frame_features, match_events};

pub mod decisions;
pub mod model;
pub mod parameters;
