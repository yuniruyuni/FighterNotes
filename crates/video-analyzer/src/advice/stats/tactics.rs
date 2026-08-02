use super::super::*;
use crate::temporal::{SUPER_SPEND_CONFIRM_LOOKAHEAD, SUPER_SPEND_CONFIRM_SAMPLES};

const SUPER_STATS_MIN_COVERAGE_PERCENT: u64 = 70;
const SUPER_STATS_MAX_UNRELIABLE_RUN: usize =
    SUPER_SPEND_CONFIRM_LOOKAHEAD - SUPER_SPEND_CONFIRM_SAMPLES;

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
    let mut stats = TacticStats {
        super_art_stats_available: has_complete_super_coverage(features, &events.rounds, own),
        opponent_super_art_stats_available: has_complete_super_coverage(
            features,
            &events.rounds,
            opponent,
        ),
        ..TacticStats::default()
    };
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
            if let Some(evidence) = events.reliable_attack_evidence_for_super(event) {
                stats.super_damage_samples += 1;
                stats.super_reported_combo_damage += evidence.combo_damage;
                if let Some(marginal) = evidence.marginal_damage {
                    stats.super_reported_marginal_damage += marginal;
                }
                if !event.ko
                    && evidence
                        .entry_scaling_percent
                        .is_some_and(|percent| percent <= 50)
                {
                    stats.super_low_scaling_uses += 1;
                }
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

/// A detected spend proves that at least one SA occurred, but it does not prove
/// that no other spend was hidden. Missing features, non-match frames, and
/// uncertain reads are all gaps: compressing them out would turn a long blind
/// interval into adjacent observations.
///
/// The temporal cleaner needs 12 reliable lower-gauge samples within its next
/// 90 input frames. The dense timeline applies the same contract to every
/// round. Requiring the exact first and last round frames also makes a spend
/// too close to the end fail closed: an unconfirmed lower reading is marked
/// uncertain by the cleaner, and the round boundary is not implicit evidence.
fn has_complete_super_coverage(
    features: &[FrameFeatures],
    rounds: &[crate::match_events::RoundInfo],
    side: u8,
) -> bool {
    if rounds.is_empty() || !matches!(side, 1 | 2) {
        return false;
    }

    rounds.iter().all(|round| {
        if round.end_frame < round.start_frame {
            return false;
        }
        let expected_frames = u64::from(round.end_frame - round.start_frame) + 1;
        let round_feature_count = features
            .iter()
            .filter(|feature| {
                feature.frame_index >= round.start_frame && feature.frame_index <= round.end_frame
            })
            .count() as u64;
        if round_feature_count * 100 < expected_frames * SUPER_STATS_MIN_COVERAGE_PERCENT {
            return false;
        }
        let Ok(expected_frames) = usize::try_from(expected_frames) else {
            return false;
        };
        let mut reliable = vec![false; expected_frames];
        for feature in features.iter().filter(|feature| {
            feature.frame_index >= round.start_frame && feature.frame_index <= round.end_frame
        }) {
            let certain = feature.is_match_screen
                && match side {
                    1 => !feature.left_super_uncertain,
                    2 => !feature.right_super_uncertain,
                    _ => false,
                };
            if certain {
                reliable[(feature.frame_index - round.start_frame) as usize] = true;
            }
        }
        let reliable_count = reliable.iter().filter(|&&sample| sample).count();
        if reliable_count < SUPER_SPEND_CONFIRM_SAMPLES
            || (reliable_count as u64) * 100
                < (reliable.len() as u64) * SUPER_STATS_MIN_COVERAGE_PERCENT
        {
            return false;
        }
        if reliable.first() != Some(&true) || reliable.last() != Some(&true) {
            return false;
        }

        let mut unreliable_run = 0;
        for &sample in &reliable {
            if sample {
                unreliable_run = 0;
            } else {
                unreliable_run += 1;
                if unreliable_run > SUPER_STATS_MAX_UNRELIABLE_RUN {
                    return false;
                }
            }
        }

        reliable
            .windows(SUPER_SPEND_CONFIRM_LOOKAHEAD)
            .all(|window| {
                window.iter().filter(|&&sample| sample).count() >= SUPER_SPEND_CONFIRM_SAMPLES
            })
    })
}
