//! 被ダメージ列を、重複しない1つの主起点へ帰属する。

use super::{AdviceCard, AttributedDamageEvent, DamageBreakdown, DamageOrigin};
use crate::frame_features::FrameFeatures;
use crate::match_events::MatchEvents;

mod candidate;
mod classification;
mod contexts;
mod strike;

pub(crate) const DAMAGE_ATTRIBUTION_VERSION: u32 = 2;

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
            let strike = (candidate.origin == DamageOrigin::Strike)
                .then(|| {
                    strike::strike_attribution(features, events, own, damage, opponent_character)
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
                origin: candidate.origin,
                confidence: candidate.confidence,
                strike_kind: strike.map(|value| value.kind),
                strike_kind_confidence: strike.map(|value| value.confidence),
                contexts: contexts::damage_contexts(events, own, damage),
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
