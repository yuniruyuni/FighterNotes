use super::support::*;

#[test]
fn test_fill_ratio_ko_ghost_only_reads_zero_certain() {
    // リグレッションテスト（frame 4063-4078 相当、ラウンド終了時）:
    // KO 直後は HP=0 だがコンボで失った分の暗いゴースト残像がバー全域に
    // 点灯し続ける（HUD が消えるまで約 20 フレーム）。
    // fill（明るい充填色）を一度も見ずに Ghost に遭遇した場合、
    // HP≈0% の確定値（fill_ratio=0, uncertain=false）を返すべき。
    let rgba = make_rgba_p1_bar_ghost_only(0.24);
    let (fill, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!(
        !uncertain,
        "KO ゴースト残像のみのバーは uncertain=false であるべき: fill={fill:.3}"
    );
    assert!(
        fill < 0.01,
        "KO ゴースト残像のみのバーは HP≈0 と読むべき: {fill:.3}"
    );
}
