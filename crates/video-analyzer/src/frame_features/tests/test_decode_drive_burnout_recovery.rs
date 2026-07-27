use super::support::*;

#[test]
fn test_decode_drive_burnout_recovery() {
    // バーンアウト回復バー: アンカーから Gray スラブ 50%
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Gray, 162), (Rest, 162)]);
    let d = decode_drive_runs(&runs, 324);
    assert!(!d.uncertain);
    assert!(d.burnout, "Gray スラブはバーンアウトと判定すべき");
    assert!(
        (d.recovery - 0.5).abs() < 0.01,
        "回復進捗 0.5: {}",
        d.recovery
    );
}
