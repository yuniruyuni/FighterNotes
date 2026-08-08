use super::super::clean_drive_temporal;
use super::support::{assert_close, drive_series};

#[test]
fn drive_cleanup_processes_right_side_independently() {
    let mut features = drive_series(&[(0.8, false, false); 44]);
    for frame in &mut features {
        frame.right_drive_ratio = 1.0;
    }
    for frame in &mut features[20..24] {
        frame.right_drive_ratio = 0.5;
    }

    clean_drive_temporal(&mut features);

    assert!(features.iter().all(|frame| !frame.left_drive_uncertain));
    assert!(features.iter().all(|frame| frame.left_drive_ratio == 0.8));
    assert!(features[20..24]
        .iter()
        .all(|frame| frame.right_drive_uncertain));
    assert!(features[20..24]
        .iter()
        .all(|frame| frame.right_drive_ratio == 1.0));
}

#[test]
fn leading_uncertain_drive_has_no_value_to_fill_from() {
    let mut values = vec![(0.2, true, true); 4];
    values.extend(vec![(0.8, false, false); 8]);
    let mut features = drive_series(&values);

    clean_drive_temporal(&mut features);

    assert!(features[..4].iter().all(|frame| frame.left_drive_uncertain));
    assert!(features[..4]
        .iter()
        .all(|frame| frame.left_drive_ratio == 0.2));
    assert!(features[..4].iter().all(|frame| frame.left_burnout));
}

#[test]
fn leading_uncertain_frame_does_not_stabilize_a_short_run() {
    let mut values = vec![(0.8, false, true)];
    values.extend(vec![(0.8, false, false); 7]);
    let mut features = drive_series(&values);

    clean_drive_temporal(&mut features);

    assert!(features[1..].iter().all(|frame| frame.left_drive_uncertain));
}

#[test]
fn leading_non_match_frame_does_not_stabilize_a_short_run() {
    let mut features = drive_series(&[(0.8, false, false); 8]);
    features[0].is_match_screen = false;

    clean_drive_temporal(&mut features);

    assert!(features[1..].iter().all(|frame| frame.left_drive_uncertain));
}

#[test]
fn drive_hold_resets_at_non_match_gap() {
    let mut features = drive_series(&[(1.0, false, false); 20]);
    for frame in &mut features[8..12] {
        frame.is_match_screen = false;
    }
    features[12].left_drive_uncertain = true;

    clean_drive_temporal(&mut features);

    assert!(features[12].left_drive_uncertain);
}

#[test]
fn non_match_gap_prevents_drive_value_from_crossing_rounds() {
    let mut values = vec![(0.8, false, false); 8];
    values.push((0.4, false, false));
    values.push((0.2, true, true));
    values.extend(vec![(0.6, false, false); 8]);
    let mut features = drive_series(&values);
    features[8].is_match_screen = false;

    clean_drive_temporal(&mut features);

    assert_close(features[9].left_drive_ratio, 0.2);
    assert!(features[9].left_burnout);
    assert!(features[9].left_drive_uncertain);
    assert_close(features[10].left_drive_ratio, 0.6);
    assert!(!features[10].left_drive_uncertain);
}
