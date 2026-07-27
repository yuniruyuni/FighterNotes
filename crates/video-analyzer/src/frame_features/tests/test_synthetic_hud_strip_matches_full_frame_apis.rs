use super::support::*;

#[test]
fn test_synthetic_hud_strip_matches_full_frame_apis() {
    let mut rgba = make_rgba_p1_bar_yellow(0.25);
    paint_full_left_drive_gauge(&mut rgba);
    let strip = hud_strip_from_frame(&rgba);

    let hp_full = hp_fill_ratio_with_quality(&rgba, 1920, 1080, "p1");
    let hp_strip = hp_fill_ratio_with_quality_from_hud_strip(&strip, 1920, 1080, "p1");
    assert!((hp_full.0 - hp_strip.0).abs() < 1e-6);
    assert_eq!(hp_full.1, hp_strip.1);

    let drive_full = drive_gauge_read(&rgba, 1920, 1080, "left");
    let drive_strip = drive_gauge_read_from_hud_strip(&strip, 1920, 1080, "left");
    assert!(!drive_full.uncertain);
    assert!(drive_full.value > 5.5);
    assert!((drive_full.value - drive_strip.value).abs() < 1e-6);
    assert_eq!(drive_full.burnout, drive_strip.burnout);
    assert!((drive_full.recovery - drive_strip.recovery).abs() < 1e-6);
    assert_eq!(drive_full.uncertain, drive_strip.uncertain);
}
