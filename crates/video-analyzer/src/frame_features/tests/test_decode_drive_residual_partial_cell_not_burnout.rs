use super::support::*;

#[test]
fn test_decode_drive_residual_partial_cell_not_burnout() {
    // ほぼ空のゲージ（残り 0.5 セル）の部分セルがヒットフラッシュで
    // 3 断片に割れた形（frame 1452-1464 実測: 幅 4/1/7、広がり 28px）。
    // EMPTY 文字（広がり 109-133px）とは広がりで区別し、burnout に
    // しない（遮蔽扱い uncertain → 時間フィルタが前値で埋める）
    use DriveColClass::*;
    let runs = drive_runs_from(&[
        (Gray, 1),
        (Rest, 2),
        (Lit, 4),
        (Rest, 12),
        (Lit, 1),
        (Rest, 1),
        (Lit, 7),
        (Rest, 286),
    ]);
    let d = decode_drive_runs(&runs, 314);
    assert!(!d.burnout, "残存部分セルの断片を EMPTY 文字と誤認しない");
    assert!(d.uncertain, "細切れ Lit は遮蔽として uncertain");
}
