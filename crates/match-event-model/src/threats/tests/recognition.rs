use super::super::*;
use super::support::*;

#[test]
fn inv_active_is_not_called_teleport_without_character_profile() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let p1 = timeline("left", &[]);
    let p2 = timeline("right", &[(170, 176, "inv_full"), (190, 195, "active")]);
    let meter = [state_per_frame(&p1, 400), state_per_frame(&p2, 400)];
    let (_, teleports, _) = extract_test_threats!(
        &features,
        [&p1, &p2],
        &meter,
        &[vec![], vec![teleport_segment(160)]],
        &[],
        &[],
        &[],
        &[round()],
        [Some("BLANKA"), Some("KEN")],
    );
    assert!(teleports.is_empty());
}

#[test]
fn held_projectile_cell_is_not_carried_as_a_projectile() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let empty = timeline("left", &[]);
    let held = MeterTimeline {
        side: "right".to_string(),
        segments: vec![meter_tracker::TimelineSegment {
            segment_id: 0,
            entries: vec![meter_tracker::TimelineEntry {
                game_frame: 14,
                state: "projectile_active".to_string(),
                video_frame_first: 100,
                video_frame_last: 150,
                confidence: 1.0,
            }],
        }],
    };
    let meter = [state_per_frame(&empty, 400), state_per_frame(&held, 400)];

    let (projectiles, _, _) = extract_test_threats!(
        &features,
        [&empty, &held],
        &meter,
        &[vec![], vec![]],
        &[],
        &[],
        &[],
        &[round()],
        [Some("BLANKA"), Some("DHALSIM")],
    );

    assert!(projectiles.is_empty());
}

#[test]
fn mixed_attack_or_throw_chords_are_not_teleports() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let free = timeline("left", &[]);
    let dhalsim = timeline("right", &[(170, 176, "inv_full"), (190, 195, "active")]);
    let meter = [state_per_frame(&free, 400), state_per_frame(&dhalsim, 400)];
    let mut mixed = teleport_segment(160);
    mixed.badges = vec!["中P".to_string(), "中K".to_string()];
    let mut throw = teleport_segment(160);
    throw.badges = vec!["弱P".to_string(), "弱K".to_string()];
    throw.throw = true;

    for segment in [mixed, throw] {
        let (_, teleports, _) = extract_test_threats!(
            &features,
            [&free, &dhalsim],
            &meter,
            &[vec![], vec![segment]],
            &[],
            &[],
            &[],
            &[round()],
            [Some("BLANKA"), Some("DHALSIM")],
        );
        assert!(teleports.is_empty());
    }
}

#[test]
fn long_invincibility_run_is_not_a_teleport() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let free = timeline("left", &[]);
    let super_move = timeline("right", &[(170, 200, "inv_full"), (210, 215, "active")]);
    let meter = [
        state_per_frame(&free, 400),
        state_per_frame(&super_move, 400),
    ];
    let (_, teleports, _) = extract_test_threats!(
        &features,
        [&free, &super_move],
        &meter,
        &[vec![], vec![teleport_segment(160)]],
        &[],
        &[],
        &[],
        &[round()],
        [Some("BLANKA"), Some("DHALSIM")],
    );
    assert!(teleports.is_empty());
}

#[test]
fn a_projectile_without_a_timely_response_keeps_its_carry_window() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let projectile = timeline("left", &[(100, 150, "projectile_active")]);
    // This response is well before the projectile run ends. Starting the response
    // search at frame zero would incorrectly attach it to this projectile.
    let early_response = timeline("right", &[(100, 104, "parry"), (105, 107, "stun")]);
    let meter = [
        state_per_frame(&projectile, 400),
        state_per_frame(&early_response, 400),
    ];

    let (projectiles, _, _) = extract_test_threats!(
        &features,
        [&projectile, &early_response],
        &meter,
        &[vec![], vec![]],
        &[],
        &[],
        &[],
        &[round()],
        [None, None],
    );

    assert_eq!(projectiles.len(), 1);
    assert_eq!(projectiles[0].contact_frame, None);
    assert_eq!(
        projectiles[0].threat_end_frame,
        150 + PROJECTILE_CARRY_WINDOW
    );
}

#[test]
fn an_out_of_round_projectile_does_not_hide_a_later_valid_one() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let p1 = timeline(
        "left",
        &[
            (10, 20, "projectile_active"),
            (100, 110, "projectile_active"),
        ],
    );
    let p2 = timeline("right", &[]);
    let meter = [state_per_frame(&p1, 400), state_per_frame(&p2, 400)];
    let valid_round = RoundInfo {
        start_frame: 50,
        ..round()
    };

    let (projectiles, _, _) = extract_test_threats!(
        &features,
        [&p1, &p2],
        &meter,
        &[vec![], vec![]],
        &[],
        &[],
        &[],
        &[valid_round],
        [None, None],
    );

    assert_eq!(projectiles.len(), 1);
    assert_eq!(projectiles[0].observed_start_frame, 100);
}

#[test]
fn invalid_invincibility_candidates_do_not_hide_a_later_teleport() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let p1 = timeline("left", &[]);
    let p2 = timeline(
        "right",
        &[
            // Valid shape but outside the confirmed round.
            (20, 26, "inv_full"),
            // A cinematic-sized invincibility run.
            (50, 80, "inv_full"),
            // Short enough, but without a matching chord.
            (100, 106, "inv_full"),
            // The valid candidate that must still be reached.
            (170, 176, "inv_full"),
            (190, 195, "active"),
        ],
    );
    let meter = [state_per_frame(&p1, 400), state_per_frame(&p2, 400)];
    let valid_round = RoundInfo {
        start_frame: 80,
        ..round()
    };

    let (_, teleports, _) = extract_test_threats!(
        &features,
        [&p1, &p2],
        &meter,
        &[
            vec![],
            vec![
                teleport_segment(10),
                teleport_segment(40),
                teleport_segment(160),
            ],
        ],
        &[],
        &[],
        &[],
        &[valid_round],
        [Some("BLANKA"), Some("DHALSIM")],
    );

    assert_eq!(teleports.len(), 1);
    assert_eq!(teleports[0].inv_start_frame, 170);
}

#[test]
fn invincibility_at_the_teleport_limit_is_accepted() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let p1 = timeline("left", &[]);
    let p2 = timeline("right", &[(170, 181, "inv_full")]);
    let meter = [state_per_frame(&p1, 400), state_per_frame(&p2, 400)];

    let (_, teleports, _) = extract_test_threats!(
        &features,
        [&p1, &p2],
        &meter,
        &[vec![], vec![teleport_segment(160)]],
        &[],
        &[],
        &[],
        &[round()],
        [Some("BLANKA"), Some("DHALSIM")],
    );

    assert_eq!(teleports.len(), 1);
    assert_eq!(
        teleports[0].inv_end_frame - teleports[0].inv_start_frame + 1,
        TELEPORT_INV_MAX
    );
}
