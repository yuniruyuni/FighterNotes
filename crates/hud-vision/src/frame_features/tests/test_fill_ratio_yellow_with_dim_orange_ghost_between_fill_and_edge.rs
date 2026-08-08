use super::support::*;

#[test]
fn test_fill_ratio_yellow_with_dim_orange_ghost_between_fill_and_edge() {
    // リグレッションテスト（frame 4010-4013 相当）:
    // dim orange ghost（V≈160<200, G/R≈0.84>0.82）が fill と fill_edge の間に存在する場合、
    // ghost を Fill と分類して fill_edge White を正しく検出できること。
    // fill_ratio ≈ total HP（fill+ghost=24%）、uncertain=false を期待する。
    const TOTAL_RATIO: f32 = 0.24;
    const GHOST_WIDTH: usize = 61; // 9% 相当
    let rgba = make_rgba_p1_bar_yellow_with_dim_orange_ghost(TOTAL_RATIO, GHOST_WIDTH);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!(
        !uncertain,
        "dim orange ghost + fill のバーは uncertain=false であるべき: fill={fill:.3}"
    );
    assert!((fill - TOTAL_RATIO).abs() < 0.03,
        "dim orange ghost ありバーの fill_ratio が期待範囲外: {fill:.3} (expected≈{TOTAL_RATIO:.2})");
}

//
