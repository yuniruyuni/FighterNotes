//! イベント層のテストで使う観測列の組み立て補助。
//!
//! 実体は下の層にある。観測列そのものは `match-event-model`、抽出器を
//! 呼ぶものは各抽出 crate が持つ。ここは経路を保つための再輸出。

pub use crate::match_events::*;
pub use match_event_defense::test_support::*;
