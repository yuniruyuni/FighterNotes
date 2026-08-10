//! 守勢だったと言えるかどうかの判断に対するテスト。
//!
//! メーターが読めている試合では、直前に削られたかガードしたかで決まる。
//! 読めていない試合では代わりにドライブゲージの減りを見る。固められて
//! いればガードで削られるので、短時間で大きく減る。
//!
//! ここが緩むと、地上戦の読み合いで押したボタンまで「守勢での暴れ」に
//! なる。厳しすぎると、実際に固められている場面が拾えない。

use super::support::*;
use crate::match_events::{DamageEvent, InputSegment, MatchEvents};
use match_event_layer::test_support::feat;

fn press(start_frame: u32) -> InputSegment {
    InputSegment {
        start_frame,
        end_frame: start_frame + 5,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }
}

fn big_hit(start_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 20,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

/// メーターを読めていない試合で、押して被弾した一場面。
fn one_mash_without_a_meter() -> MatchEvents {
    let mut events = empty_events();
    events.damage = vec![big_hit(1000, 0.12)];
    events.segments[0] = vec![press(990)];
    events
}

/// ドライブゲージの推移。`spent` の分だけ、被弾までの間に減る。
fn drive_dropping_by(spent: f32) -> Vec<crate::FrameFeatures> {
    (0..1200u32)
        .map(|frame| {
            let mut feature = feat(frame, 1.0, 1.0);
            let ratio = if frame < 880 {
                1.0
            } else {
                1.0 - spent * (frame - 880) as f32 / 120.0
            };
            feature.left_drive_ratio = ratio.max(0.0);
            feature
        })
        .collect()
}

/// ゲージが大きく減っていれば、固められていたとみなす。少し減った
/// だけなら、自分から使っただけかもしれない。
#[test]
fn a_large_drive_drop_stands_in_for_being_pressured() {
    let events = one_mash_without_a_meter();

    assert!(
        detect_mashing(&drive_dropping_by(0.20), &events, 1, 0).is_some(),
        "大きな消費を見ていない"
    );
    assert!(
        detect_mashing(&drive_dropping_by(0.02), &events, 1, 0).is_none(),
        "わずかな消費で守勢と見なしている"
    );
}

/// 観測が無ければ判断しない。
#[test]
fn without_any_frame_features_nothing_is_assumed() {
    assert!(detect_mashing(&[], &one_mash_without_a_meter(), 1, 0).is_none());
}

/// バーンアウト中の推移は残量ではなく回復の進み具合。減ったように
/// 見えても、固められた証拠にはならない。
#[test]
fn a_burnout_recovery_is_not_a_drive_spend() {
    let mut features = drive_dropping_by(0.20);
    for feature in &mut features {
        feature.left_burnout = true;
    }

    assert!(detect_mashing(&features, &one_mash_without_a_meter(), 1, 0).is_none());
}

/// 見る側を取り違えない。相手のゲージが減っていても、自分が固められて
/// いたことにはならない。
#[test]
fn the_opponents_drive_spend_is_not_yours() {
    let features = drive_dropping_by(0.20);

    assert!(
        detect_mashing(&features, &one_mash_without_a_meter(), 1, 0).is_some(),
        "自分側のゲージを見ていない"
    );
    assert!(
        detect_mashing(&features, &one_mash_without_a_meter(), 2, 1).is_none(),
        "相手側のゲージで自分を守勢にしている"
    );
}

/// 見るのは被弾までの一定時間だけ。ラウンド全体で見ると、攻めのために
/// 使った分まで「固められた」に数える。
#[test]
fn only_the_drive_spent_shortly_before_the_hit_counts() {
    let recent: Vec<_> = (0..1200u32)
        .map(|frame| {
            let mut feature = feat(frame, 1.0, 1.0);
            feature.left_drive_ratio = if frame < 900 { 1.0 } else { 0.90 };
            feature
        })
        .collect();
    let long_ago: Vec<_> = (0..1200u32)
        .map(|frame| {
            let mut feature = feat(frame, 1.0, 1.0);
            feature.left_drive_ratio = if frame < 500 { 1.0 } else { 0.90 };
            feature
        })
        .collect();

    assert!(
        detect_mashing(&recent, &one_mash_without_a_meter(), 1, 0).is_some(),
        "直前の消費を見ていない"
    );
    assert!(
        detect_mashing(&long_ago, &one_mash_without_a_meter(), 1, 0).is_none(),
        "ずっと前の消費で守勢にしている"
    );
}
