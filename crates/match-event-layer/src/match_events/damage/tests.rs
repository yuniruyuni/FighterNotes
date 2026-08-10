//! HP の下降を一連の被弾へまとめるところに対するテスト。
//!
//! 体力ゲージは 1 発ごとに階段状に減る。コンボは階段の連続だが、
//! 別々の被弾も階段の連続として見える。区別できるのは間隔だけ。
//!
//! 間隔は動画のフレーム数では測れない。SA の暗転中は動画だけが進んで
//! ゲームは止まっているので、その分を引いてから測る。

use super::*;
use match_event_model::test_support::feat;

/// HP 列から観測列を作る。左右とも同じ長さ。
fn features_for(left: &[f32], right: &[f32]) -> Vec<FrameFeatures> {
    (0..left.len())
        .map(|frame| feat(frame as u32, left[frame], right[frame]))
        .collect()
}

/// 指定のフレームで階段状に落ちる HP 列。
fn stairs(length: usize, steps: &[(usize, f32)]) -> Vec<f32> {
    let mut values = vec![1.0f32; length];
    for &(frame, drop) in steps {
        for value in &mut values[frame..] {
            *value -= drop;
        }
    }
    values
}

fn round(length: usize) -> Vec<RoundInfo> {
    vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: length as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }]
}

/// 右側だけが削られる試合の被弾列。
fn damage_of(right: &[f32]) -> Vec<DamageEvent> {
    let left = vec![1.0f32; right.len()];
    let features = features_for(&left, right);
    let hp = [left, right.to_vec()];
    extract_damage_sequences(&features, &hp, &round(right.len()), &[], [&[], &[]])
}

// ── 何を下降とみなすか ───────────────────────────────────────────────────

/// 一続きの下降は 1 回の被弾。
#[test]
fn one_run_of_falling_health_is_one_event() {
    let events = damage_of(&stairs(200, &[(50, 0.1)]));

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].victim, 2);
    assert_eq!(events[0].start_frame, 50);
    assert!((events[0].drop - 0.1).abs() < 1e-5);
    assert!((events[0].hp_before - 1.0).abs() < 1e-5);
    assert!((events[0].hp_after - 0.9).abs() < 1e-5);
}

/// 読み取りの揺れ程度の下降は被弾ではない。
#[test]
fn a_wobble_too_small_to_be_a_hit_is_ignored() {
    assert!(damage_of(&stairs(200, &[(50, 0.001)])).is_empty());
}

/// 削り 1 回分にも満たない下降は指摘の対象にしない。
#[test]
fn a_drop_below_the_reporting_floor_is_not_an_event() {
    assert!(
        damage_of(&stairs(200, &[(50, 0.01)])).is_empty(),
        "小さすぎる下降を被弾にしている"
    );
    assert_eq!(
        damage_of(&stairs(200, &[(50, 0.02)])).len(),
        1,
        "報告すべき下降を落としている"
    );
}

/// 既に倒れている相手の HP が動いても被弾ではない。KO 演出中の
/// バーの揺れを追撃と数えない。
#[test]
fn health_moving_after_a_knockout_is_not_a_hit() {
    let mut values = stairs(200, &[(50, 0.97)]);
    for value in &mut values[100..] {
        *value -= 0.02;
    }

    let events = damage_of(&values);

    assert_eq!(events.len(), 1, "KO 後の下降を被弾にしている");
    assert_eq!(events[0].start_frame, 50);
}

// ── どこで切るか ─────────────────────────────────────────────────────────

/// 近い下降は一連のコンボ。
#[test]
fn hits_close_together_are_one_combo() {
    let events = damage_of(&stairs(300, &[(50, 0.1), (90, 0.1)]));

    assert_eq!(events.len(), 1, "コンボを分けている");
    assert_eq!(events[0].start_frame, 50);
    assert_eq!(events[0].end_frame, 90);
    assert!((events[0].drop - 0.2).abs() < 1e-5);
}

/// 間が空けば別の被弾。相手が動ける時間があった。
#[test]
fn hits_far_apart_are_separate_events() {
    let events = damage_of(&stairs(300, &[(50, 0.1), (150, 0.1)]));

    assert_eq!(events.len(), 2, "別々の被弾をまとめている");
    assert_eq!(events[0].start_frame, 50);
    assert_eq!(events[1].start_frame, 150);
}

