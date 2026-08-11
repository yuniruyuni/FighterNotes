use super::super::hp::{test_enforce_monotonic, test_round_reset_frames};
use super::super::{confirm_hp, confirm_hp_with_fight_markers, FULL_HP, FULL_MIN_RUN};
use super::support::{assert_close, hp_series, own_hp_series};
use crate::round_start::FightMarker;

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
fn stage_biased_full_hp_is_normalized_only_after_a_round_transition() {
    let mut features = hp_series(&[(0.2, 0.0); 40]);
    features.extend(hp_series(&[(-1.0, -1.0); 30]));
    features.extend(hp_series(&[(0.916, 1.0); 40]));
    features.extend(hp_series(&[(0.914, 1.0); 40]));
    features.extend(hp_series(&[(0.87, 1.0); 20]));
    reindex(&mut features);
    for frame in &mut features[40..70] {
        frame.is_match_screen = false;
        frame.left_drive_uncertain = true;
        frame.right_drive_uncertain = true;
    }
    for frame in &mut features[110..] {
        frame.left_drive_ratio = 0.5;
        frame.right_drive_ratio = 0.5;
    }

    confirm_hp(&mut features);

    assert!(features[80..150]
        .iter()
        .all(|frame| frame.own_hp == 1.0 && frame.opponent_hp == 1.0));
    assert_close(features[155].own_hp, 0.87);
}

#[test]
fn near_full_hp_inside_a_round_is_not_promoted() {
    let mut features = hp_series(&[(0.916, 1.0); 40]);

    confirm_hp(&mut features);

    assert_close(features[20].own_hp, 0.916);
}

