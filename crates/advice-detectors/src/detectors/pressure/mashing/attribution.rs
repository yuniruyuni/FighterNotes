use crate::match_events::{
    DamageEvent, DriveImpactOutcome, DriveRushOutcome, EventConfidence, InputSegment, JumpOutcome,
    MatchEvents, MinusPressOutcome, ThreatOutcome, JUMP_ATTACK_MAX, JUMP_ATTACK_MIN,
    JUMP_SELF_HIT_MIN, JUMP_SELF_HIT_WINDOW, THREAT_DAMAGE_WINDOW,
};
use crate::MASH_PRESS_WINDOW;

pub fn nearest_direct_press<'a>(
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

pub fn claimed_by_other_detector(
    events: &MatchEvents,
    own: u8,
    opponent: u8,
    damage: &DamageEvent,
) -> bool {
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
        event.confidence == EventConfidence::High
            && event.damage > 0.0
            && ((event.side == own && event.outcome == DriveImpactOutcome::Countered)
                || (event.side == opponent && event.outcome == DriveImpactOutcome::Hit))
            && damage.start_frame >= event.input_frame
            && damage.start_frame <= event.input_frame.saturating_add(90)
    });
    // 生ラッシュ同士の衝突など、自分の前進行動が止められた被弾は
    // 「守勢でボタンを押した」場面ではない。空間確定前でも、パリィ始動・
    // 自分が接触の被害側・HP 低下が同じ接触へ結び付く条件をすべて要求する。
    let own_raw_drive_rush = events.drive_rushes.iter().any(|rush| {
        rush.side == own
            && rush.raw
            && rush.outcome == DriveRushOutcome::Stopped
            && rush.damage > 0.0
            && rush.round_no == damage.round_no
            && rush.contact_frame.is_some_and(|contact_frame| {
                damage.start_frame.saturating_add(5) >= contact_frame
                    && damage.start_frame <= contact_frame.saturating_add(25)
            })
    });
    jump || compound || reversal || minus_press || drive_impact || own_raw_drive_rush
}
