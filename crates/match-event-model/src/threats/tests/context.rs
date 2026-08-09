use super::super::*;
use super::support::*;

#[test]
fn teleport_context_separates_naked_movement_and_combo_cases() {
    let features: Vec<_> = (0..400).map(feature).collect();

    let free = timeline("left", &[]);
    let naked = timeline("right", &[(170, 176, "inv_full"), (190, 195, "active")]);
    let naked_meter = [state_per_frame(&free, 400), state_per_frame(&naked, 400)];
    let (_, teleports, compounds) = extract_test_threats!(
        &features,
        [&free, &naked],
        &naked_meter,
        &[vec![], vec![teleport_segment(160)]],
        &[],
        &[],
        &[],
        &[round()],
        [Some("BLANKA"), Some("DHALSIM")],
    );
    assert_eq!(teleports[0].context, TeleportContext::NakedAttack);
    assert!(compounds.is_empty());

    let movement = timeline("right", &[(170, 176, "inv_full")]);
    let movement_meter = [state_per_frame(&free, 400), state_per_frame(&movement, 400)];
    let (_, teleports, _) = extract_test_threats!(
        &features,
        [&free, &movement],
        &movement_meter,
        &[vec![], vec![teleport_segment(160)]],
        &[],
        &[],
        &[],
        &[round()],
        [Some("BLANKA"), Some("DHALSIM")],
    );
    assert_eq!(teleports[0].context, TeleportContext::MovementOnly);

    let stunned = timeline("left", &[(150, 180, "stun")]);
    let combo_meter = [state_per_frame(&stunned, 400), state_per_frame(&naked, 400)];
    let (_, teleports, _) = extract_test_threats!(
        &features,
        [&stunned, &naked],
        &combo_meter,
        &[vec![], vec![teleport_segment(160)]],
        &[],
        &[],
        &[],
        &[round()],
        [Some("BLANKA"), Some("DHALSIM")],
    );
    assert_eq!(teleports[0].context, TeleportContext::DefenderUnavailable);
    assert!(!teleports[0].defender_actionable);
}
