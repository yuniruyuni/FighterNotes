use super::support::*;

#[test]
fn test_decode_drive_fragmented_wide_lit_uncertain() {
    // 被覆率が低いが太いラン（幅 30 > EMPTY_MAX_STROKE）を含む
    // = スプライト遮蔽 → uncertain（バーンアウトと誤判定しない）
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Lit, 30), (Rest, 40), (Lit, 20), (Rest, 224)]);
    let d = decode_drive_runs(&runs, 314);
    assert!(
        d.uncertain,
        "太いラン混じりの断片化 Lit は uncertain にすべき"
    );
    assert!(!d.burnout);
}
