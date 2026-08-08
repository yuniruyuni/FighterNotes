use super::super::confirm_hp;
use super::super::hp::{test_reject_hp_crashes, TEST_CRASH_CONFIRM};
use super::support::{assert_close, hp_series, own_hp_series};

#[test]
fn crash_bounce_is_rejected() {
    let mut values = vec![1.0; 40];
    values.extend(vec![0.05; 10]);
    values.extend(vec![0.9; 100]);
    let mut features = own_hp_series(&values);

    confirm_hp(&mut features);

    assert_close(features[45].own_hp, 1.0);
    assert_close(features[60].own_hp, 0.9);
}

#[test]
fn crash_sustained_is_accepted_as_ko() {
    let mut values = vec![0.5; 40];
    values.extend(vec![0.02; 100]);
    let mut features = own_hp_series(&values);

    confirm_hp(&mut features);

    assert_close(features[80].own_hp, 0.02);
}

#[test]
fn ordinary_damage_below_crash_threshold_is_kept() {
    let mut values = vec![1.0; 40];
    values.extend(vec![0.89; 100]);
    let mut features = own_hp_series(&values);

    confirm_hp(&mut features);

    assert_close(features[80].own_hp, 0.89);
}

#[test]
fn crash_threshold_is_strict() {
    let mut values = vec![1.0, 1.0 - 0.12];
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert_close(values[1], 1.0 - 0.12);
}

#[test]
fn short_zero_hp_reading_is_rejected_as_a_crash() {
    let mut values = vec![1.0, 0.0];
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert_eq!(values, vec![1.0, 1.0]);
}

#[test]
fn crash_correction_stops_at_an_unknown_reading() {
    let mut values = vec![1.0, 0.05, -1.0, 0.9];
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert_eq!(values, vec![1.0, 1.0, -1.0, 0.9]);
}

#[test]
fn crash_correction_includes_the_exact_step_boundary() {
    let mut values = vec![1.0, 0.05, 1.0 - 0.12, 0.9];
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert_eq!(values, vec![1.0, 1.0, 1.0, 0.9]);
}

#[test]
fn bounce_threshold_is_strict() {
    let mut values = vec![1.0, 0.05];
    values.extend(vec![0.15; TEST_CRASH_CONFIRM]);
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert_close(values[1], 0.05);
}

#[test]
fn recovery_after_confirmation_does_not_reject_a_crash() {
    let mut values = vec![1.0];
    values.extend(vec![0.05; TEST_CRASH_CONFIRM]);
    values.push(0.9);
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert!(values[1..=TEST_CRASH_CONFIRM]
        .iter()
        .all(|&value| value == 0.05));
}

#[test]
fn ordinary_frame_does_not_hide_a_later_bouncing_crash() {
    let mut values = vec![1.0, 0.95];
    values.extend(vec![0.05; 5]);
    values.extend(vec![0.9; 10]);
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert!(values[2..7].iter().all(|&value| value == 0.95));
}

#[test]
fn short_crash_at_clip_end_is_rejected() {
    let mut values = vec![1.0; 40];
    values.extend(vec![0.05; TEST_CRASH_CONFIRM / 2 - 1]);
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert!(values[40..].iter().all(|&value| value == 1.0));
}

#[test]
fn half_confirmation_at_clip_end_accepts_sustained_crash() {
    let mut values = vec![1.0; 40];
    values.extend(vec![0.05; TEST_CRASH_CONFIRM / 2]);
    let match_frames = vec![true; values.len()];

    test_reject_hp_crashes(&mut values, &match_frames);

    assert!(values[40..].iter().all(|&value| value == 0.05));
}

#[test]
fn crash_confirmation_counts_only_match_frames() {
    let mut values = vec![1.0];
    values.extend(vec![0.05; TEST_CRASH_CONFIRM]);
    let mut match_frames = vec![true; values.len()];
    match_frames[1..=TEST_CRASH_CONFIRM - 20].fill(false);

    test_reject_hp_crashes(&mut values, &match_frames);

    assert!(values[1..].iter().all(|&value| value == 1.0));
}

#[test]
fn hp_cleanup_processes_opponent_independently() {
    let mut values = vec![(0.8, 1.0); 40];
    values.extend(vec![(0.8, 0.05); 10]);
    values.extend(vec![(0.8, 0.9); 100]);
    let mut features = hp_series(&values);

    confirm_hp(&mut features);

    assert!(features.iter().all(|frame| frame.own_hp == 0.8));
    assert_close(features[45].opponent_hp, 1.0);
    assert_close(features[60].opponent_hp, 0.9);
}
