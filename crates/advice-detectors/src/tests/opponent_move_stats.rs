//! 同定できた相手の技ごとの収支に対する契約。
//!
//! 同定できない接触・確定 round 外・自分の攻撃は数えず、どの値も
//! 下限として扱えることを固定する。

use super::support::*;
use crate::detectors::build_opponent_move_stats;
use crate::match_events::{ContactEvent, InputSegment, MeterState};

fn contact(frame: u32, hit: bool) -> ContactEvent {
    ContactEvent {
        frame,
        attacker: 2,
        victim: 1,
        hit,
        projectile: false,
        round_no: 1,
    }
}

/// 相手ケンのしゃがみ中K(発生 7)の実測列と入力表示を接触の前へ敷く。
fn plant_move(ev: &mut MatchEvents, contact_frame: usize) {
    for frame in contact_frame - 8..contact_frame - 1 {
        ev.meter_state[1][frame] = MeterState::Startup;
    }
    for frame in contact_frame - 1..=contact_frame {
        ev.meter_state[1][frame] = MeterState::Active;
    }
    ev.segments[1].push(InputSegment {
        start_frame: contact_frame as u32 - 8,
        end_frame: contact_frame as u32 - 4,
        dir: "D".to_string(),
        badges: vec!["中K".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
}

#[test]
fn opponent_move_stats_tally_touches_and_answers() {
    let mut ev = empty_events();
    ev.meter_state[1] = vec![MeterState::Free; 2_000];
    ev.meter_confidence[1] = vec![1.0; 2_000];

    // 同定できない接触(入力・実測なし)は数えず、後続の走査も止まらない。
    ev.contacts.push(contact(100, false));
    // 2MK: ヒット 1(-12%)、ガード 2(確反 1・見逃し 1)。
    for frame in [200usize, 400, 600] {
        plant_move(&mut ev, frame);
    }
    ev.contacts.push(contact(200, true));
    ev.contacts.push(contact(400, false));
    ev.contacts.push(contact(600, false));
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 210,
        pre_freeze_frame: 210,
        end_frame: 230,
        hp_before: 1.0,
        hp_after: 0.88,
        drop: 0.12,
        round_no: 1,
    });
    let punish = |contact_frame: u32, outcome, reachability| crate::match_events::PunishChance {
        frame: contact_frame + 10,
        side: 1,
        advantage: 8,
        outcome,
        origin: crate::match_events::PunishOrigin::BlockedMove,
        recovery_start_frame: contact_frame + 2,
        recovery_end_frame: contact_frame + 13,
        source_contact_frame: Some(contact_frame),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    };
    use crate::match_events::{PunishOutcome, PunishReachability};
    ev.punishes.push(punish(
        400,
        PunishOutcome::Success,
        PunishReachability::Confirmed,
    ));
    ev.punishes.push(punish(
        600,
        PunishOutcome::Missed,
        PunishReachability::Confirmed,
    ));
    // 自分の攻撃は相手の技ではない。
    ev.contacts.push(ContactEvent {
        attacker: 1,
        victim: 2,
        ..contact(1_000, true)
    });
    // 確定 round 外は数えない。
    ev.contacts.push(ContactEvent {
        round_no: 2,
        ..contact(1_200, false)
    });

    let stats = build_opponent_move_stats(&ev, 1, Some("KEN"));
    assert_eq!(stats.len(), 1);
    let stat = &stats[0];
    assert_eq!(stat.name, "2MK");
    assert!(!stat.projectile);
    assert_eq!(stat.touches, 3);
    assert_eq!(stat.hits_taken, 1);
    assert!((stat.hp_lost - 0.12).abs() < 1e-6);
    assert_eq!(stat.blocked, 2);
    assert_eq!(stat.punished, 1);
    assert_eq!(stat.punish_missed, 1);
}

/// 触られた回数の多い順に並び、Missed でも近距離を確認できていない
/// 機会は見逃しに数えない。弾として届いた接触は印を持つ。
#[test]
fn stats_are_sorted_and_only_confirmed_misses_count() {
    let mut ev = empty_events();
    ev.meter_state[1] = vec![MeterState::Free; 2_000];
    ev.meter_confidence[1] = vec![1.0; 2_000];

    // 2MK ×1(ガード、Missed だが Unknown)。
    plant_move(&mut ev, 200);
    ev.contacts.push(contact(200, false));
    ev.punishes.push(crate::match_events::PunishChance {
        frame: 210,
        side: 1,
        advantage: 8,
        outcome: crate::match_events::PunishOutcome::Missed,
        origin: crate::match_events::PunishOrigin::BlockedMove,
        recovery_start_frame: 202,
        recovery_end_frame: 213,
        source_contact_frame: Some(200),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: crate::match_events::PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    });
    // 236PP(OD 波動、発生 12) ×3。うち 2 本は弾として届いた。
    // 弾の印は一度立ったら消えない(2 本目の弾で裏返らない)。
    for frame in [500usize, 700, 900] {
        for startup in frame - 13..frame - 1 {
            ev.meter_state[1][startup] = MeterState::Startup;
        }
        ev.meter_state[1][frame - 1] = MeterState::Active;
        ev.meter_state[1][frame] = MeterState::Active;
        ev.segments[1].push(InputSegment {
            start_frame: frame as u32 - 13,
            end_frame: frame as u32 - 9,
            dir: "R".to_string(),
            badges: vec!["弱P".to_string(), "強P".to_string()],
            auto: false,
            throw: false,
            evidence: Default::default(),
        });
    }
    ev.contacts.push(contact(500, false));
    ev.contacts.push(ContactEvent {
        projectile: true,
        ..contact(700, false)
    });
    ev.contacts.push(ContactEvent {
        projectile: true,
        ..contact(900, false)
    });

    let stats = build_opponent_move_stats(&ev, 1, Some("KEN"));
    assert_eq!(
        stats
            .iter()
            .map(|stat| (stat.name.as_str(), stat.touches))
            .collect::<Vec<_>>(),
        vec![("236PP", 3), ("2MK", 1)]
    );
    assert!(stats[0].projectile, "弾として届いた印が消えている");
    assert_eq!(
        stats[1].punish_missed, 0,
        "Unknown の機会を見逃しに数えている"
    );
}

/// 実測発生の走査境界。接触から 16F までの遡りで Startup に届かなければ
/// 実測とせず、届けば直前の連続 Startup だけを数える(別の古い Startup
/// 連なりへ繋げない)。信頼度の足りない Startup 表示は実測に使わない。
#[test]
fn measured_startup_respects_scan_bounds_and_confidence() {
    let identified = |setup: &dyn Fn(&mut MatchEvents)| {
        let mut ev = empty_events();
        ev.meter_state[1] = vec![MeterState::Free; 400];
        ev.meter_confidence[1] = vec![1.0; 400];
        ev.contacts.push(contact(100, false));
        ev.segments[1].push(InputSegment {
            start_frame: 60,
            end_frame: 96,
            dir: "D".to_string(),
            badges: vec!["中K".to_string()],
            auto: false,
            throw: false,
            evidence: Default::default(),
        });
        setup(&mut ev);
        !build_opponent_move_stats(&ev, 1, Some("KEN")).is_empty()
    };
    let plant_run = |ev: &mut MatchEvents, end: usize, len: usize| {
        for frame in end + 1 - len..=end {
            ev.meter_state[1][frame] = MeterState::Startup;
        }
    };

    // Startup 終端が接触の 16F 前(遡り上限ちょうど)なら実測になる。
    assert!(identified(&|ev| plant_run(ev, 84, 7)));
    // 17F 前は別の場面の表示なので実測にしない。
    assert!(!identified(&|ev| plant_run(ev, 83, 7)));
    // 直前の連続だけを数える。切れ目の向こうの古い Startup を足すと
    // 発生が伸びて、実際の技と一致しなくなる。
    assert!(identified(&|ev| {
        plant_run(ev, 96, 7);
        plant_run(ev, 80, 5);
    }));
    // 信頼度の足りない表示は実測に使わない。
    assert!(!identified(&|ev| {
        plant_run(ev, 96, 7);
        ev.meter_confidence[1][92] = 0.2;
    }));
    // 表示が接触フレームまで Startup のまま残る場合、その 1F も実測に
    // 数える。発生 5 は 2MK(7)の許容下限ちょうどで、1F 落とすと
    // 一致しなくなる。
    assert!(identified(&|ev| plant_run(ev, 100, 5)));
    // フレーム 0 まで続く Startup も数え切れる(発生 6 は 2MK の ±2 内)。
    assert!(identified(&|ev| {
        ev.contacts[0].frame = 8;
        ev.segments[1][0].start_frame = 0;
        ev.segments[1][0].end_frame = 4;
        for frame in 0..=5 {
            ev.meter_state[1][frame] = MeterState::Startup;
        }
    }));
}

/// カード文言用のキャラ表記。下線区切りの id を履歴画面と同じ
/// ハイフン表記へ直し、接尾辞は未指定なら何も足さない。
#[test]
fn character_labels_match_the_history_screen_notation() {
    use crate::detectors::{display_character, opponent_suffix};

    assert_eq!(display_character("CHUN_LI"), "CHUN-LI");
    assert_eq!(display_character("KEN"), "KEN");
    assert_eq!(display_character("A_K_I"), "A-K-I");
    assert_eq!(opponent_suffix(Some("E_HONDA")), "(相手: E-HONDA)");
    assert_eq!(opponent_suffix(None), "");
}
