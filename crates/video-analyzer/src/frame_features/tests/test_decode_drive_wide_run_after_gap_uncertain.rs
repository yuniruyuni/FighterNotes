use super::support::*;

#[test]
fn test_decode_drive_wide_run_after_gap_uncertain() {
    // 大ギャップ（>8px）の先の幅広ラン（frame 2223 実測: gap 34 + 37px）
    // = 遮蔽体。分離小島（≤24px）とは幅で区別する
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Lit, 149), (Rest, 34), (Lit, 37), (Rest, 94)]);
    let d = decode_drive_runs(&runs, 314);
    assert!(
        d.uncertain,
        "大ギャップ先の幅広ランは遮蔽として uncertain にすべき"
    );
}
