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
