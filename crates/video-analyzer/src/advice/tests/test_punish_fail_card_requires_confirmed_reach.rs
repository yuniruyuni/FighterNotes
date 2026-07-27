use super::support::*;

#[test]
fn test_punish_fail_card_requires_confirmed_reach() {
    let mut ev = empty_events();
    ev.punishes.push(PunishChance {
        frame: 200,
        side: 1,
        advantage: 7,
        outcome: PunishOutcome::WhiffFail,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 194,
        recovery_end_frame: 207,
        source_contact_frame: Some(193),
        attack_start_frame: Some(200),
        attack_active_frame: Some(205),
        reachability: PunishReachability::Unknown,
        punished_drop: 0.14,
        pressed: "弱".to_string(),
        round_no: 1,
    });

    assert!(detect_punish_fail(&ev, 1, Some("LUKE")).is_none());
    ev.punishes[0].reachability = PunishReachability::Confirmed;
    let card = detect_punish_fail(&ev, 1, Some("LUKE"))
        .expect("spatially confirmed close whiff is actionable");
    assert_eq!(card.id, "punish_fail");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_invites_user_review(&card);
    assert!(card.title.contains("ガード後の反撃が届かなかった"));
    assert!(card.evidence[0].label.contains("ガード後の反撃空振り"));
    assert!(card.evidence[0].label.contains("距離確認"));
    assert!(card.description.contains("確定する技があるか"));
    assert!(!card.description.contains("前ステップ"));
    assert!(!card.practice.contains("前ステ投げ"));

    let mut repeated = ev.punishes[0].clone();
    repeated.frame = 1200;
    repeated.recovery_start_frame = 1194;
    repeated.recovery_end_frame = 1207;
    repeated.source_contact_frame = Some(1193);
    repeated.attack_start_frame = Some(1200);
    repeated.attack_active_frame = Some(1205);
    ev.punishes.push(repeated);
    assert_eq!(
        detect_punish_fail(&ev, 1, Some("LUKE")).unwrap().kind,
        AdviceKind::Diagnosis
    );

    for punish in &mut ev.punishes {
        punish.reachability = PunishReachability::OutOfRange;
    }
    assert!(detect_punish_fail(&ev, 1, Some("LUKE")).is_none());
}
