use super::support::*;

#[test]
fn test_decode_drive_foreign_far_from_chain_uncertain() {
    // 連鎖の遠方にある幅広 Foreign（frame 2221 実測: 遮蔽体が連鎖を短く
    // 切断しつつ Foreign 24px が遠方に出現）→ uncertain
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Lit, 72), (Rest, 125), (Foreign, 24), (Rest, 93)]);
    let d = decode_drive_runs(&runs, 314);
    assert!(
        d.uncertain,
        "遠方の幅広 Foreign は遮蔽として uncertain にすべき"
    );
}
