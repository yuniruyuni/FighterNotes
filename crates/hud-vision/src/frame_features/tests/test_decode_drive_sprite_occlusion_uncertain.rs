use super::support::*;

#[test]
fn test_decode_drive_sprite_occlusion_uncertain() {
    // 計測範囲にかかる幅広 Foreign（スプライト遮蔽）→ uncertain
    use DriveColClass::*;
    let runs = drive_runs_from(&[(Lit, 54), (Foreign, 30), (Lit, 54), (Rest, 186)]);
    let d = decode_drive_runs(&runs, 324);
    assert!(d.uncertain, "幅広 Foreign は遮蔽として uncertain にすべき");
}
