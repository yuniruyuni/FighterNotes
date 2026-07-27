use super::support::*;

#[test]
fn test_fill_ratio_dithered_half_p1() {
    let rgba = make_rgba_p1_bar_dithered(0.5);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!(
        (fill - 0.5).abs() < 0.03,
        "dithered 50% → ~0.5, got {fill:.3}"
    );
    assert!(!uncertain);
}

// ── spike_hold_forward_pass ───────────────────────────────────────────────
