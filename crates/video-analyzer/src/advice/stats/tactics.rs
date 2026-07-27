use super::super::*;

pub(crate) fn build_tactic_stats(events: &MatchEvents, own: u8, opponent: u8) -> TacticStats {
    use crate::match_events::{
        BurnoutCause, DriveImpactOutcome, DriveRushOutcome, ThrowApproach, ThrowOutcome,
    };

    let event_in_round = |round_no: u32, frame: u32| {
        crate::match_events::round_of(&events.rounds, frame) == Some(round_no)
    };
    let mut stats = TacticStats::default();
    for jump in events.jumps.iter().filter(|jump| {
        event_in_round(jump.round_no, jump.frame)
            && jump.side == opponent
            && jump.takeoff_confirmed
            && jump.direction != JumpDirection::Backward
            && matches!(jump.outcome, JumpOutcome::GotHit | JumpOutcome::LandedHit)
    }) {
        stats.anti_air_opportunities += 1;
        match jump.outcome {
            JumpOutcome::GotHit => stats.anti_air_successes += 1,
            JumpOutcome::LandedHit => stats.jump_ins_allowed += 1,
            _ => {}
        }
    }

    for impact in events.drive_impacts.iter().filter(|impact| {
        event_in_round(impact.round_no, impact.input_frame) && impact.side == opponent
    }) {
        if impact.confidence != EventConfidence::High {
            stats.di_unconfirmed += 1;
            continue;
        }
        stats.di_faced += 1;
        match impact.outcome {
            crate::match_events::DriveImpactOutcome::Countered => stats.di_returned += 1,
            DriveImpactOutcome::Blocked => stats.di_blocked += 1,
            DriveImpactOutcome::Parried => stats.di_parried += 1,
            DriveImpactOutcome::Hit => stats.di_hit += 1,
            DriveImpactOutcome::Whiffed => stats.di_avoided += 1,
            DriveImpactOutcome::Unconfirmed => stats.di_unconfirmed += 1,
        }
    }

    for rush in events.drive_rushes.iter().filter(|rush| {
        event_in_round(rush.round_no, rush.frame) && rush.side == opponent && rush.raw
    }) {
        if rush.confidence != EventConfidence::High {
            stats.raw_drive_rushes_unconfirmed += 1;
            continue;
        }
        stats.raw_drive_rushes_faced += 1;
        match rush.outcome {
            DriveRushOutcome::Hit => stats.raw_drive_rushes_hit += 1,
            DriveRushOutcome::Blocked | DriveRushOutcome::Stopped | DriveRushOutcome::NoContact => {
                stats.raw_drive_rushes_defended += 1
            }
            DriveRushOutcome::Unconfirmed => stats.raw_drive_rushes_unconfirmed += 1,
        }
    }

    stats.dash_throws_faced = events
        .throw_actions
        .iter()
        .filter(|throw| {
            event_in_round(throw.round_no, throw.input_frame)
                && throw.thrower == opponent
                && throw.confidence == EventConfidence::High
                && throw.approach == ThrowApproach::ForwardDash
                && throw.outcome == ThrowOutcome::Hit
        })
        .count() as u32;
    stats.throw_whiffs = events
        .throw_actions
        .iter()
        .filter(|throw| {
            event_in_round(throw.round_no, throw.input_frame)
                && throw.thrower == own
                && throw.confidence == EventConfidence::High
                && throw.outcome == ThrowOutcome::ExecutedWhiff
        })
        .count() as u32;

    stats.minus_defense_opportunities = events
        .minus_situations
        .iter()
        .filter(|situation| {
            event_in_round(situation.round_no, situation.frame)
                && situation.side == own
                && situation.confidence == EventConfidence::High
        })
        .count() as u32;

    for challenge in events.presses_while_minus.iter().filter(|challenge| {
        event_in_round(challenge.round_no, challenge.frame)
            && challenge.side == own
            && challenge.confidence == EventConfidence::High
    }) {
        match challenge.action_kind {
            DefensiveActionKind::Strike => {
                stats.fastest_strike_challenges += 1;
                if challenge.outcome == MinusPressOutcome::CounterHit {
                    stats.fastest_strike_losses += 1;
                }
            }
            DefensiveActionKind::Throw => {
                stats.fastest_throw_challenges += 1;
                if challenge.outcome == MinusPressOutcome::CounterHit {
                    stats.fastest_throw_losses += 1;
                }
            }
        }
    }

    for burnout in events.burnouts.iter().filter(|burnout| {
        event_in_round(burnout.round_no, burnout.start_frame) && burnout.side == own
    }) {
        stats.burnout_count += 1;
        stats.burnout_seconds +=
            burnout.end_frame.saturating_sub(burnout.start_frame) as f32 / 60.0;
        stats.burnout_hp_lost += burnout.hp_lost;
        stats.burnout_hp_dealt += burnout.hp_dealt;
        match burnout.cause {
            BurnoutCause::SelfInitiated => stats.burnout_self_initiated += 1,
            BurnoutCause::ForcedByGuard => stats.burnout_forced += 1,
            BurnoutCause::Mixed => stats.burnout_mixed += 1,
            BurnoutCause::Unknown => stats.burnout_unknown += 1,
        }
    }
    stats
}
