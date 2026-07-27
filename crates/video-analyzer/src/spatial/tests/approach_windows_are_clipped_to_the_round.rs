use super::*;

#[test]
fn approach_windows_are_clipped_to_the_round() {
    use crate::match_events::{ThrowActionEvent, ThrowOutcome};

    let mut events = empty_events();
    events.throw_actions.push(ThrowActionEvent {
        thrower: 1,
        input_frame: 10,
        startup_frame: Some(12),
        active_frame: Some(15),
        outcome: ThrowOutcome::Hit,
        damage: 0.12,
        approach: ThrowApproach::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.drive_rushes.push(DriveRushEvent {
        side: 2,
        frame: 380,
        raw: true,
        outcome: DriveRushOutcome::NoContact,
        contact_frame: None,
        damage: 0.0,
        confidence: EventConfidence::Medium,
        round_no: 1,
    });

    let windows = spatial_candidate_windows(&events);
    assert_eq!(
        windows
            .iter()
            .map(|window| (window.start_frame, window.end_frame))
            .collect::<Vec<_>>(),
        [(0, 45), (365, 399)]
    );
}
