//! 集計のテストで使う観測列。
//!
//! 器（何も起きていない 1 ラウンド）はイベント層の test-support から借りる。
//! ここへ主題のイベントだけを足して、数え方を確かめる。

pub(super) use crate::match_events::BurnoutPeriod;
pub(super) use crate::stats::*;
pub(super) use crate::*;
pub(super) use match_event_layer::test_support::empty_events;
