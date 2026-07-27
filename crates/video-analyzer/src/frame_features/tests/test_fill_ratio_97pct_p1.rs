use super::support::*;

#[test]
fn test_fill_ratio_97pct_p1() {
    let rgba = make_rgba_p1_bar(0.97);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!((fill - 0.97).abs() < 0.02, "97% HP → ~0.97, got {fill:.3}");
    assert!(!uncertain);
}
