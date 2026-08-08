use super::support::*;

#[test]
fn test_round_split_and_winner() {
    let fs = synth_two_rounds();
    let ev = build_match_events(&fs, &[], &[], None, "p1");
    assert_eq!(
        ev.rounds.len(),
        2,
        "2 ラウンドに分割されるべき: {:?}",
        ev.rounds
    );
    assert_eq!(ev.rounds[0].winner, Some(1));
    assert_eq!(ev.rounds[1].winner, Some(2));
}

#[test]
fn obscured_hp_tail_keeps_the_rest_of_the_continuous_match_hud() {
    let mut features = Vec::new();
    for frame in 0..30 {
        features.push(feat(frame, 1.0, 1.0));
    }
    for frame in 30..100 {
        features.push(feat(frame, 0.8, 0.7));
    }
    for frame in 100..180 {
        let p2_hp = 0.7 - 0.5 * (frame - 99) as f32 / 80.0;
        let mut feature = feat(frame, 0.8, p2_hp);
        feature.left_hp_raw = 0.0;
        feature.left_hp_raw_quality = 1.0;
        features.push(feature);
    }
    for frame in 180..230 {
        let mut feature = feat(frame, 0.8, 0.2);
        feature.is_match_screen = false;
        features.push(feature);
    }
    for frame in 230..300 {
        features.push(feat(frame, 1.0, 1.0));
    }

    let events = build_match_events(&features, &[], &[], None, "p1");
    let first = &events.rounds[0];

    assert_eq!(first.end_frame, 179);
    assert_eq!(first.winner, Some(1));
    assert!((first.p2_hp_end - 0.2).abs() < 0.01);
    assert!(
        events
            .damage
            .iter()
            .any(|damage| damage.round_no == 1 && damage.end_frame >= 170),
        "遮蔽後の HP 低下をラウンド外へ落とさない"
    );
}
