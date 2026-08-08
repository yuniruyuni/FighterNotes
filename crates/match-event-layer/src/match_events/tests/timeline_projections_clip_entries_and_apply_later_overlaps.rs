use super::support::*;

#[test]
fn timeline_projections_clip_entries_and_apply_later_overlaps() {
    let timeline = synth_segmented_timeline(3, vec![(10, "counter", 2, 4), (11, "stun", 4, 9)]);

    assert_eq!(gf_per_frame(&timeline, 6), vec![-1, -1, 10, 10, 11, 11]);
    assert_eq!(epoch_per_frame(&timeline, 6), vec![-1, -1, 3, 3, 3, 3]);
    assert_eq!(
        state_per_frame(&timeline, 6),
        vec![
            MeterState::Free,
            MeterState::Free,
            MeterState::Startup,
            MeterState::Startup,
            MeterState::Stun,
            MeterState::Stun,
        ]
    );
    assert_eq!(
        confidence_per_frame(&timeline, 6),
        vec![0.0, 0.0, 1.0, 1.0, 1.0, 1.0]
    );
}
