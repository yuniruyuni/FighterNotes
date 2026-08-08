use super::support::*;

#[test]
fn test_layered_defense_card_requires_short_parry_and_damage() {
    let mut ev = empty_events();
    ev.compound_threats.push(CompoundThreat {
        attacker: 2,
        defender: 1,
        projectile_start_frame: 100,
        teleport_frame: 140,
        followup_attack_frame: 180,
        followup_contact_frame: Some(180),
        projectile_response: Some(DefenseResponse {
            side: 1,
            kind: DefenseResponseKind::Parry,
            start_frame: 150,
            end_frame: 160,
        }),
        followup_response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.12,
        round_no: 1,
        confidence: 0.8,
    });

    let card = detect_layered_defense(&ev, 1).expect("短いパリィを指摘");
    assert_eq!(card.id, "layered_defense");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_invites_user_review(&card);
    assert_eq!(card.evidence[0].frame, 100);

    ev.compound_threats[0]
        .projectile_response
        .as_mut()
        .unwrap()
        .end_frame = 190;
    assert!(detect_layered_defense(&ev, 1).is_none());

    ev.compound_threats[0]
        .projectile_response
        .as_mut()
        .unwrap()
        .end_frame = 160;
    let mut repeated = ev.compound_threats[0].clone();
    repeated.projectile_start_frame += 1000;
    repeated.teleport_frame += 1000;
    repeated.followup_attack_frame += 1000;
    repeated.followup_contact_frame = Some(1180);
    repeated.round_no = 2;
    ev.compound_threats.push(repeated);
    assert_eq!(
        detect_layered_defense(&ev, 1).unwrap().kind,
        AdviceKind::Diagnosis
    );
}
