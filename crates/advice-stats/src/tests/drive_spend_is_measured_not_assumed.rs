use super::support::*;
use crate::match_events::{DriveImpactEvent, DriveImpactOutcome, RoundInfo};

/// 自分の Drive ゲージ量を frame ごとに与える features を作る。
fn features_with_drive(drive: &[f32], uncertain_from: Option<usize>) -> Vec<FrameFeatures> {
    drive
        .iter()
        .enumerate()
        .map(|(index, &ratio)| FrameFeatures {
            frame_index: index as u32,
            fps: 60.0,
            own_hp: 1.0,
            opponent_hp: 1.0,
            is_match_screen: true,
            own_meter_state: None,
            opponent_meter_state: None,
            left_hp_score: 0.1,
            right_hp_score: 0.1,
            left_drive_ratio: ratio,
            right_drive_ratio: 1.0,
            left_burnout: false,
            right_burnout: false,
            left_drive_uncertain: uncertain_from.is_some_and(|from| index >= from),
            right_drive_uncertain: false,
            left_super_value: 0.0,
            right_super_value: 0.0,
            left_super_uncertain: true,
            right_super_uncertain: true,
            left_ca_ready: false,
            right_ca_ready: false,
            left_hp_raw: 1.0,
            right_hp_raw: 1.0,
            left_hp_raw_quality: 0.0,
            right_hp_raw_quality: 0.0,
        })
        .collect()
}

