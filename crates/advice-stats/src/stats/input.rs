use super::super::*;

pub fn build_input_stats(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    own_i: usize,
) -> Option<InputStats> {
    let event_in_round = |round_no: u32, frame: u32| {
        crate::match_events::round_of(&events.rounds, frame) == Some(round_no)
    };
    let overlap_frames = |start: u32, end: u32| -> u32 {
        events
            .rounds
            .iter()
            .map(|round| {
                let a = start.max(round.start_frame);
                let b = end.min(round.end_frame);
                if a <= b {
                    b - a + 1
                } else {
                    0
                }
            })
            .sum()
    };
    let segs: Vec<_> = events.segments[own_i]
        .iter()
        .filter(|segment| overlap_frames(segment.start_frame, segment.end_frame) > 0)
        .collect();
    if segs.is_empty() {
        return None;
    }
    let match_frames = features
        .iter()
        .filter(|f| {
            f.is_match_screen
                && crate::match_events::round_of(&events.rounds, f.frame_index).is_some()
        })
        .count();
    let minutes = match_frames as f32 / 3600.0;

    let jumps = events
        .jumps
        .iter()
        .filter(|j| j.side == own && j.takeoff_confirmed && event_in_round(j.round_no, j.frame))
        .count() as u32;
    let jump_got_hit = events
        .jumps
        .iter()
        .filter(|j| {
            j.side == own
                && j.takeoff_confirmed
                && j.outcome == JumpOutcome::GotHit
                && event_in_round(j.round_no, j.frame)
        })
        .count() as u32;
    let jump_landed = events
        .jumps
        .iter()
        .filter(|j| {
            j.side == own
                && j.takeoff_confirmed
                && j.outcome == JumpOutcome::LandedHit
                && event_in_round(j.round_no, j.frame)
        })
        .count() as u32;
    let throw_attempts = events
        .throws
        .iter()
        .filter(|t| t.thrower == own && event_in_round(t.round_no, t.frame))
        .count() as u32;
    let throw_hits = events
        .throws
        .iter()
        .filter(|t| t.thrower == own && t.connected && event_in_round(t.round_no, t.frame))
        .count() as u32;
    let button_presses = segs.iter().filter(|s| s.has_button()).count() as u32;
    let auto_presses = segs.iter().filter(|s| s.auto).count() as u32;
    // Modern は DI 箱バッジ、クラシックは強P+強K 同時押しで DI
    let di_presses = segs
        .iter()
        .filter(|s| {
            s.badges.iter().any(|b| b == "DI")
                || (s.badges.iter().any(|b| b == "強P") && s.badges.iter().any(|b| b == "強K"))
        })
        .count() as u32;
    let crouch_frames: u32 = segs
        .iter()
        .filter(|s| matches!(s.dir.as_str(), "D" | "DL" | "DR"))
        .map(|s| overlap_frames(s.start_frame, s.end_frame))
        .sum();

    Some(InputStats {
        total_inputs: segs.len() as u32,
        minutes,
        jumps,
        jumps_per_min: if minutes > 0.0 {
            jumps as f32 / minutes
        } else {
            0.0
        },
        jump_got_hit,
        jump_landed,
        throw_attempts,
        throw_hits,
        button_presses,
        auto_presses,
        auto_ratio: if button_presses > 0 {
            auto_presses as f32 / button_presses as f32
        } else {
            0.0
        },
        di_presses,
        crouch_ratio: if match_frames > 0 {
            crouch_frames as f32 / match_frames as f32
        } else {
            0.0
        },
    })
}
