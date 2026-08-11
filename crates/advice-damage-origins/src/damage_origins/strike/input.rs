use super::startup::startup_observation;
use super::{StrikeAttribution, ATTACK_LOOKBACK};
use crate::frame_data;
use crate::frame_features::FrameFeatures;
use crate::match_events::{ContactEvent, EventConfidence, InputSegment, MatchEvents};

const INPUT_START_LINK_WINDOW: u32 = 6;

pub fn segment_distance(segment: &InputSegment, target: u32) -> u32 {
    // 区間内なら両方 0、手前なら左側、後ろなら右側だけが正になる。
    // 境界で分岐する必要がなく、start/end のどちらからの距離かも対称になる。
    segment
        .start_frame
        .saturating_sub(target)
        .max(target.saturating_sub(segment.end_frame))
}

fn attacker_is_airborne(events: &MatchEvents, attacker: u8, contact_frame: u32) -> bool {
    events.jumps.iter().any(|jump| {
        jump.side == attacker
            && jump.takeoff_confirmed
            && contact_frame
                >= jump
                    .frame
                    .saturating_add(crate::match_events::JUMP_C_ATK_MIN)
            && contact_frame <= jump.air_end
    })
}

pub fn match_strike_input(
    features: &[FrameFeatures],
    events: &MatchEvents,
    attacker: u8,
    contact: &ContactEvent,
    character: &str,
) -> Option<StrikeAttribution> {
    let observation = startup_observation(features, events, attacker, contact.frame);
    let airborne = attacker_is_airborne(events, attacker, contact.frame);
    let segments = &events.segments[attacker as usize - 1];
    let mut relevant: Vec<(&InputSegment, u32)> = segments
        .iter()
        .filter(|segment| {
            segment.start_frame <= contact.frame
                && segment.evidence.has_direct_observation()
                && segment.has_button()
                && !segment.throw
                && !segment.is_drive_impact()
                && !segment.badges.iter().any(|badge| badge == "DP")
        })
        .filter_map(|segment| {
            if let Some(startup) = observation {
                let distance = segment_distance(segment, startup.frame);
                (distance <= INPUT_START_LINK_WINDOW).then_some((segment, distance))
            } else {
                let age = contact.frame.saturating_sub(segment.start_frame);
                (age <= ATTACK_LOOKBACK as u32).then_some((segment, age))
            }
        })
        .collect();
    let best_distance = relevant.iter().map(|(_, distance)| *distance).min()?;
    relevant.retain(|(_, distance)| *distance == best_distance);

    let mut kinds = Vec::new();
    for (segment, _) in relevant {
        let observed_startup = observation.map(|value| value.startup);
        if let Some(kind) = frame_data::strike_kind_for_input(
            character,
            &segment.dir,
            &segment.badges,
            segment.auto,
            airborne,
            observed_startup,
        ) {
            kinds.push(kind);
        }
    }
    let kind = *kinds.first()?;
    kinds
        .iter()
        .all(|candidate| *candidate == kind)
        .then_some(StrikeAttribution {
            kind,
            confidence: if observation.is_some() {
                EventConfidence::High
            } else {
                EventConfidence::Medium
            },
        })
}
