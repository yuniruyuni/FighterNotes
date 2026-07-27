use super::super::candidate::{contact_matches, offer, threat_confidence, Candidate};
use crate::advice::DamageOrigin;
use crate::match_events::{DamageEvent, EventConfidence, MatchEvents};

pub(super) fn offer_projectile_contacts(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for contact in events.contacts.iter().filter(|contact| {
        contact.attacker == opponent
            && contact.victim == own
            && contact.round_no == damage.round_no
            && contact.hit
            && contact.projectile
    }) {
        if contact_matches(damage, contact.frame) {
            offer(
                candidate,
                DamageOrigin::Projectile,
                EventConfidence::High,
                75,
                contact.frame,
                damage,
            );
        }
    }
}

pub(super) fn offer_projectile_threats(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for projectile in events
        .projectiles
        .iter()
        .filter(|projectile| projectile.owner == opponent && projectile.round_no == damage.round_no)
    {
        let Some(anchor) = projectile.contact_frame else {
            continue;
        };
        if contact_matches(damage, anchor) {
            if let Some(confidence) = threat_confidence(projectile.confidence) {
                offer(
                    candidate,
                    DamageOrigin::Projectile,
                    confidence,
                    74,
                    anchor,
                    damage,
                );
            }
        }
    }
}

pub(super) fn offer_strike_contacts(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for contact in events.contacts.iter().filter(|contact| {
        contact.attacker == opponent
            && contact.victim == own
            && contact.round_no == damage.round_no
            && contact.hit
            && !contact.projectile
    }) {
        if contact_matches(damage, contact.frame) {
            offer(
                candidate,
                DamageOrigin::Strike,
                EventConfidence::High,
                10,
                contact.frame,
                damage,
            );
        }
    }
}
