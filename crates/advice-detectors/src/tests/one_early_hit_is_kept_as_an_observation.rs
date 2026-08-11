use super::support::*;

#[test]
fn one_early_hit_is_kept_as_an_observation() {
    let mut ev = empty_events();
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 100,
        pre_freeze_frame: 95,
        end_frame: 140,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    });
    let round = RoundSummary {
        round_no: 1,
        start_frame: 0,
        end_frame: 1000,
        won: None,
        own_hp_end: 0.9,
        opp_hp_end: 1.0,
        own_hp_lost: 0.1,
        opp_hp_lost: 0.0,
        own_hits_taken: 1,
        early_hit: true,
        own_burnouts: 0,
        detection_confidence: "medium".to_string(),
    };

    let card = detect_early_hits(&ev, &[round], 1).expect("単発も確認場面にする");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert!((card.severity - 0.05).abs() < 1e-6);
    assert_invites_user_review(&card);
    assert_eq!(card.evidence.len(), 1);
}
