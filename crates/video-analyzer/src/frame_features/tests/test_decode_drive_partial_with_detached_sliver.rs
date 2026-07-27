use super::support::*;

#[test]
fn test_decode_drive_partial_with_detached_sliver() {
    // 排出中の部分セルが本体から 15px 離れた小島（frame 3400 実測形状）
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Lit, 62), (Rest, 15), (Lit, 5), (Rest, 242)]);
    let d = decode_drive_runs(&runs, 324);
    assert!(!d.uncertain, "分離小島は部分セルとして連鎖すべき");
    assert!(
        (d.value - 82.0 / 324.0 * 6.0).abs() < 0.01,
        "value は小島遠端まで含むべき: {}",
        d.value
    );
}
