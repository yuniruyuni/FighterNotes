//! 被ダメージ列を、重複しない1つの主起点へ帰属する。

use super::{
    AdviceCard, AttributedDamageEvent, DamageApproach, DamageBreakdown, DamageContact, DamageOrigin,
};
use crate::attack_info::AttackAttribute;
use crate::frame_data::StrikeKind;
use crate::frame_features::FrameFeatures;
use crate::match_events::{AttackDamageConsistency, DamageEvent, EventConfidence, MatchEvents};

mod candidate;
mod classification;
mod contexts;
mod strike;

pub(crate) const DAMAGE_ATTRIBUTION_VERSION: u32 = 5;

fn centrally_confirms_throw(events: &MatchEvents, damage: &DamageEvent) -> bool {
    events
        .attack_evidence_for_damage(damage)
        .is_some_and(|evidence| {
            evidence.complete
                && !evidence.recovered_from_max
                && evidence.confidence == EventConfidence::High
                && evidence.hp_consistency == AttackDamageConsistency::Consistent
                && evidence.starter_attribute == Some(AttackAttribute::Throw)
        })
}

fn observed_strike_kind(
    events: &MatchEvents,
    damage: &DamageEvent,
) -> Option<strike::StrikeAttribution> {
    let evidence = events.attack_evidence_for_damage(damage)?;
    if !evidence.complete || evidence.confidence != EventConfidence::High {
        return None;
    }
    let kind = match evidence.starter_attribute? {
        AttackAttribute::Upper => StrikeKind::High,
        AttackAttribute::Middle => StrikeKind::Overhead,
        AttackAttribute::Lower => StrikeKind::Low,
        AttackAttribute::Throw => return None,
    };
    Some(strike::StrikeAttribution {
        kind,
        confidence: EventConfidence::High,
    })
}

fn primary_contact(origin: DamageOrigin) -> Option<DamageContact> {
    match origin {
        DamageOrigin::Throw => Some(DamageContact::Throw),
        DamageOrigin::Strike => Some(DamageContact::Strike),
        DamageOrigin::DriveImpact => Some(DamageContact::DriveImpact),
        DamageOrigin::Projectile => Some(DamageContact::Projectile),
        _ => None,
    }
}

pub(crate) fn build_damage_breakdown(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    opponent_character: Option<&str>,
) -> DamageBreakdown {
    let attributed: Vec<_> = events
        .damage
        .iter()
        .filter(|damage| damage.victim == own)
        .enumerate()
        .map(|(index, damage)| {
            let candidate = classification::classify_damage(events, own, damage);
            let strict_throw = centrally_confirms_throw(events, damage);
            // 中央表示は接触属性の証拠なので、Drive Rushやjump-inなどの接近起点は
            // 保持する。単純な打撃または未分類だけを、厳格に整合した投げ表示で補う。
            let origin = if strict_throw
                && matches!(
                    candidate.origin,
                    DamageOrigin::Strike | DamageOrigin::Unclassified
                ) {
                DamageOrigin::Throw
            } else {
                candidate.origin
            };
            let confidence = if origin == DamageOrigin::Throw && origin != candidate.origin {
                EventConfidence::High
            } else {
                candidate.confidence
            };
            let approach = (candidate.origin == DamageOrigin::RawDriveRush)
                .then_some(DamageApproach::RawDriveRush);
            let contact = if strict_throw {
                Some(DamageContact::Throw)
            } else {
                primary_contact(origin)
            };
            let contact_confidence = contact.map(|_| {
                if strict_throw {
                    EventConfidence::High
                } else {
                    confidence
                }
            });
            let strike = (contact == Some(DamageContact::Strike))
                .then(|| {
                    observed_strike_kind(events, damage).or_else(|| {
                        strike::strike_attribution(
                            features,
                            events,
                            own,
                            damage,
                            opponent_character,
                        )
                    })
                })
                .flatten();
            let scene_frame = if damage.pre_freeze_frame <= damage.start_frame
                && (damage.pre_freeze_frame > 0 || damage.start_frame == 0)
            {
                damage.pre_freeze_frame
            } else {
                damage.start_frame
            };
            AttributedDamageEvent {
                sequence_no: index as u32 + 1,
                round_no: damage.round_no,
                start_frame: damage.start_frame,
                end_frame: damage.end_frame,
                scene_frame,
                hp_before: damage.hp_before,
                hp_after: damage.hp_after,
                hp_drop: damage.drop,
                origin,
                confidence,
                approach,
                contact,
                contact_confidence,
                strike_kind: strike.map(|value| value.kind),
                strike_kind_confidence: strike.map(|value| value.confidence),
                contexts: contexts::damage_contexts(events, own, damage),
                attack_evidence: events.attack_evidence_for_damage(damage).cloned(),
            }
        })
        .collect();
    let total_hp_lost = attributed.iter().map(|event| event.hp_drop).sum();
    let classified_hp_lost = attributed
        .iter()
        .filter(|event| event.origin != DamageOrigin::Unclassified)
        .map(|event| event.hp_drop)
        .sum();

    DamageBreakdown {
        attribution_version: DAMAGE_ATTRIBUTION_VERSION,
        total_hp_lost,
        classified_hp_lost,
        events: attributed,
    }
}

pub(crate) fn apply_advice_contexts(breakdown: &mut DamageBreakdown, cards: &[AdviceCard]) {
    contexts::apply_advice_contexts(breakdown, cards);
}

#[cfg(test)]
use candidate::{approximately_same_drop, contact_matches, offer, starts_in, threat_confidence};
#[cfg(test)]
use strike::{frame_index, segment_distance};

#[cfg(test)]
mod tests;
