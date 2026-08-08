use super::support::*;

#[test]
fn test_decode_drive_empty_text_burnout_entry() {
    // バーンアウト突入演出の「EMPTY」文字（frame 2095 実測形状）:
    // 細切れの Lit ストローク群（被覆率 ≈0.5）→ バーンアウト突入と判定
    use DriveColClass::*;
    let runs = drive_runs_from(&[
        (Lit, 22),
        (Rest, 6),
        (Lit, 11),
        (Rest, 9),
        (Lit, 9),
        (Rest, 4),
        (Lit, 1),
        (Rest, 1),
        (Lit, 6),
        (Rest, 29),
        (Lit, 1),
        (Rest, 1),
        (Lit, 4),
        (Rest, 10),
        (Lit, 1),
        (Rest, 10),
        (Lit, 8),
        (Rest, 181),
    ]);
    let d = decode_drive_runs(&runs, 314);
    assert!(!d.uncertain, "EMPTY 文字はバーンアウト突入として確定すべき");
    assert!(d.burnout, "EMPTY 文字シグネチャは burnout=true");
    assert!(d.recovery < 0.01, "突入瞬間の回復進捗は 0");
}
