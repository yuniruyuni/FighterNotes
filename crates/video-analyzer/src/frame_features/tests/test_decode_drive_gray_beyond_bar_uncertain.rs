use super::support::*;

#[test]
fn test_decode_drive_gray_beyond_bar_uncertain() {
    // バーンアウト回復バーの先に幅広 Gray（frame 2996 実測: 遮蔽体が
    // バーを分断して recovery が偽の低値になる）→ uncertain
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Gray, 24), (Rest, 40), (Gray, 128), (Rest, 122)]);
    let d = decode_drive_runs(&runs, 314);
    assert!(
        d.uncertain,
        "回復バー分断は uncertain にすべき: burnout={} rec={}",
        d.burnout, d.recovery
    );
}
