use super::support::*;

#[test]
fn test_fill_ratio_dithered_full_p1() {
    let rgba = make_rgba_p1_bar_dithered(1.0);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!(fill > 0.95, "dithered full → >0.95, got {fill:.3}");
    assert!(!uncertain, "dithered full → not uncertain (density≈25%)");
}