/// 切れ目は最後の下降から測る。コンボの先頭からではない。
#[test]
fn the_gap_is_measured_from_the_last_hit_not_the_first() {
    let events = damage_of(&stairs(400, &[(50, 0.1), (90, 0.1), (130, 0.1)]));

    assert_eq!(events.len(), 1, "先頭からの距離で切っている");
    assert_eq!(events[0].end_frame, 130);
}

/// 演出で止まっていた時間は間隔に数えない。SA の暗転を挟んだ後半の
/// ヒットが別の被弾になると、1 回のコンボが 2 回に見える。
#[test]
fn a_freeze_between_hits_does_not_split_the_combo() {
    let right = stairs(400, &[(50, 0.1), (160, 0.1)]);
    let left = vec![1.0f32; right.len()];
    let features = features_for(&left, &right);
    let hp = [left, right.clone()];

    let split = extract_damage_sequences(&features, &hp, &round(right.len()), &[], [&[], &[]]);
    let joined = extract_damage_sequences(
        &features,
        &hp,
        &round(right.len()),
        &[(60, 150)],
        [&[], &[]],
    );

    assert_eq!(split.len(), 2, "止まっていなければ別の被弾");
    assert_eq!(joined.len(), 1, "演出の停止を間隔に数えている");
}

/// 硬直が途切れていなければ、間隔が空いていても同じコンボ。相手に
/// 動ける瞬間が無かった。
#[test]
fn an_unbroken_stun_keeps_the_hits_together() {
    let right = stairs(400, &[(50, 0.1), (160, 0.1)]);
    let left = vec![1.0f32; right.len()];
    let features = features_for(&left, &right);
    let hp = [left, right.clone()];
    let stunned = vec![true; right.len()];

    let joined = extract_damage_sequences(
        &features,
        &hp,
        &round(right.len()),
        &[],
        [&[], &stunned[..]],
    );

    assert_eq!(joined.len(), 1, "硬直の継続を見ていない");

    let mut broken = stunned.clone();
    broken[100] = false;
    let split =
        extract_damage_sequences(&features, &hp, &round(right.len()), &[], [&[], &broken[..]]);

    assert_eq!(split.len(), 2, "硬直の切れ目を無視している");
}

/// 硬直は殴られた側のものを見る。殴った側の硬直では繋がらない。
#[test]
fn the_stun_that_matters_is_the_victims() {
    let right = stairs(400, &[(50, 0.1), (160, 0.1)]);
    let left = vec![1.0f32; right.len()];
    let features = features_for(&left, &right);
    let hp = [left, right.clone()];
    let stunned = vec![true; right.len()];

    let events = extract_damage_sequences(
        &features,
        &hp,
        &round(right.len()),
        &[],
        [&stunned[..], &[]],
    );

    assert_eq!(events.len(), 2, "相手側の硬直で繋いでいる");
}

// ── ラウンドの内側だけ ───────────────────────────────────────────────────

/// ラウンドの外の下降は数えない。
#[test]
fn drops_outside_the_round_are_not_counted() {
    let right = stairs(400, &[(50, 0.1), (300, 0.1)]);
    let left = vec![1.0f32; right.len()];
    let features = features_for(&left, &right);
    let hp = [left, right.clone()];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 100,
        end_frame: 200,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];

    let events = extract_damage_sequences(&features, &hp, &rounds, &[], [&[], &[]]);

    assert!(
        events.is_empty(),
        "ラウンド外の下降を拾っている: {events:?}"
    );
}

/// 被弾にはそのラウンドの番号が付く。
#[test]
fn each_event_carries_its_round_number() {
    let right = stairs(400, &[(50, 0.1), (300, 0.1)]);
    let left = vec![1.0f32; right.len()];
    let features = features_for(&left, &right);
    let hp = [left, right.clone()];
    let rounds = vec![
        RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: 199,
            winner: None,
            p1_hp_end: 1.0,
            p2_hp_end: 1.0,
        },
        RoundInfo {
            round_no: 2,
            start_frame: 200,
            end_frame: 399,
            winner: None,
            p1_hp_end: 1.0,
            p2_hp_end: 1.0,
        },
    ];

    let events = extract_damage_sequences(&features, &hp, &rounds, &[], [&[], &[]]);

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].round_no, 1);
    assert_eq!(events[1].round_no, 2);
}

