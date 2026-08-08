use super::super::clean_drive_temporal;
use super::support::{assert_close, drive_series};

#[test]
fn short_unstable_drive_segment_is_rejected_and_filled() {
    let mut values = vec![(1.0, false, false); 30];
    values.extend(vec![(0.5, false, false); 4]);
    values.extend(vec![(1.0, false, false); 30]);
    let mut features = drive_series(&values);

    clean_drive_temporal(&mut features);

    assert!(features[31].left_drive_uncertain);
    assert_close(features[31].left_drive_ratio, 1.0);
    assert!(!features[40].left_drive_uncertain);
}

#[test]
fn long_stable_drive_segment_is_kept() {
    let mut features = drive_series(&[(0.5, false, false); 20]);

    clean_drive_temporal(&mut features);

    assert!(features.iter().all(|frame| !frame.left_drive_uncertain));
    assert!(features.iter().all(|frame| frame.left_drive_ratio == 0.5));
}

#[test]
fn drive_stable_segment_boundary_is_eight_frames() {
    let mut seven = drive_series(&[(0.5, false, false); 7]);
    clean_drive_temporal(&mut seven);
    assert!(seven.iter().all(|frame| frame.left_drive_uncertain));

    let mut eight = drive_series(&[(0.5, false, false); 8]);
    clean_drive_temporal(&mut eight);
    assert!(eight.iter().all(|frame| !frame.left_drive_uncertain));
}

#[test]
fn gradual_drive_change_within_step_limit_is_kept() {
    let values: Vec<_> = (0..12)
        .map(|index| (0.9 - index as f32 * 0.04, false, false))
        .collect();
    let mut features = drive_series(&values);

    clean_drive_temporal(&mut features);

    assert!(features.iter().all(|frame| !frame.left_drive_uncertain));
    assert_close(features[11].left_drive_ratio, 0.46);
}

#[test]
fn exact_drive_step_limit_stays_in_the_same_segment() {
    let mut values = vec![(0.25 / 6.0, false, false); 7];
    values.push((0.0, false, false));
    let mut features = drive_series(&values);

    clean_drive_temporal(&mut features);

    assert!(features.iter().all(|frame| !frame.left_drive_uncertain));
}

#[test]
fn uncertain_frame_terminates_the_current_segment() {
    let mut values = vec![(0.8, false, false); 8];
    values.push((0.8, false, true));
    values.extend(vec![(0.8, false, false); 7]);
    let mut features = drive_series(&values);

    clean_drive_temporal(&mut features);

    assert!(features[9..].iter().all(|frame| frame.left_drive_uncertain));
}

#[test]
fn non_match_frame_terminates_the_current_segment() {
    let mut features = drive_series(&[(0.8, false, false); 16]);
    features[8].is_match_screen = false;

    clean_drive_temporal(&mut features);

    assert!(features[9..].iter().all(|frame| frame.left_drive_uncertain));
}

#[test]
fn large_drive_step_rejects_short_tail() {
    let mut values = vec![(0.8, false, false); 8];
    values.extend(vec![(0.7, false, false); 4]);
    let mut features = drive_series(&values);

    clean_drive_temporal(&mut features);

    assert!(features[8..].iter().all(|frame| frame.left_drive_uncertain));
    assert!(features[8..]
        .iter()
        .all(|frame| frame.left_drive_ratio == 0.8));
}

#[test]
fn burnout_flag_change_splits_segment() {
    let mut values = vec![(0.5, false, false); 30];
    values.extend(vec![(0.5, true, false); 3]);
    values.extend(vec![(0.5, false, false); 30]);
    let mut features = drive_series(&values);

    clean_drive_temporal(&mut features);

    assert!(features[31].left_drive_uncertain);
    assert!(!features[31].left_burnout);
    assert!(!features[10].left_drive_uncertain);
    assert!(!features[40].left_drive_uncertain);
}
