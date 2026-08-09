use super::support::*;

#[test]
fn test_punish_missed_card_requires_confirmed_reach() {
    let mut ev = empty_events();
    ev.punishes.push(PunishChance {
        frame: 200,
        side: 1,
        advantage: 4,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 196,
        recovery_end_frame: 203,
        source_contact_frame: Some(195),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    });

    assert!(detect_punish_missed(&ev, 1, Some("BLANKA")).is_none());
    ev.punishes[0].reachability = PunishReachability::Confirmed;
    let card =
        detect_punish_missed(&ev, 1, Some("BLANKA")).expect("近距離を確認できた候補だけ指摘する");
    assert_eq!(card.id, "punish_missed");
    assert!(card.evidence[0].label.contains("近距離確認"));

    ev.punishes[0].reachability = PunishReachability::OutOfRange;
    assert!(detect_punish_missed(&ev, 1, Some("BLANKA")).is_none());
}
