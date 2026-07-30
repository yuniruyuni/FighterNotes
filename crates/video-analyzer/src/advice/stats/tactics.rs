use super::super::*;

pub(crate) fn build_tactic_stats(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    opponent: u8,
) -> TacticStats {
    use crate::match_events::{
        BurnoutCause, DriveImpactOutcome, DriveRushOutcome, SuperArtContext, SuperArtOutcome,
        ThrowApproach, ThrowOutcome,
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
    for event in events.super_arts.iter().filter(|event| {
        event_in_round(event.round_no, event.frame) && event.confidence != EventConfidence::Low
    }) {
        let own_event = event.side == own;
        match (own_event, event.level, event.critical_art) {
            (true, 1, _) => stats.sa1_used += 1,
            (true, 2, _) => stats.sa2_used += 1,
            (true, 3, true) => stats.ca_used += 1,
            (true, 3, false) => stats.sa3_used += 1,
            (false, 1, _) if event.side == opponent => stats.opponent_sa1_used += 1,
            (false, 2, _) if event.side == opponent => stats.opponent_sa2_used += 1,
            (false, 3, true) if event.side == opponent => stats.opponent_ca_used += 1,
            (false, 3, false) if event.side == opponent => stats.opponent_sa3_used += 1,
            _ => {}
        }
        if own_event {
            match event.outcome {
                SuperArtOutcome::Hit => stats.super_hits += 1,
                SuperArtOutcome::Blocked => stats.super_blocked += 1,
                SuperArtOutcome::NoImmediateContact => stats.super_no_immediate_contact += 1,
                SuperArtOutcome::Unconfirmed => {}
            }
            stats.super_punished += u32::from(event.punished);
            stats.super_kos += u32::from(event.ko);
            match event.context {
                SuperArtContext::Combo => stats.super_combo_uses += 1,
                SuperArtContext::Punish => stats.super_punish_uses += 1,
                SuperArtContext::DefensiveReversal => stats.super_reversal_uses += 1,
                SuperArtContext::Neutral => stats.super_neutral_uses += 1,
                SuperArtContext::Unknown => {}
            }
        } else if event.side == opponent {
            match event.outcome {
                SuperArtOutcome::Hit => stats.opponent_super_hits += 1,
                SuperArtOutcome::Blocked => stats.opponent_super_blocked += 1,
                SuperArtOutcome::NoImmediateContact => {
                    stats.opponent_super_no_immediate_contact += 1
                }
                SuperArtOutcome::Unconfirmed => {}
            }
            stats.opponent_super_punished += u32::from(event.punished);
            stats.opponent_super_kos += u32::from(event.ko);
        }
    }

    let last_round_end = events.rounds.last().map(|round| round.end_frame);
    if let Some(end_frame) = last_round_end {
        if let Some(feature) = features
            .iter()
            .rev()
            .find(|feature| feature.is_match_screen && feature.frame_index <= end_frame)
        {
            let own_is_left = own == 1;
            stats.super_gauge_end = if own_is_left {
                feature.left_super_value
            } else {
                feature.right_super_value
            };
            stats.opponent_super_gauge_end = if own_is_left {
                feature.right_super_value
            } else {
                feature.left_super_value
            };
        }
    }
    stats
}
