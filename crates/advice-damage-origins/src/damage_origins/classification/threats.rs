use super::super::candidate::{contact_matches, offer, threat_confidence, Candidate};
use crate::match_events::{DamageEvent, MatchEvents, ThreatOutcome};
use crate::DamageOrigin;

pub fn offer_compound(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for threat in events.compound_threats.iter().filter(|threat| {
        threat.attacker == opponent
            && threat.defender == own
            && threat.round_no == damage.round_no
            && threat.outcome == ThreatOutcome::Hit
            && threat.damage > 0.0
    }) {
        let anchor = threat
            .followup_contact_frame
            .unwrap_or(threat.followup_attack_frame);
        if contact_matches(damage, anchor) {
            if let Some(confidence) = threat_confidence(threat.confidence) {
                offer(
                    candidate,
                    DamageOrigin::CompoundThreat,
                    confidence,
                    100,
                    anchor,
                    damage,
                );
            }
        }
    }
}

pub fn offer_teleport(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for teleport in events.teleports.iter().filter(|teleport| {
        teleport.attacker == opponent
            && teleport.defender == own
            && teleport.round_no == damage.round_no
            && teleport.outcome == ThreatOutcome::Hit
            && teleport.damage > 0.0
    }) {
        let anchor = teleport
            .followup_contact_frame
            .or(teleport.followup_attack_frame)
            .unwrap_or(teleport.inv_end_frame);
        if contact_matches(damage, anchor) {
            if let Some(confidence) = threat_confidence(teleport.confidence) {
                offer(
                    candidate,
                    DamageOrigin::Teleport,
                    confidence,
                    95,
                    anchor,
                    damage,
                );
            }
        }
    }
}
