use super::support::*;
use crate::match_events::{DamageDistance, DamageEvent, RoundInfo, SpatialCoverage};

fn damage(victim: u8, start_frame: u32) -> DamageEvent {
    DamageEvent {
        victim,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 10,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    }
}

fn distance(victim: u8, start_frame: u32, value: f32) -> DamageDistance {
    DamageDistance {
        victim,
        damage_start_frame: start_frame,
        distance: value,
    }
}

/// 被弾直前の間合いは、間合いを観測できた自分の被弾だけを距離帯へ分ける。
/// 境界(3/4 と 3/2)ちょうどは遠い側の帯に入る。相手の被弾・対応する
/// 被弾イベントの無い間合い・確定 round 外は数えない。
#[test]
fn hits_taken_ranges_bucket_only_observed_own_hits() {
    let mut events = empty_events();
    events.rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 5_999,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    events.damage = vec![
        damage(1, 100),
        damage(1, 200),
        damage(1, 300),
        damage(1, 400),
        damage(2, 500),
        damage(1, 6_500),
    ];
    events.damage_distances = vec![
        // 密着帯と、その上限ちょうど(3/4 = 差し合い帯の始まり)。
        distance(1, 100, 0.5),
        distance(1, 200, 0.75),
        // 差し合い帯の上限ちょうど(3/2 = 遠め帯の始まり)。
        distance(1, 300, 1.5),
        distance(1, 400, 2.0),
        // 相手の被弾は自分の間合い分布ではない。
        distance(2, 500, 0.5),
        // 確定 round 外。
        distance(1, 6_500, 0.5),
        // 対応する被弾イベントの無い間合いは数えない。
        distance(1, 999, 0.5),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);
    assert_eq!(stats.hits_taken_close_range, 1);
    assert_eq!(stats.hits_taken_mid_range, 1, "3/4 ちょうどは差し合い帯");
    assert_eq!(stats.hits_taken_far_range, 2, "3/2 ちょうどは遠め帯");
}

/// 端滞在時間は定周期サンプルの分母と、side 別の端確認数を own/opponent へ
/// 写す。side を入れ替えれば自分と相手も入れ替わる。
#[test]
fn corner_time_samples_map_sides_to_own_and_opponent() {
    let mut events = empty_events();
    events.spatial_coverage = SpatialCoverage {
        periodic_pair_samples: 40,
        p1_cornered_samples: 12,
        p2_cornered_samples: 3,
        ..Default::default()
    };

    let stats = build_tactic_stats(&[], &events, 1, 2);
    assert_eq!(stats.corner_time_samples, 40);
    assert_eq!(stats.own_corner_time_samples, 12);
    assert_eq!(stats.opponent_corner_time_samples, 3);

    let stats = build_tactic_stats(&[], &events, 2, 1);
    assert_eq!(stats.own_corner_time_samples, 3);
    assert_eq!(stats.opponent_corner_time_samples, 12);
}
