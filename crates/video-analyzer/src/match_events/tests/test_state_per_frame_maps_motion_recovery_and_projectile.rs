use super::support::*;

#[test]
fn test_state_per_frame_maps_motion_recovery_and_projectile() {
    let tl = synth_timeline(vec![
        (100, "counter", 10, 11),
        (101, "projectile_active", 12, 13),
        (102, "motion_recovery", 14, 17),
    ]);
    let st = state_per_frame(&tl, 20);
    assert_eq!(st[12], MeterState::ProjectileActive);
    assert_eq!(st[14], MeterState::MotionRecovery);
    assert_eq!(st[17], MeterState::MotionRecovery);
    assert_eq!(st[18], MeterState::Free);
    let confidence = confidence_per_frame(&tl, 20);
    assert_eq!(confidence[12], 1.0);
    assert_eq!(confidence[18], 0.0);
}
