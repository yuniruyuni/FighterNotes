use super::super::candidate::{contact_matches, offer, starts_in, Candidate};
use crate::advice::DamageOrigin;
use crate::match_events::{DamageEvent, DriveImpactOutcome, DriveRushOutcome, MatchEvents};

const DRIVE_IMPACT_RESULT_WINDOW: u32 = 80;
const DRIVE_RUSH_RESULT_WINDOW: u32 = 90;

pub(super) fn offer_drive_impacts(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for impact in events.drive_impacts.iter().filter(|impact| {
        impact.round_no == damage.round_no
            && impact.damage > 0.0
            && ((impact.side == opponent && impact.outcome == DriveImpactOutcome::Hit)
                || (impact.side == own && impact.outcome == DriveImpactOutcome::Countered))
    }) {
        let anchor = impact
            .contact_frame
            .or(impact.active_frame)
            .unwrap_or(impact.input_frame);
        if starts_in(
            damage,
            anchor.saturating_sub(2),
            anchor.saturating_add(DRIVE_IMPACT_RESULT_WINDOW),
        ) {
            offer(
                candidate,
                DamageOrigin::DriveImpact,
                impact.confidence,
                85,
                anchor,
                damage,
            );
        }
    }
}

pub(super) fn offer_drive_rushes(
    candidate: &mut Option<Candidate>,
    events: &MatchEvents,
    own: u8,
    damage: &DamageEvent,
) {
    let opponent = 3 - own;
    for rush in events.drive_rushes.iter().filter(|rush| {
        rush.side == opponent
            && rush.round_no == damage.round_no
            && rush.raw
            && rush.outcome == DriveRushOutcome::Hit
            && rush.damage > 0.0
    }) {
        let anchor = rush.contact_frame.unwrap_or(rush.frame);
        if contact_matches(damage, anchor)
            || starts_in(
                damage,
                rush.frame,
                rush.frame.saturating_add(DRIVE_RUSH_RESULT_WINDOW),
            )
        {
            offer(
                candidate,
                DamageOrigin::RawDriveRush,
                rush.confidence,
                80,
                anchor,
                damage,
            );
        }
    }
}