fn impact(frame: u32, outcome: DriveImpactOutcome, damage: f32) -> DriveImpactEvent {
    DriveImpactEvent {
        side: 1,
        input_frame: frame,
        active_frame: Some(frame + 10),
        contact_frame: Some(frame + 12),
        outcome,
        damage,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn events_with_impact(event: DriveImpactEvent) -> MatchEvents {
    let mut events = empty_events();
    events.rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 199,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    events.drive_impacts = vec![event];
    events
}

/// 消費量は SF6 の本数を仮定せず、実際のゲージ減少から取る。
/// 定数を書くと、行動の取り違えや仕様変更がそのまま誤った数値になる。
#[test]
fn drive_spend_comes_from_the_observed_gauge_drop() {
    let mut drive = vec![1.0_f32; 200];
    // f50 の DI で 1本ぶん（≈0.167）減り、数フレームかけて反映される。
    for (index, value) in drive.iter_mut().enumerate().skip(52) {
        *value = if index < 56 { 0.9 } else { 0.833 };
    }
    let features = features_with_drive(&drive, None);
    let events = events_with_impact(impact(50, DriveImpactOutcome::Hit, 0.22));

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert_eq!(stats.own_di_used, 1);
    assert_eq!(stats.own_di_hit, 1);
    assert_eq!(stats.drive_spend_samples, 1);
    assert!((stats.drive_spent_on_impacts - 0.167).abs() < 0.005);
    assert!((stats.drive_damage_from_impacts - 0.22).abs() < 1e-6);
}

/// ゲージを読めない区間は消費量に数えない。欠測を 0 消費として積むと、
/// 1本あたりの効率が実際より良く出る。
#[test]
fn an_unreadable_gauge_contributes_no_spend() {
    let drive = vec![1.0_f32; 200];
    let features = features_with_drive(&drive, Some(51));
    let events = events_with_impact(impact(50, DriveImpactOutcome::Hit, 0.22));

    let stats = build_tactic_stats(&features, &events, 1, 2);

    // 行動そのものは数えるが、消費量の分母には入れない。
    assert_eq!(stats.own_di_used, 1);
    assert_eq!(stats.drive_spend_samples, 0);
    assert_eq!(stats.drive_spent_on_impacts, 0.0);
    assert_eq!(stats.drive_damage_from_impacts, 0.0);
}

/// 1行動では説明できない大きな減少は、その行動の消費として帰属しない。
/// ガード削りなどが重なった区間を消費へ足すと過大評価になる。
#[test]
fn an_implausibly_large_drop_is_not_attributed() {
    let mut drive = vec![1.0_f32; 200];
    for value in drive.iter_mut().skip(52) {
        *value = 0.2;
    }
    let features = features_with_drive(&drive, None);
    let events = events_with_impact(impact(50, DriveImpactOutcome::Hit, 0.22));

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert_eq!(stats.own_di_used, 1);
    assert_eq!(stats.drive_spend_samples, 0);
}

/// 自分の DI の結末を、相手の DI を受けた側の集計と混ぜない。
#[test]
fn own_and_faced_drive_impacts_stay_separate() {
    let mut events = events_with_impact(impact(50, DriveImpactOutcome::Countered, 0.0));
    let mut faced = impact(120, DriveImpactOutcome::Hit, 0.3);
    faced.side = 2;
    events.drive_impacts.push(faced);
    let features = features_with_drive(&vec![1.0_f32; 200], None);

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert_eq!(stats.own_di_used, 1);
    assert_eq!(stats.own_di_countered, 1);
    assert_eq!(stats.own_di_hit, 0);
    assert_eq!(stats.di_faced, 1);
    assert_eq!(stats.di_hit, 1);
    assert_eq!(stats.di_returned, 0);
}

/// 生ラッシュの消費も同じ経路で実測する。DI だけ数えていると、ゲージの
/// 主要な使途がひとつ丸ごと収支から抜ける。
#[test]
fn a_raw_drive_rush_contributes_its_own_spend() {
    use crate::match_events::{DriveRushEvent, DriveRushOutcome};
    let mut drive = vec![1.0_f32; 200];
    for value in drive.iter_mut().skip(52) {
        *value = 0.5;
    }
    let features = features_with_drive(&drive, None);
    let mut events = events_with_impact(impact(9_999, DriveImpactOutcome::Hit, 0.0));
    events.drive_impacts.clear();
    events.drive_rushes = vec![DriveRushEvent {
        side: 1,
        frame: 50,
        raw: true,
        outcome: DriveRushOutcome::Hit,
        contact_frame: Some(62),
        damage: 0.3,
        confidence: EventConfidence::High,
        round_no: 1,
    }];

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert_eq!(stats.drive_spend_samples, 1);
    assert!((stats.drive_spent_on_rushes - 0.5).abs() < 0.005);
    assert!((stats.drive_damage_from_rushes - 0.3).abs() < 1e-6);
    assert_eq!(stats.drive_spent_on_impacts, 0.0);
}

/// 読み取り値そのものが壊れている frame は、uncertain でなくても使わない。
/// 範囲外や非有限を混ぜると、消費量が実測とかけ離れる。
#[test]
fn a_broken_gauge_reading_is_not_used_as_a_baseline() {
    let mut drive = vec![1.0_f32; 200];
    // 行動直前だけ壊れた値にする。uncertain flag は立っていない。
    drive[48] = f32::NAN;
    drive[49] = 5.0;
    drive[50] = -1.0;
    for value in drive.iter_mut().skip(52) {
        *value = 0.833;
    }
    let features = features_with_drive(&drive, None);
    let events = events_with_impact(impact(50, DriveImpactOutcome::Hit, 0.22));

    let stats = build_tactic_stats(&features, &events, 1, 2);

    // 壊れた値を無視しても、猶予内の健全な frame から基準を取れる。
    assert_eq!(stats.drive_spend_samples, 1);
    assert!((stats.drive_spent_on_impacts - 0.167).abs() < 0.005);
}

/// 行動より後の窓は行動フレームの次から始める。行動フレーム自体を含めると、
/// まだ減っていない値を「消費後」として読んでしまう。
#[test]
fn the_spend_window_starts_after_the_action_frame() {
    let mut drive = vec![1.0_f32; 200];
    // 行動フレームでは満タンのまま、次のフレームから減る。
    for value in drive.iter_mut().skip(51) {
        *value = 0.833;
    }
    let features = features_with_drive(&drive, None);
    let events = events_with_impact(impact(50, DriveImpactOutcome::Hit, 0.22));

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert_eq!(stats.drive_spend_samples, 1);
    assert!((stats.drive_spent_on_impacts - 0.167).abs() < 0.005);
}

/// 行動より前に読める frame が1つも無ければ基準を作れない。片側だけの
/// 欠測でも消費としては数えない。
#[test]
fn a_missing_baseline_alone_blocks_the_measurement() {
    let drive = vec![1.0_f32; 200];
    let mut features = features_with_drive(&drive, None);
    // 行動より前だけを読めなくする。後ろは読める。
    for feature in features.iter_mut().take(51) {
        feature.left_drive_uncertain = true;
    }
    let events = events_with_impact(impact(50, DriveImpactOutcome::Hit, 0.22));

    let stats = build_tactic_stats(&features, &events, 1, 2);

    assert_eq!(stats.drive_spend_samples, 0);
}

/// P2 を自分として解析するときは右側の Drive 系列を見る。P1 の満タン列を
/// 読んでも同じにならないよう、右側だけを減らす。
#[test]
fn player_two_spend_comes_from_the_right_drive_gauge() {
    let drive = vec![1.0_f32; 200];
    let mut features = features_with_drive(&drive, None);
    for feature in features.iter_mut().skip(52) {
        feature.right_drive_ratio = 0.75;
    }
    let mut own_impact = impact(50, DriveImpactOutcome::Hit, 0.22);
    own_impact.side = 2;
    let events = events_with_impact(own_impact);

    let stats = build_tactic_stats(&features, &events, 2, 1);

    assert_eq!(stats.drive_spend_samples, 1);
    assert!((stats.drive_spent_on_impacts - 0.25).abs() < 0.005);
}
