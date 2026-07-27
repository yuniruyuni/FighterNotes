use super::support::*;

#[test]
fn test_decode_drive_background_bleed_not_burnout() {
    // 背景透けの Gray がアンカーから離れた位置にあってもバーンアウトではない
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Rest, 100), (Gray, 60), (Rest, 164)]);
    let d = decode_drive_runs(&runs, 324);
    assert!(
        d.uncertain,
        "Lit なし・アンカー起点の回復バーなし → uncertain: burnout={}",
        d.burnout
    );
    assert!(!d.burnout, "アンカーから離れた Gray は回復バーではない");
}
