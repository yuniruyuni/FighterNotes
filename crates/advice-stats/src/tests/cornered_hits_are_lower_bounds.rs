use super::support::*;
use crate::match_events::{CornerSpan, DamageEvent, RoundInfo};

fn damage(frame: u32, victim: u8) -> DamageEvent {
    DamageEvent {
        victim,
        start_frame: frame,
        end_frame: frame + 5,
        pre_freeze_frame: frame,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    }
}

/// 端での被弾は「被弾フレームがその側の corner span 内にある」ものだけを
/// 数える。span は候補 window 内でしか観測できないため下限値であり、
/// span 外や相手側の span、確定 round 外の被弾は数えない。
#[test]
fn cornered_hits_are_lower_bounds() {
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
        CornerSpan {
            side: 1,
            start_frame: 100,
            end_frame: 150,
        },
        CornerSpan {
            side: 2,
            start_frame: 300,
            end_frame: 350,
        },
    ];
    events.damage = vec![
        // 自分(P1)が端で受けた被弾。境界の両端も span 内。
        damage(100, 1),
        damage(150, 1),
        // ヒットの瞬間に追跡が乱れて span が閉じることが多いので、
        // 終端から 30 フレームまでは端での被弾として数える。
        damage(180, 1),
        // 猶予も過ぎた被弾は数えない。
        damage(181, 1),
        // 同じ区間でも、端を背負っていない側の被弾は「追い込んだ」でも
        // 「受けた」でもない。
        damage(120, 2),
        // 相手(P2)が端で受けた被弾 = 追い込んで与えた被弾。
        damage(300, 2),
        // span より前は数えない。
        damage(299, 2),
        // 確定 round の外も数えない。
        damage(6_500, 1),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);
    assert_eq!(stats.cornered_hits_taken, 3);
    assert_eq!(stats.cornered_hits_dealt, 1);

    // side を入れ替えると taken と dealt も入れ替わる。
    let stats = build_tactic_stats(&[], &events, 2, 1);
    assert_eq!(stats.cornered_hits_taken, 1);
    assert_eq!(stats.cornered_hits_dealt, 3);
}
