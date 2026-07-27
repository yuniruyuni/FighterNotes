use super::super::hp::{test_enforce_monotonic, test_round_reset_frames};
use super::super::{confirm_hp, FULL_HP, FULL_MIN_RUN};
use super::support::{assert_close, hp_series, own_hp_series};

#[test]
fn monotonic_within_round_resets_on_full_run() {
    let mut values = vec![0.4; 40];
    values.extend(vec![0.7; 30]);
    values.extend(vec![1.0; 40]);
    let mut features = own_hp_series(&values);

    confirm_hp(&mut features);

    assert_close(features[50].own_hp, 0.4);
    assert_close(features[100].own_hp, 1.0);
}

#[test]
fn full_hp_reset_requires_both_players_and_minimum_run() {
    let mut one_player_full = hp_series(&[(0.4, 0.6); 40]);
    one_player_full.extend(hp_series(&[(1.0, 0.9); FULL_MIN_RUN + 5]));
    reindex(&mut one_player_full);
    confirm_hp(&mut one_player_full);
    assert_close(one_player_full.last().unwrap().own_hp, 0.4);

    let mut too_short = hp_series(&[(0.4, 0.6); 40]);
    too_short.extend(hp_series(&[(1.0, 1.0); FULL_MIN_RUN - 1]));
    reindex(&mut too_short);
    confirm_hp(&mut too_short);
    assert_close(too_short.last().unwrap().own_hp, 0.4);
    assert_close(too_short.last().unwrap().opponent_hp, 0.6);

    let mut exact_run = hp_series(&[(0.4, 0.6); 40]);
    exact_run.extend(hp_series(&[(1.0, 1.0); FULL_MIN_RUN]));
    reindex(&mut exact_run);
    confirm_hp(&mut exact_run);
    assert_close(exact_run[40].own_hp, 1.0);
    assert_close(exact_run[40].opponent_hp, 1.0);
}

#[test]
fn reset_detection_marks_only_the_start_of_each_complete_run() {
    let mut own = vec![0.5];
    own.extend(vec![FULL_HP; FULL_MIN_RUN]);
    own.push(0.5);
    own.extend(vec![1.0; FULL_MIN_RUN - 1]);
    let opponent = own.clone();
    let match_frames = vec![true; own.len()];

    let reset_at = test_round_reset_frames(&own, &opponent, &match_frames);

    assert_eq!(reset_at.iter().filter(|&&reset| reset).count(), 1);
    assert!(reset_at[1]);
}

#[test]
fn reset_detection_requires_a_match_frame_at_the_full_hp_boundary() {
    let own = vec![FULL_HP; FULL_MIN_RUN];
    let opponent = own.clone();
    let mut match_frames = vec![true; own.len()];
    match_frames[0] = false;

    let reset_at = test_round_reset_frames(&own, &opponent, &match_frames);

    assert!(reset_at.iter().all(|&reset| !reset));
}

#[test]
fn monotonic_enforcement_preserves_unknowns_and_commits_zero_hp() {
    let mut values = vec![0.8, -1.0, 0.9, 0.0, 0.4];
    let reset_at = vec![false; values.len()];

    test_enforce_monotonic(&mut values, &reset_at);

    assert_eq!(values, vec![0.8, -1.0, 0.8, 0.0, 0.0]);
}

#[test]
fn monotonic_enforcement_allows_an_increase_at_reset() {
    let mut values = vec![0.4, 1.0];

    test_enforce_monotonic(&mut values, &[false, true]);

    assert_eq!(values, vec![0.4, 1.0]);
}

fn reindex(features: &mut [crate::frame_features::FrameFeatures]) {
    for (index, frame) in features.iter_mut().enumerate() {
        frame.frame_index = index as u32;
    }
}
