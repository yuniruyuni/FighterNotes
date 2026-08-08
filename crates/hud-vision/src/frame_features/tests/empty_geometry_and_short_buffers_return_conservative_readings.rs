use super::support::*;

#[test]
fn empty_geometry_and_short_buffers_return_conservative_readings() {
    assert_eq!(hp_bar_score(&[], 0, 0, "p1"), 0.0);
    assert!(hp_col_active(&[], 0, 0, "p1").is_empty());
    assert!(hp_col_orange(&[], 0, 0, "p1").is_empty());
    assert!(hp_col_yellow(&[], 0, 0, "p1").is_empty());
    assert_eq!(hp_fill_ratio_with_quality(&[], 0, 0, "p1"), (0.0, true));

    let drive = drive_gauge_read(&[0; 4], 1920, 1080, "left");
    assert_eq!(drive.value, 0.0);
    assert!(!drive.burnout);
    assert_eq!(drive.recovery, 0.0);
    assert!(drive.uncertain);
}