#[test]
fn transition_without_material_hp_recovery_is_not_promoted() {
    let mut features = hp_series(&[(0.89, 1.0); 40]);
    features.extend(hp_series(&[(-1.0, -1.0); 30]));
    features.extend(hp_series(&[(0.916, 1.0); 40]));
    reindex(&mut features);
    for frame in &mut features[40..70] {
        frame.is_match_screen = false;
        frame.left_drive_uncertain = true;
        frame.right_drive_uncertain = true;
    }

    confirm_hp(&mut features);

    assert_close(features[90].own_hp, 0.89);
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

#[test]
fn fight_markers_are_the_only_resets_in_the_marker_aware_path() {
    let mut features = hp_series(&[(0.4, 0.5); 20]);
    features.extend(hp_series(&[(0.916, 1.0); 20]));
    features.extend(hp_series(&[(0.7, 0.8); 70]));
    features.extend(hp_series(&[(1.0, 1.0); 30]));
    features.extend(hp_series(&[(0.6, 0.7); 20]));
    features.extend(hp_series(&[(-1.0, -1.0); 20]));
    features.extend(hp_series(&[(0.91, 0.92); 20]));
    features.extend(hp_series(&[(0.5, 0.6); 70]));
    reindex(&mut features);
    for feature in &mut features[160..180] {
        feature.is_match_screen = false;
    }
    let markers = [
        FightMarker {
            first_frame: 20,
            last_frame: 36,
            peak_frame: 28,
            peak_score: 0.9,
        },
        FightMarker {
            first_frame: 180,
            last_frame: 196,
            peak_frame: 188,
            peak_score: 0.9,
        },
    ];

    confirm_hp_with_fight_markers(&mut features, &markers, "p1");

    assert_close(features[20].own_hp, 1.0);
    assert_close(features[37].own_hp, 1.0);
    assert_close(features[45].own_hp, 0.7);
    // HP だけが満タンへ戻っても、FIGHT marker が無ければ reset しない。
    assert_close(features[125].own_hp, 0.7);
    assert_close(features[180].own_hp, 1.0);
    assert_close(features[205].own_hp, 0.5);
}

#[test]
fn fight_opening_uses_reliable_raw_hp_instead_of_previous_round_fill() {
    let mut features = hp_series(&[(0.4, 0.1); 20]);
    features.extend(hp_series(&[(1.0, -1.0); 25]));
    features.extend(hp_series(&[(1.0, 1.0); 15]));
    features.extend(hp_series(&[(1.0, 0.94); 20]));
    reindex(&mut features);
    for feature in &mut features[20..45] {
        feature.right_hp_raw = 0.0;
        feature.right_hp_raw_quality = 1.0;
    }
    let markers = [FightMarker {
        first_frame: 20,
        last_frame: 52,
        peak_frame: 36,
        peak_score: 0.9,
    }];

    confirm_hp_with_fight_markers(&mut features, &markers, "p1");

    assert_close(features[52].opponent_hp, 1.0);
    assert_close(features[60].opponent_hp, 0.94);
}

#[test]
fn p2_uses_the_right_raw_bar_as_its_own_opening_evidence() {
    let mut features = hp_series(&[(0.89, 0.50); 50]);
    reindex(&mut features);
    for (index, feature) in features.iter_mut().enumerate() {
        feature.left_hp_raw = if (10..=20).contains(&index) {
            1.0
        } else {
            0.50
        };
        feature.right_hp_raw = if (10..=20).contains(&index) {
            0.90
        } else {
            0.89
        };
        feature.left_hp_raw_quality = 0.0;
        feature.right_hp_raw_quality = 0.0;
    }
    let markers = [FightMarker {
        first_frame: 10,
        last_frame: 20,
        peak_frame: 15,
        peak_score: 0.9,
    }];

    confirm_hp_with_fight_markers(&mut features, &markers, "p2");

    assert_close(features[25].own_hp, 1.0);
}

#[test]
fn recoverable_hp_is_restored_for_the_opponent_too() {
    let mut opponent = vec![1.0; 40];
    opponent.extend(vec![0.96; 100]);
    for step in 1..=20 {
        opponent.extend(vec![0.96 + step as f32 * 0.0015; 6]);
    }
    opponent.extend(vec![0.991; 12]);
    opponent.extend(vec![0.72; 40]);
    let pairs: Vec<_> = opponent.iter().map(|&value| (1.0, value)).collect();
    let mut features = hp_series(&pairs);
    for feature in &mut features[40..] {
        feature.own_hp = 0.875;
        feature.left_hp_raw = 0.875;
    }
    for feature in &mut features[246..260] {
        feature.opponent_hp = -1.0;
        feature.right_hp_raw = 0.0;
        feature.right_hp_raw_quality = 1.0;
    }

    confirm_hp(&mut features);

    assert!(features[40..272]
        .iter()
        .all(|feature| feature.opponent_hp == 1.0));
    assert_close(features[272].opponent_hp, 0.72);
}

#[test]
fn sustained_stepwise_recovery_is_not_committed_as_permanent_damage() {
    let mut values = vec![1.0; 40];
    values.extend(vec![0.96; 100]);
    for step in 1..=20 {
        values.extend(vec![0.96 + step as f32 * 0.0015; 6]);
    }
    values.extend(vec![0.991; 12]);
    values.extend(vec![0.72; 40]);
    let mut features = own_hp_series(&values);
    for feature in &mut features[40..] {
        feature.opponent_hp = 0.875;
        feature.right_hp_raw = 0.875;
    }
    for feature in &mut features[246..260] {
        feature.own_hp = -1.0;
        feature.left_hp_raw = 0.0;
        feature.left_hp_raw_quality = 1.0;
    }

    confirm_hp(&mut features);

    assert!(features[40..272]
        .iter()
        .all(|feature| feature.own_hp == 1.0));
    assert_close(features[272].own_hp, 0.72);
}

#[test]
fn partial_or_single_step_rebounds_remain_monotonic() {
    let mut partial = own_hp_series(
        &vec![1.0; 40]
            .into_iter()
            .chain(vec![0.70; 80])
            .chain(vec![0.75; 80])
            .collect::<Vec<_>>(),
    );
    for feature in &mut partial[40..] {
        feature.opponent_hp = 0.8;
        feature.right_hp_raw = 0.8;
    }
    confirm_hp(&mut partial);
    assert_close(partial.last().unwrap().own_hp, 0.70);

    let mut spike = own_hp_series(
        &vec![1.0; 40]
            .into_iter()
            .chain(vec![0.70; 80])
            .chain(vec![0.99; 20])
            .collect::<Vec<_>>(),
    );
    for feature in &mut spike[40..] {
        feature.opponent_hp = 0.8;
        feature.right_hp_raw = 0.8;
    }
    confirm_hp(&mut spike);
    assert_close(spike.last().unwrap().own_hp, 0.70);
}

fn reindex(features: &mut [crate::frame_features::FrameFeatures]) {
    for (index, frame) in features.iter_mut().enumerate() {
        frame.frame_index = index as u32;
    }
}
