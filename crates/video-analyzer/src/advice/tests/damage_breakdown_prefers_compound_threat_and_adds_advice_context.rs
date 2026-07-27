use super::support::*;

#[test]
fn damage_breakdown_prefers_compound_threat_and_adds_advice_context() {
    use crate::match_events::{ContactEvent, DpReachability, TeleportContext, ThreatOutcome};

    let mut ev = empty_events();
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 230,
        pre_freeze_frame: 180,
        end_frame: 260,
        hp_before: 1.0,
        hp_after: 0.8,
        drop: 0.2,
        round_no: 1,
    });
    ev.contacts.push(ContactEvent {
        frame: 220,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: true,
        round_no: 1,
    });
    ev.teleports.push(TeleportEvent {
        attacker: 2,
        defender: 1,
        input_frame: 180,
        inv_start_frame: 190,
        inv_end_frame: 200,
        followup_attack_frame: Some(220),
        followup_contact_frame: Some(220),
        airborne: true,
        defender_actionable: true,
        context: TeleportContext::ProjectileCovered,
        response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.2,
        dp_reachability: DpReachability::Unknown,
        round_no: 1,
        confidence: 0.9,
    });
    ev.compound_threats.push(CompoundThreat {
        attacker: 2,
        defender: 1,
        projectile_start_frame: 150,
        teleport_frame: 180,
        followup_attack_frame: 220,
        followup_contact_frame: Some(220),
        projectile_response: None,
        followup_response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.2,
        round_no: 1,
        confidence: 0.75,
    });

    let mut breakdown = super::damage_origins::build_damage_breakdown(&[], &ev, 1, None);
    assert_eq!(breakdown.events[0].origin, DamageOrigin::CompoundThreat);
    assert_eq!(breakdown.events[0].confidence, EventConfidence::Medium);
    assert_eq!(breakdown.events[0].scene_frame, 180);

    super::damage_origins::apply_advice_contexts(
        &mut breakdown,
        &[AdviceCard {
            id: "mashing".to_string(),
            kind: AdviceKind::Observation,
            confidence: EventConfidence::High,
            title: String::new(),
            severity: 0.2,
            description: String::new(),
            practice: String::new(),
            evidence: vec![EvidenceClip {
                frame: 210,
                end_frame: Some(260),
                label: String::new(),
            }],
        }],
    );
    assert_eq!(breakdown.events[0].contexts, [DamageContext::Mashing]);

    let json = serde_json::to_value(&breakdown).expect("WASM report JSONへ直列化できる");
    assert_eq!(json["attribution_version"], 2);
    assert_eq!(json["events"][0]["origin"], "compound_threat");
    assert_eq!(json["events"][0]["confidence"], "medium");
    assert_eq!(json["events"][0]["contexts"][0], "mashing");
}
