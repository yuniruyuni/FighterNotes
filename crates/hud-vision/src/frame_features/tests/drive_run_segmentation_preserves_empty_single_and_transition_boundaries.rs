use super::support::*;

#[test]
fn drive_run_segmentation_preserves_empty_single_and_transition_boundaries() {
    use DriveColClass::*;

    assert!(segment_drive_runs(&[]).is_empty());
    assert_eq!(segment_drive_runs(&[Lit, Lit]), vec![(Lit, 0, 1)]);
    assert_eq!(
        segment_drive_runs(&[Rest, Lit, Lit, Gray, Rest]),
        vec![(Rest, 0, 0), (Lit, 1, 2), (Gray, 3, 3), (Rest, 4, 4)]
    );
}
