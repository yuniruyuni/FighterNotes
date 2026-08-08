use super::support::*;

#[test]
fn test_decode_drive_lit_beyond_chain_uncertain() {
    // 連鎖範囲（gap ≤56px）の先に実体 Lit（frame 2671/2682 実測形状）
    // = 遮蔽体がゲージ中間を暗転させた → uncertain
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Lit, 34), (Rest, 164), (Lit, 33), (Rest, 83)]);
    let d = decode_drive_runs(&runs, 314);
    assert!(
        d.uncertain,
        "連鎖範囲外の実体 Lit は遮蔽として uncertain にすべき"
    );
}
