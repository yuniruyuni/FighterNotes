use super::DamageOrigin;
use crate::match_events::{DamageEvent, EventConfidence};

pub(super) const CONTACT_BACK: u32 = 5;
const CONTACT_DAMAGE_WINDOW: u32 = 25;

#[derive(Debug, Clone, Copy)]
pub(super) struct Candidate {
    pub(super) origin: DamageOrigin,
    pub(super) confidence: EventConfidence,
    priority: u8,
    distance: u32,
}

impl Candidate {
    pub(super) fn unclassified() -> Self {
        Self {
            origin: DamageOrigin::Unclassified,
            confidence: EventConfidence::Low,
            priority: 0,
            distance: u32::MAX,
        }
    }
}

fn confidence_rank(confidence: EventConfidence) -> u8 {
    match confidence {
        EventConfidence::High => 2,
        EventConfidence::Medium => 1,
        EventConfidence::Low => 0,
    }
}

pub(super) fn threat_confidence(confidence: f32) -> Option<EventConfidence> {
    if confidence >= 0.85 {
        Some(EventConfidence::High)
    } else if confidence >= 0.65 {
        Some(EventConfidence::Medium)
    } else {
        None
    }
}

pub(super) fn offer(
    current: &mut Option<Candidate>,
    origin: DamageOrigin,
    confidence: EventConfidence,
    priority: u8,
    anchor: u32,
    damage: &DamageEvent,
) {
    if confidence == EventConfidence::Low {
        return;
    }
    let next = Candidate {
        origin,
        confidence,
        priority,
        distance: damage.start_frame.abs_diff(anchor),
    };
    let replace = current.is_none_or(|existing| {
        next.priority > existing.priority
            || (next.priority == existing.priority
                && confidence_rank(next.confidence) > confidence_rank(existing.confidence))
            || (next.priority == existing.priority
                && next.confidence == existing.confidence
                && next.distance < existing.distance)
    });
    if replace {
        *current = Some(next);
    }
}

pub(super) fn contact_matches(damage: &DamageEvent, frame: u32) -> bool {
    damage.start_frame.saturating_add(CONTACT_BACK) >= frame
        && damage.start_frame <= frame.saturating_add(CONTACT_DAMAGE_WINDOW)
}

pub(super) fn starts_in(damage: &DamageEvent, start: u32, end: u32) -> bool {
    damage.start_frame >= start && damage.start_frame <= end
}

pub(super) fn approximately_same_drop(actual: f32, attributed: f32) -> bool {
    (actual - attributed).abs() <= 0.005
}
