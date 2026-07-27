use frame_meter::RowObs;

use crate::calibration::{LABEL_DECIDE_MARGIN, LABEL_DIGIT_MIN};
use crate::tracker::reading::label::{
    decide_label, digit_layout, has_label_evidence, label_layout_score,
};

use super::{digit_correlations, insert_read, tracker_at};

fn tracker_with_run(absolute: i64, visible: i64) -> super::MeterTracker {
    let mut tracker = tracker_at(absolute);
    for back in 1..=visible {
        insert_read(&mut tracker, "left", absolute - back, "stun", 1.0, false);
    }
    tracker
}

#[test]
fn label_evidence_can_choose_current_run_or_empty_layout() {
    let cell = 10;
    let tracker = tracker_with_run(20, 3);
    let mut observation = RowObs::empty();
    let mut correlations = digit_correlations();
    correlations[cell as usize][4] = 0.6;
    observation.digit_corr = Some(correlations);
    assert_eq!(
        tracker.resolve_slab(&observation, None, cell, "left"),
        "stun"
    );

    let tracker = tracker_with_run(20, 4);
    let mut correlations = digit_correlations();
    correlations[(cell - 1) as usize][4] = 0.6;
    observation.digit_corr = Some(correlations);
    assert_eq!(
        tracker.resolve_slab(&observation, None, cell, "left"),
        "empty"
    );
}

#[test]
fn label_decision_requires_correlations_valid_run_length_and_evidence() {
    let cell = 10;
    let tracker = tracker_with_run(20, 3);
    assert_eq!(
        tracker.resolve_slab(&RowObs::empty(), None, cell, "left"),
        "other"
    );

    let mut observation = RowObs::empty();
    observation.digit_corr = Some(digit_correlations());
    let short = tracker_with_run(20, 2);
    assert_eq!(
        short.resolve_slab(&observation, None, cell, "left"),
        "other"
    );
    assert_eq!(
        tracker.resolve_slab(&observation, None, cell, "left"),
        "other"
    );

    let too_long = tracker_with_run(100, 99);
    let mut correlations = digit_correlations();
    correlations[(cell - 2) as usize][1] = 0.6;
    correlations[(cell - 1) as usize][0] = 0.6;
    correlations[cell as usize][0] = 0.6;
    observation.digit_corr = Some(correlations);
    assert_eq!(
        too_long.resolve_slab(&observation, None, cell, "left"),
        "other"
    );
}

#[test]
fn digit_layout_wraps_multi_digit_positions() {
    assert_eq!(
        digit_layout("10", 0),
        [(79, '1'), (0, '0')].into_iter().collect()
    );
    assert_eq!(
        digit_layout("123", 10),
        [(8, '1'), (9, '2'), (10, '3')].into_iter().collect()
    );
}

#[test]
fn layout_score_distinguishes_digits_and_blank_maxima() {
    let mut correlations = digit_correlations();
    correlations[8][4] = 0.7;
    correlations[9][2] = 0.2;
    correlations[10][7] = 0.9;
    let score = label_layout_score(&correlations, &[(8, '4')].into_iter().collect(), 10);
    assert!((score - 0.15).abs() < 1e-6, "{score}");

    let positions = [(8, '4')].into_iter().collect();
    correlations[8][4] = LABEL_DIGIT_MIN as f32;
    assert!(has_label_evidence(&correlations, &positions));
    correlations[8][4] = LABEL_DIGIT_MIN as f32 - 0.01;
    assert!(!has_label_evidence(&correlations, &positions));
}

#[test]
fn label_decision_requires_margin_and_matching_evidence() {
    assert_eq!(
        decide_label("stun".to_string(), LABEL_DECIDE_MARGIN, true, 0.0, false),
        Some("stun".to_string())
    );
    assert_eq!(
        decide_label("stun".to_string(), 0.2, false, 0.0, false),
        None
    );
    assert_eq!(
        decide_label("stun".to_string(), 0.1, true, -0.1, false),
        Some("stun".to_string())
    );
    assert_eq!(
        decide_label("stun".to_string(), 0.0, false, LABEL_DECIDE_MARGIN, true),
        Some("empty".to_string())
    );
    assert_eq!(
        decide_label("stun".to_string(), 0.0, false, 0.2, false),
        None
    );
    assert_eq!(
        decide_label("stun".to_string(), -0.1, false, 0.1, true),
        Some("empty".to_string())
    );
}
