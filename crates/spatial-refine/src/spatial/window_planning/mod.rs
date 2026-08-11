mod approaches;
mod jumps;
mod model;
mod punishes;
mod round_bounds;
mod teleports;

pub use model::{SpatialCandidateWindow, SpatialHintRange};

use crate::match_events::MatchEvents;

/// Builds candidate-driven second-pass windows from input and meter events.
pub fn spatial_candidate_windows(events: &MatchEvents) -> Vec<SpatialCandidateWindow> {
    let mut windows = teleports::windows(
        &events.teleports,
        &events.compound_threats,
        &events.jumps,
        &events.rounds,
    );
    windows.extend(punishes::windows(&events.punishes, &events.rounds));
    windows.extend(jumps::windows(&events.jumps, &events.rounds));
    windows.extend(approaches::throw_windows(
        &events.throw_actions,
        &events.rounds,
    ));
    windows.extend(approaches::drive_rush_windows(
        &events.drive_rushes,
        &events.rounds,
    ));
    merge_adjacent(windows)
}

fn merge_adjacent(mut windows: Vec<SpatialCandidateWindow>) -> Vec<SpatialCandidateWindow> {
    windows.sort_by_key(|window| window.start_frame);
    let mut merged: Vec<SpatialCandidateWindow> = Vec::new();
    for mut window in windows {
        match merged.last_mut() {
            Some(last) if window.start_frame <= last.end_frame.saturating_add(1) => {
                last.end_frame = last.end_frame.max(window.end_frame);
                last.teleport_hints.append(&mut window.teleport_hints);
                last.airborne_hints.append(&mut window.airborne_hints);
            }
            _ => merged.push(window),
        }
    }
    for window in &mut merged {
        window
            .teleport_hints
            .sort_by_key(|hint| (hint.start_frame, hint.side));
        window.teleport_hints.dedup();
        window
            .airborne_hints
            .sort_by_key(|hint| (hint.start_frame, hint.side));
        window.airborne_hints.dedup();
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_events::{
        EventConfidence, PunishChance, PunishOrigin, PunishOutcome, PunishReachability, RoundInfo,
        ThrowActionEvent, ThrowApproach, ThrowOutcome,
    };
    use crate::test_support::{jump, teleport};

    fn window(start_frame: u32, end_frame: u32, hint_side: u8) -> SpatialCandidateWindow {
        SpatialCandidateWindow {
            start_frame,
            end_frame,
            teleport_hints: vec![SpatialHintRange {
                side: hint_side,
                start_frame,
                end_frame,
            }],
            airborne_hints: vec![],
        }
    }

    fn round() -> RoundInfo {
        RoundInfo {
            round_no: 2,
            start_frame: 100,
            end_frame: 200,
            winner: None,
            p1_hp_end: 1.0,
            p2_hp_end: 1.0,
        }
    }

    #[test]
    fn exactly_adjacent_windows_merge_without_dropping_a_later_window() {
        let merged = merge_adjacent(vec![window(30, 40, 3), window(11, 20, 2), window(0, 10, 1)]);

        assert_eq!(merged.len(), 2);
        assert_eq!((merged[0].start_frame, merged[0].end_frame), (0, 20));
        assert_eq!(
            merged[0]
                .teleport_hints
                .iter()
                .map(|hint| hint.side)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!((merged[1].start_frame, merged[1].end_frame), (30, 40));
    }

    #[test]
    fn every_window_family_uses_round_bounds_and_teleports_use_jump_hints() {
        let rounds = [round()];

        let throw = ThrowActionEvent {
            thrower: 1,
            input_frame: 110,
            startup_frame: Some(112),
            active_frame: Some(190),
            outcome: ThrowOutcome::Hit,
            damage: 0.0,
            approach: ThrowApproach::Unknown,
            confidence: EventConfidence::High,
            round_no: 2,
        };
        let throw_windows = approaches::throw_windows(&[throw], &rounds);
        assert_eq!(
            (throw_windows[0].start_frame, throw_windows[0].end_frame),
            (100, 200)
        );

        let mut jump_event = jump(102, crate::match_events::JumpOutcome::GotHit, "U");
        jump_event.contact_frame = Some(199);
        jump_event.round_no = 2;
        let jump_windows = jumps::windows(&[jump_event], &rounds);
        assert_eq!(
            (jump_windows[0].start_frame, jump_windows[0].end_frame),
            (100, 200)
        );

        let punish = PunishChance {
            frame: 150,
            side: 1,
            advantage: 4,
            outcome: PunishOutcome::Missed,
            origin: PunishOrigin::BlockedMove,
            recovery_start_frame: 145,
            recovery_end_frame: 155,
            source_contact_frame: Some(110),
            attack_start_frame: Some(190),
            attack_active_frame: Some(195),
            reachability: PunishReachability::Unknown,
            punished_drop: 0.0,
            pressed: String::new(),
            round_no: 2,
        };
        let punish_windows = punishes::windows(&[punish], &rounds);
        assert_eq!(
            (punish_windows[0].start_frame, punish_windows[0].end_frame),
            (100, 200)
        );

        let mut teleport_event = teleport(105);
        teleport_event.round_no = 2;
        let mut overlapping_jump = jump(120, crate::match_events::JumpOutcome::GotHit, "U");
        overlapping_jump.side = teleport_event.attacker;
        overlapping_jump.air_end = 145;
        overlapping_jump.round_no = 2;
        let teleport_windows =
            teleports::windows(&[teleport_event], &[], &[overlapping_jump], &rounds);
        assert_eq!(teleport_windows[0].start_frame, 100);
        assert_eq!(
            teleport_windows[0].airborne_hints,
            [SpatialHintRange {
                side: 2,
                start_frame: 120,
                end_frame: 145,
            }]
        );
    }
}
