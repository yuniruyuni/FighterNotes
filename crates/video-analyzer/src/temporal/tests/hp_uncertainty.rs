use super::super::confirm_hp;
use super::super::hp::{test_backward_fill, test_expand_uncertain, test_obscure_neighbors};
use super::support::{assert_close, own_hp_series};

#[test]
fn forward_fill_fills_gaps_and_keeps_leading_unknown() {
    let values = [[-1.0; 3].as_slice(), &[0.8; 30], &[-1.0; 30], &[0.8; 30]].concat();
    let mut features = own_hp_series(&values);

    confirm_hp(&mut features);

    assert_eq!(features[0].own_hp, -1.0);
    assert_close(features[40].own_hp, 0.8);
}

#[test]
fn expand_uncertain_swallows_neighbors() {
    let mut values = vec![0.9; 100];
    values[50] = -1.0;
    values[45] = 0.2;
    let mut features = own_hp_series(&values);

    confirm_hp(&mut features);

    assert_close(features[45].own_hp, 0.9);
}

#[test]
fn expand_uncertain_clips_windows_at_sequence_edges() {
    assert_eq!(
        test_expand_uncertain(&[true, false, false, false, true], 1),
        vec![true, true, false, true, true]
    );
    assert!(test_expand_uncertain(&[], 10).is_empty());
}

#[test]
fn obscure_neighbors_marks_every_positive_value_in_the_expanded_window() {
    let source = vec![0.8, -1.0, 0.7];
    let mut values = vec![0.8, 0.6, 0.7];

    test_obscure_neighbors(&source, &mut values, 1);

    assert_eq!(values, vec![-1.0, -1.0, -1.0]);
}

#[test]
fn backward_fill_updates_only_originally_unknown_values() {
    let source = vec![0.5, -1.0, -1.0, 0.0];
    let mut values = vec![0.5, 0.5, 0.5, 0.0];

    test_backward_fill(&source, &mut values, 0);

    assert_eq!(values, vec![0.5, 0.0, 0.0, 0.0]);
}

#[test]
fn backward_fill_never_rewinds_originally_certain_frames() {
    let mut values = vec![0.5; 40];
    values.extend(vec![-1.0; 20]);
    values.extend(vec![0.0; 60]);
    let mut features = own_hp_series(&values);

    confirm_hp(&mut features);

    assert_close(features[20].own_hp, 0.5);
    assert_close(features[100].own_hp, 0.0);
    assert!(features[50].own_hp <= 0.5 + 1e-6);
}

#[test]
fn backward_fill_propagates_confirmed_ko_through_original_unknowns() {
    let mut values = vec![0.5; 30];
    values.extend(vec![-1.0; 5]);
    values.extend(vec![0.0; 40]);
    let mut features = own_hp_series(&values);

    confirm_hp(&mut features);

    assert_close(features[29].own_hp, 0.5);
    assert!(features[30..35].iter().all(|frame| frame.own_hp == 0.0));
}
