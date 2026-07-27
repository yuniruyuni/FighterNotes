use super::support::*;

#[test]
fn test_fill_ratio_yellow_with_bright_orange_overwriting_fill_edge() {
    // バグ回帰テスト（frame 4014 相当）:
    // 高輝度橙色（V>200, H=22, G/R<0.80）が fill_edge_white を上書きすると、
    // 修正前: bright orange が Fill に誤分類され fill_ratio ≈ fill+damage（≈0.24）になる。
    // 修正後: bright orange が Orange に正分類され uncertain=true になること。
    const FILL_RATIO: f32 = 0.15;
    const DAMAGE_WIDTH: usize = 62;
    let rgba =
        make_rgba_p1_bar_yellow_with_bright_orange_overwriting_fill_edge(FILL_RATIO, DAMAGE_WIDTH);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);

    let expected_wrong = FILL_RATIO + (DAMAGE_WIDTH + 2) as f32 / 681.0; // ≈0.24
    assert!(
        uncertain || (fill - FILL_RATIO).abs() < 0.05,
        "高輝度橙色で fill_edge 上書き時は uncertain=true か fill≈{FILL_RATIO:.2} であるべき \
         (修正前は fill≈{expected_wrong:.2} になる): fill={fill:.3}, uncertain={uncertain}"
    );
}
