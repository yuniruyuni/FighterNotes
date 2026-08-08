use super::support::*;

#[test]
fn test_fill_ratio_half_p1() {
    let rgba = make_rgba_p1_bar(0.5);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!((fill - 0.5).abs() < 0.02, "50% HP → ~0.5, got {fill:.3}");
    assert!(!uncertain);
}
