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
