use super::support::*;

#[test]
fn test_decode_drive_detached_sliver_still_reads() {
    // 排出中の分離小島（被覆率 0.81）は引き続き値として読めること
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Lit, 62), (Rest, 15), (Lit, 5), (Rest, 232)]);
    let d = decode_drive_runs(&runs, 314);
    assert!(!d.uncertain, "被覆率 0.81 の分離小島は正常読みすべき");
    assert!(!d.burnout);
}
