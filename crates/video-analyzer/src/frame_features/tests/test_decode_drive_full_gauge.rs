use super::support::*;

#[test]
fn test_decode_drive_full_gauge() {
    // 満タン: セル間ギャップ（Rest 2-4px）を挟んだ 6 セル
    use DriveColClass::*;
    let runs = drive_runs_from(&[
        (Lit, 52),
        (Rest, 2),
        (Lit, 52),
        (Rest, 2),
        (Lit, 52),
        (Rest, 2),
        (Lit, 52),
        (Rest, 2),
        (Lit, 52),
        (Rest, 2),
        (Lit, 54),
    ]);
    let d = decode_drive_runs(&runs, 324);
    assert!(!d.uncertain);
    assert!(!d.burnout);
    assert!(
        (d.value - 6.0).abs() < 0.05,
        "満タンは 6.0 を読むべき: {}",
        d.value
    );
}
