use crate::advice::{PRESSURE_DMG_WINDOW, PRESSURE_DRIVE_DROP, PRESSURE_DRIVE_WINDOW};
use crate::frame_features::FrameFeatures;
use crate::match_events::{DamageEvent, MatchEvents};

fn drive_was_spent(features: &[FrameFeatures], own_index: usize, frame: u32) -> bool {
    if features.is_empty() {
        return false;
    }
    let end = features
        .binary_search_by_key(&frame, |feature| feature.frame_index)
        .unwrap_or_else(|index| index.min(features.len().saturating_sub(1)));
    let start = end.saturating_sub(PRESSURE_DRIVE_WINDOW);
    let drive = |feature: &FrameFeatures| {
        if own_index == 0 {
            feature.left_drive_ratio
        } else {
            feature.right_drive_ratio
        }
    };
    let burnout = |feature: &FrameFeatures| {
        if own_index == 0 {
            feature.left_burnout
        } else {
            feature.right_burnout
        }
    };
    !burnout(&features[start])
        && !burnout(&features[end])
        && drive(&features[start]) - drive(&features[end]) >= PRESSURE_DRIVE_DROP
}

pub(super) fn is_pressured(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    own_index: usize,
    damage: &DamageEvent,
) -> bool {
    let recent_damage = events.damage.iter().any(|previous| {
        previous.victim == own
            && previous.end_frame < damage.start_frame
            && previous.end_frame + PRESSURE_DMG_WINDOW >= damage.start_frame
    });
    let recent_block = events.contacts.iter().any(|contact| {
        contact.victim == own
            && !contact.hit
            && !contact.projectile
            && contact.frame < damage.start_frame
            && contact.frame + PRESSURE_DMG_WINDOW >= damage.start_frame
    });
    if events.meter_state[0].is_empty() {
        recent_damage || drive_was_spent(features, own_index, damage.start_frame)
    } else {
        recent_damage || recent_block
    }
}
