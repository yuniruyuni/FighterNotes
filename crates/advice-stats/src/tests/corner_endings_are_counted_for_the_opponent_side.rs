use super::support::*;
use crate::match_events::{CornerEnding, CornerSpan, RoundInfo};

fn span(start: u32, side: u8, ending: CornerEnding) -> CornerSpan {
    CornerSpan {
        side,
        start_frame: start,
        end_frame: start + 60,
        ending,
    }
}

/// 「追い込んだ攻めの終わり方」は相手側 span の分類だけを数える。
/// 終わり方を確認できなかった区間と、自分が端を背負った区間、確定
/// round 外の区間は数えない。
#[test]
fn corner_endings_are_counted_for_the_opponent_side() {
    let mut events = empty_events();
    events.rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 5_999,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    events.corner_spans = vec![
        // 相手(P2)を追い込んだ区間の終わり方。
        span(100, 2, CornerEnding::SideSwap),
        span(300, 2, CornerEnding::SideSwap),
        span(500, 2, CornerEnding::Separated),
        // 確認できなかった終わりは数えない。
        span(700, 2, CornerEnding::Unobserved),
        // 自分(P1)が背負った区間は「追い込んだ」ではない。
        span(900, 1, CornerEnding::SideSwap),
        // 確定 round の外は数えない。
        span(6_500, 2, CornerEnding::Separated),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);
    assert_eq!(stats.corner_escapes_allowed, 2);
    assert_eq!(stats.corner_pressure_released, 1);

    // side を入れ替えると、相手(P1)の span だけが分母になる。
    let stats = build_tactic_stats(&[], &events, 2, 1);
    assert_eq!(stats.corner_escapes_allowed, 1);
    assert_eq!(stats.corner_pressure_released, 0);
}
