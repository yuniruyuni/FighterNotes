//! 攻撃が触れた瞬間を、二人のメーターから読み取るところに対するテスト。
//!
//! 攻撃が当たるとヒットストップが入り、二人のメーターが同時に数フレーム
//! 止まる。止まっている間、殴った側は攻撃判定、殴られた側は硬直を表示
//! している。この組み合わせが接触の印。
//!
//! ガードもヒットストップを起こし、硬直の表示も同じなので、メーターだけ
//! では当たったのか防がれたのかが分からない。HP が減ったかどうかで分ける。

use super::*;
use meter_tracker::{TimelineEntry, TimelineSegment};

/// 指定の区間だけ同じ状態で止まっているメーター。
fn timeline(side: &str, spans: &[(i64, i64, &str)]) -> MeterTimeline {
    timeline_in_segment(side, 0, spans)
}

/// 区間の識別子まで指定するメーター。メーターが読み直された前後は
/// 別の区間になる。
fn timeline_in_segment(side: &str, segment_id: i32, spans: &[(i64, i64, &str)]) -> MeterTimeline {
    segmented(side, &[(segment_id, spans)])
}

/// 停止区間の並び（開始動画フレーム, 終了動画フレーム, 状態）。
type Spans<'a> = &'a [(i64, i64, &'a str)];

/// 複数の区間を持つメーター。
fn segmented(side: &str, groups: &[(i32, Spans<'_>)]) -> MeterTimeline {
    MeterTimeline {
        side: side.to_string(),
        segments: groups
            .iter()
            .map(|&(segment_id, spans)| TimelineSegment {
                segment_id,
                entries: spans
                    .iter()
                    .map(|&(first, last, state)| TimelineEntry {
                        game_frame: first,
                        state: state.to_string(),
                        video_frame_first: first,
                        video_frame_last: last,
                        confidence: 1.0,
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn rounds() -> Vec<RoundInfo> {
    vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 3_000,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }]
}

/// 相手が受けた被弾。
fn damage_to(victim: u8, start_frame: u32) -> DamageEvent {
    DamageEvent {
        victim,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 30,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    }
}

fn extract(
    left: &MeterTimeline,
    right: &MeterTimeline,
    damage: &[DamageEvent],
) -> Vec<ContactEvent> {
    extract_contacts(left, right, damage, &rounds())
}

// ── 接触が成立する条件 ───────────────────────────────────────────────────

/// 片方が攻撃判定、もう片方が硬直で同時に止まっていれば接触。
#[test]
fn an_attack_meeting_a_stun_is_a_contact() {
    let left = timeline("left", &[(100, 106, "active")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let contacts = extract(&left, &right, &[]);

    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0].attacker, 1);
    assert_eq!(contacts[0].victim, 2);
    assert_eq!(contacts[0].frame, 100);
}

/// 向きは逆にもなる。
#[test]
fn the_attacker_can_be_either_side() {
    let left = timeline("left", &[(100, 106, "stun")]);
    let right = timeline("right", &[(100, 106, "active")]);

    let contacts = extract(&left, &right, &[]);

    assert_eq!(contacts[0].attacker, 2);
    assert_eq!(contacts[0].victim, 1);
}

/// 二人とも硬直していれば相打ち。どちらも殴っている。
#[test]
fn two_stuns_at_once_are_a_trade() {
    let left = timeline("left", &[(100, 106, "stun")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let contacts = extract(&left, &right, &[]);

    assert_eq!(contacts.len(), 2, "相打ちを片方向しか記録していない");
    assert_eq!(contacts[0].attacker, 1);
    assert_eq!(contacts[1].attacker, 2);
}

/// 飛び道具でも接触は接触。ただし飛び道具として記録する。距離が
/// 違うので、以降の判断が変わる。
#[test]
fn a_projectile_contact_is_marked_as_one() {
    let left = timeline("left", &[(100, 106, "projectile_active")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let contacts = extract(&left, &right, &[]);

    assert!(contacts[0].projectile, "飛び道具を通常技として記録している");
}

/// 飛び道具の印は左右どちらの攻撃にも付く。
#[test]
fn a_projectile_from_the_right_is_marked_too() {
    let left = timeline("left", &[(100, 106, "stun")]);
    let right = timeline("right", &[(100, 106, "projectile_active")]);

    let contacts = extract(&left, &right, &[]);

    assert!(contacts[0].projectile, "右側の飛び道具を見落としている");
}

/// 通常技には飛び道具の印を付けない。左右とも。
#[test]
fn a_normal_attack_is_never_marked_as_a_projectile() {
    let from_left = extract(
        &timeline("left", &[(100, 106, "active")]),
        &timeline("right", &[(100, 106, "stun")]),
        &[],
    );
    let from_right = extract(
        &timeline("left", &[(100, 106, "stun")]),
        &timeline("right", &[(100, 106, "active")]),
        &[],
    );

    assert!(!from_left[0].projectile);
    assert!(!from_right[0].projectile);
}

/// 相打ちでは、どちらが飛び道具だったか分からない。断定しない。
#[test]
fn a_trade_does_not_claim_which_side_was_a_projectile() {
    let left = timeline("left", &[(100, 106, "stun")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let contacts = extract(&left, &right, &[]);

    assert!(contacts.iter().all(|contact| !contact.projectile));
}

/// 二人とも攻撃判定なら、まだ触れていない。
#[test]
fn two_attacks_at_once_are_not_a_contact() {
    let left = timeline("left", &[(100, 106, "active")]);
    let right = timeline("right", &[(100, 106, "active")]);

    assert!(extract(&left, &right, &[]).is_empty());
}

/// 止まっていない表示は接触の印にならない。技の途中で数フレーム
/// 同じ状態が続くのは普通のこと。
#[test]
fn a_pause_too_short_to_be_hitstop_is_not_a_contact() {
    let long = extract(
        &timeline("left", &[(100, 104, "active")]),
        &timeline("right", &[(100, 104, "stun")]),
        &[],
    );
    let short = extract(
        &timeline("left", &[(100, 103, "active")]),
        &timeline("right", &[(100, 103, "stun")]),
        &[],
    );

    assert_eq!(long.len(), 1, "ちょうどの長さを落としている");
    assert!(short.is_empty(), "短い停止を接触にしている");
}

/// 止まっている区間が十分に重なっていなければ、同じ瞬間ではない。
#[test]
fn pauses_that_barely_overlap_are_not_the_same_moment() {
    let overlapping = extract(
        &timeline("left", &[(100, 106, "active")]),
        &timeline("right", &[(103, 109, "stun")]),
        &[],
    );
    let separate = extract(
        &timeline("left", &[(100, 106, "active")]),
        &timeline("right", &[(104, 110, "stun")]),
        &[],
    );

    assert_eq!(overlapping.len(), 1, "重なった停止を落としている");
    assert!(separate.is_empty(), "ずれた停止を同じ接触にしている");
}

/// 硬直しているのが片方だけで、もう片方が攻撃判定でもなければ、
/// 誰が殴ったのか分からない。相打ちにはしない。
#[test]
fn a_lone_stun_facing_a_recovery_is_not_a_trade() {
    let left = timeline("left", &[(100, 106, "stun")]);
    let right = timeline("right", &[(100, 106, "recovery")]);

    assert!(extract(&left, &right, &[]).is_empty());
    assert!(extract(&right, &left, &[]).is_empty());
}

/// メーターが読み直された前後の停止は繋がない。別の区間の時刻を
/// 突き合わせても意味が無い。
#[test]
fn pauses_from_different_meter_segments_are_not_paired() {
    let left = timeline_in_segment("left", 0, &[(100, 106, "active")]);
    let right = timeline_in_segment("right", 1, &[(100, 106, "stun")]);

    assert!(extract(&left, &right, &[]).is_empty());
}

/// 区間の違う停止に当たっても、その先の停止を見るのをやめない。
/// 読み直しの前後で停止が重なって並ぶことはある。
#[test]
fn a_mismatched_segment_does_not_end_the_search() {
    let left = timeline_in_segment("left", 0, &[(105, 115, "active")]);
    let right = segmented(
        "right",
        &[(1, &[(100, 110, "stun")]), (0, &[(105, 115, "stun")])],
    );

    let contacts = extract(&left, &right, &[]);

    assert_eq!(contacts.len(), 1, "区間違いで打ち切っている");
    assert_eq!(contacts[0].frame, 105);
}

/// ラウンドの外の接触は記録しない。
#[test]
fn a_contact_outside_any_round_is_not_recorded() {
    let left = timeline("left", &[(4_000, 4_006, "active")]);
    let right = timeline("right", &[(4_000, 4_006, "stun")]);

    assert!(extract(&left, &right, &[]).is_empty());
}

/// ラウンド外の停止に当たっても、その先の停止を見るのをやめない。
/// ラウンド開始をまたいで停止が重なって並ぶことはある。
#[test]
fn a_contact_outside_a_round_does_not_end_the_search() {
    let late_round = vec![RoundInfo {
        round_no: 1,
        start_frame: 1_000,
        end_frame: 3_000,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let left = timeline("left", &[(905, 1_005, "active")]);
    let right = timeline("right", &[(900, 910, "stun"), (1_000, 1_010, "stun")]);

    let contacts = extract_contacts(&left, &right, &[], &late_round);

    assert_eq!(contacts.len(), 1, "ラウンド外で打ち切っている");
    assert_eq!(contacts[0].frame, 1_000);
}

// ── 当たったのか防がれたのか ─────────────────────────────────────────────

/// 直後に HP が減っていれば当たっている。
#[test]
fn health_lost_right_after_means_it_hit() {
    let left = timeline("left", &[(100, 106, "active")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let contacts = extract(&left, &right, &[damage_to(2, 105)]);

    assert!(contacts[0].hit);
}

/// HP が減っていなければガードされている。ガードでも硬直は同じ表示。
#[test]
fn no_health_lost_means_it_was_blocked() {
    let left = timeline("left", &[(100, 106, "active")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let contacts = extract(&left, &right, &[]);

    assert!(!contacts[0].hit);
}

/// 離れた被弾は、その接触の結果ではない。
#[test]
fn health_lost_far_from_the_contact_is_not_its_result() {
    let left = timeline("left", &[(100, 106, "active")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let inside = extract(&left, &right, &[damage_to(2, 125)]);
    let outside = extract(&left, &right, &[damage_to(2, 126)]);

    assert!(inside[0].hit, "窓の内側の被弾を落としている");
    assert!(!outside[0].hit, "窓の外の被弾を結び付けている");
}

/// 攻撃した側が受けた被弾は、その攻撃が当たった証拠にならない。
#[test]
fn health_lost_by_the_attacker_does_not_make_it_a_hit() {
    let left = timeline("left", &[(100, 106, "active")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let contacts = extract(&left, &right, &[damage_to(1, 105)]);

    assert!(!contacts[0].hit, "殴った側の被弾を結び付けている");
}

// ── 並べ方 ───────────────────────────────────────────────────────────────

/// 接触は時間順に並べる。
#[test]
fn the_contacts_are_ordered_by_time() {
    let left = timeline("left", &[(300, 306, "active"), (100, 106, "active")]);
    let right = timeline("right", &[(300, 306, "stun"), (100, 106, "stun")]);

    let contacts = extract(&left, &right, &[]);

    assert_eq!(contacts[0].frame, 100);
    assert_eq!(contacts[1].frame, 300);
}

/// 同じ接触を二度記録しない。停止の記録が重なって出ることがある。
#[test]
fn the_same_contact_is_not_recorded_twice() {
    let left = timeline("left", &[(100, 106, "active"), (100, 108, "active")]);
    let right = timeline("right", &[(100, 106, "stun")]);

    let contacts = extract(&left, &right, &[]);

    assert_eq!(contacts.len(), 1, "同じ接触を重複させている");
}
