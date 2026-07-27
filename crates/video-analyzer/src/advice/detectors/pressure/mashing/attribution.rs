use crate::advice::{BIG_DAMAGE, MASH_PRESS_WINDOW};
use crate::match_events::{
    DamageEvent, EventConfidence, InputSegment, JumpOutcome, MatchEvents, MinusPressOutcome,
    ThreatOutcome, JUMP_ATTACK_MAX, JUMP_ATTACK_MIN, JUMP_SELF_HIT_MIN, JUMP_SELF_HIT_WINDOW,
    THREAT_DAMAGE_WINDOW,
};

pub(super) fn nearest_direct_press<'a>(
    segments: &'a [InputSegment],
    damage: &DamageEvent,
) -> Option<&'a InputSegment> {
    let nearest = segments
        .iter()
        .filter(|segment| {
            segment.has_button()
                && !segment.throw
                && segment.start_frame + MASH_PRESS_WINDOW >= damage.start_frame
                && segment.start_frame < damage.start_frame
        })
        .max_by_key(|segment| segment.start_frame)?;
    nearest.evidence.has_direct_observation().then_some(nearest)
}

pub(super) fn claimed_by_other_detector(
    events: &MatchEvents,
    own: u8,
    opponent: u8,
    damage: &DamageEvent,
) -> bool {
    debug_assert!(damage.drop >= BIG_DAMAGE);
    let jump = events.jumps.iter().any(|jump| {
        (jump.side == own
            && jump.takeoff_confirmed
            && jump.outcome == JumpOutcome::GotHit
            && damage.start_frame >= jump.frame + JUMP_SELF_HIT_MIN
            && damage.start_frame <= jump.air_end.max(jump.frame + JUMP_SELF_HIT_WINDOW))
            || (jump.side == opponent
                && jump.takeoff_confirmed
                && jump.outcome == JumpOutcome::LandedHit
                && damage.start_frame >= jump.frame + JUMP_ATTACK_MIN
                && damage.start_frame <= jump.frame + JUMP_ATTACK_MAX)
    });
    let compound = events.compound_threats.iter().any(|threat| {
        threat.defender == own
            && threat.outcome == ThreatOutcome::Hit
            && threat.damage > 0.0
            && damage.start_frame >= threat.followup_attack_frame
            && damage.start_frame
                <= threat
                    .followup_attack_frame
                    .saturating_add(THREAT_DAMAGE_WINDOW)
    });
    let reversal = events.reversals.iter().any(|event| {
        event.side == own
            && damage.start_frame + 5 >= event.frame
            && damage.start_frame <= event.frame + 105
    });
    let minus_press = events.presses_while_minus.iter().any(|event| {
        event.side == own
            && event.confidence == EventConfidence::High
            && event.outcome == MinusPressOutcome::CounterHit
            && damage.start_frame >= event.frame
            && damage.start_frame <= event.frame + 30
    });
    let drive_impact = events.drive_impacts.iter().any(|event| {
        event.side == own
            && event.confidence == EventConfidence::High
            && event.damage > 0.0
            && damage.start_frame >= event.input_frame
            && damage.start_frame <= event.input_frame.saturating_add(90)
    });
    jump || compound || reversal || minus_press || drive_impact
}
