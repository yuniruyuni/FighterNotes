use super::*;

#[test]
fn jump_contact_gets_a_short_spatial_window() {
    let mut events = empty_events();
    events
        .jumps
        .push(jump(100, JumpOutcome::UnverifiedHit, "UR"));

    let windows = spatial_candidate_windows(&events);
    assert_eq!(windows.len(), 1);
    assert_eq!((windows[0].start_frame, windows[0].end_frame), (94, 122));
    assert_eq!(
        windows[0].airborne_hints,
        [SpatialHintRange {
            side: 2,
            start_frame: 107,
            end_frame: 120,
        }]
    );
}