/// 左右の被弾は時間順に並ぶ。側ごとにまとめて返さない。
#[test]
fn events_from_both_sides_are_interleaved_by_time() {
    let left = stairs(400, &[(100, 0.1)]);
    let right = stairs(400, &[(50, 0.1), (200, 0.1)]);
    let features = features_for(&left, &right);
    let hp = [left, right];

    let events = extract_damage_sequences(&features, &hp, &round(400), &[], [&[], &[]]);

    assert_eq!(
        events
            .iter()
            .map(|event| (event.start_frame, event.victim))
            .collect::<Vec<_>>(),
        vec![(50, 2), (100, 1), (200, 2)]
    );
}

// ── 演出でラウンドが伸びる場合 ───────────────────────────────────────────

fn freeze_round(end_frame: u32) -> RoundInfo {
    RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }
}

/// ラウンドの終端が演出の途中にあれば、演出の末尾まで伸ばす。
/// KO の最後の一撃が演出に隠れて、ラウンドの外へこぼれる。
#[test]
fn a_round_ending_inside_a_freeze_is_extended_to_its_end() {
    let left = vec![1.0f32; 400];
    let right = stairs(400, &[(100, 0.5)]);
    let features = features_for(&left, &right);
    let hp = [left, right];
    let mut rounds = vec![freeze_round(150)];

    extend_rounds_through_freezes(&mut rounds, &features, &hp, &[(120, 250)]);

    assert_eq!(rounds[0].end_frame, 250);
}

/// 演出の外で終わっているラウンドは伸ばさない。
#[test]
fn a_round_ending_outside_any_freeze_is_left_alone() {
    let left = vec![1.0f32; 400];
    let right = stairs(400, &[(100, 0.5)]);
    let features = features_for(&left, &right);
    let hp = [left, right];
    let mut rounds = vec![freeze_round(300)];

    extend_rounds_through_freezes(&mut rounds, &features, &hp, &[(120, 250)]);

    assert_eq!(rounds[0].end_frame, 300);
    assert_eq!(rounds[0].p2_hp_end, 1.0, "伸ばしていないのに HP を触った");
}

/// 次のラウンドへは食い込まない。
#[test]
fn the_extension_stops_before_the_next_round() {
    let left = vec![1.0f32; 400];
    let right = stairs(400, &[(100, 0.5)]);
    let features = features_for(&left, &right);
    let hp = [left, right];
    let mut rounds = vec![
        freeze_round(150),
        RoundInfo {
            round_no: 2,
            start_frame: 200,
            end_frame: 399,
            winner: None,
            p1_hp_end: 1.0,
            p2_hp_end: 1.0,
        },
    ];

    extend_rounds_through_freezes(&mut rounds, &features, &hp, &[(120, 250)]);

    assert_eq!(rounds[0].end_frame, 199, "次のラウンドへ食い込んでいる");
}

/// 伸ばした先の HP を終値として取り直す。演出の手前の値のままだと
/// 最後の一撃が消える。
#[test]
fn the_end_of_round_health_follows_the_extension() {
    let left = vec![1.0f32; 400];
    let right = stairs(400, &[(100, 0.4), (200, 0.4)]);
    let features = features_for(&left, &right);
    let hp = [left, right];
    let mut rounds = vec![freeze_round(150)];

    extend_rounds_through_freezes(&mut rounds, &features, &hp, &[(120, 250)]);

    assert!(
        (rounds[0].p2_hp_end - 0.2).abs() < 1e-5,
        "伸ばす前の HP を残している: {}",
        rounds[0].p2_hp_end
    );
    assert!((rounds[0].p1_hp_end - 1.0).abs() < 1e-5);
}

/// 読めなかった HP は終値に採らない。
#[test]
fn an_unreadable_health_is_not_taken_as_the_end_value() {
    let mut left = vec![1.0f32; 400];
    let mut right = stairs(400, &[(100, 0.4)]);
    left[250] = -1.0;
    right[250] = -1.0;
    let features = features_for(&left, &right);
    let hp = [left, right];
    let mut rounds = vec![freeze_round(150)];

    extend_rounds_through_freezes(&mut rounds, &features, &hp, &[(120, 250)]);

    assert_eq!(rounds[0].end_frame, 250);
    assert_eq!(rounds[0].p1_hp_end, 1.0, "読めない値を終値にしている");
    assert_eq!(rounds[0].p2_hp_end, 1.0, "読めない値を終値にしている");
}
