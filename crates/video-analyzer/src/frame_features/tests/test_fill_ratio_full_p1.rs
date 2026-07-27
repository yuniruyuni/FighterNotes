use super::support::*;

#[test]
fn test_fill_ratio_full_p1() {
    let rgba = make_rgba_p1_bar(1.0);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!((fill - 1.0).abs() < 0.02, "full HP → ~1.0, got {fill:.3}");
    assert!(!uncertain, "full HP → not uncertain");
}
