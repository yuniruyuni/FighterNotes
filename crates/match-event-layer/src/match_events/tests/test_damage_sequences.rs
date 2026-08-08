use super::support::*;

#[test]
fn test_damage_sequences() {
    let fs = synth_two_rounds();
    let ev = build_match_events(&fs, &[], &[], None, "p1");
    let p2_dmg: Vec<_> = ev.damage.iter().filter(|d| d.victim == 2).collect();
    assert_eq!(p2_dmg.len(), 3, "P2 は R1 で 3 回被弾: {:?}", p2_dmg);
    assert!(
        (p2_dmg[0].drop - 0.29).abs() < 0.03,
        "初回被弾 -0.29: {}",
        p2_dmg[0].drop
    );
    let p1_dmg: Vec<_> = ev.damage.iter().filter(|d| d.victim == 1).collect();
    assert_eq!(p1_dmg.len(), 1, "P1 は R2 で 1 回被弾: {:?}", p1_dmg);
}
