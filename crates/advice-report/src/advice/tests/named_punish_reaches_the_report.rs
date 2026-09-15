use super::support::*;
use crate::match_events::{
    ContactEvent, InputSegment, MeterState, PunishChance, PunishOrigin, PunishOutcome,
    PunishReachability,
};

/// 相手キャラがレポートの文脈から検出器まで届き、同定できた技名が
/// クリップに載る。結線を忘れても例外は出ず、技名が黙って消えるだけ
/// なので、レポート境界で固定する。
#[test]
fn named_punish_reaches_the_report() {
    let mut ev = empty_events();
    ev.punishes.push(PunishChance {
        frame: 210,
        side: 1,
        advantage: 8,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 202,
        recovery_end_frame: 213,
        source_contact_frame: Some(200),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    });
    ev.contacts.push(ContactEvent {
        frame: 200,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    });
    // 相手(P2)のしゃがみ中K(発生 7)の実測列と入力表示。
    ev.meter_state[1] = vec![MeterState::Free; 300];
    for frame in 192..199 {
        ev.meter_state[1][frame] = MeterState::Startup;
    }
    for frame in 199..=200 {
        ev.meter_state[1][frame] = MeterState::Active;
    }
    ev.meter_confidence[1] = vec![1.0; 300];
    ev.segments[1] = vec![InputSegment {
        start_frame: 192,
        end_frame: 196,
        dir: "D".to_string(),
        badges: vec!["中K".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }];

    let report = detector_test_report_with_players(&ev, "p1", Some("LUKE"), Some("KEN"));
    let card = report
        .cards
        .iter()
        .find(|card| card.id == "punish_missed")
        .expect("確反見逃しカード");
    assert!(
        card.evidence[0].label.contains("相手の 2MK"),
        "相手キャラが検出器へ届いていない: {}",
        card.evidence[0].label
    );
    // 同じ同定は「相手の技の内訳」にも届く。
    assert_eq!(report.opponent_move_stats.len(), 1);
    assert_eq!(report.opponent_move_stats[0].name, "2MK");
    assert_eq!(report.opponent_move_stats[0].blocked, 1);
    assert_eq!(report.opponent_move_stats[0].punish_missed, 1);
    // 回答列の材料(自キャラの確定候補)も自キャラ文脈から届く。
    assert_eq!(report.opponent_move_stats[0].blocked_advantage, Some(8));
    assert_eq!(
        report.opponent_move_stats[0].counters,
        vec!["623PP", "5/6LPLK"]
    );
}

/// 自キャラもレポートの文脈から使用分布の集計まで届く。結線を忘れても
/// 例外は出ず、分布が黙って空になるだけなので、レポート境界で固定する。
#[test]
fn own_move_usage_reaches_the_report() {
    let mut ev = empty_events();
    // 自分(P1 ルーク)のしゃがみ中K(発生 8)の実測列と入力表示。
    ev.meter_state[0] = vec![MeterState::Free; 300];
    for frame in 200..208 {
        ev.meter_state[0][frame] = MeterState::Startup;
    }
    ev.meter_state[0][208] = MeterState::Active;
    ev.meter_confidence[0] = vec![1.0; 300];
    ev.segments[0] = vec![InputSegment {
        start_frame: 200,
        end_frame: 204,
        dir: "D".to_string(),
        badges: vec!["中K".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }];

    let report = detector_test_report_with_players(&ev, "p1", Some("LUKE"), Some("KEN"));
    assert_eq!(report.own_move_usage.len(), 1, "自分の技が届いていない");
    assert_eq!(report.own_move_usage[0].name, "2MK");
    assert_eq!(report.own_move_usage[0].uses, 1);
}
