use crate::match_events::{MatchEvents, RoundInfo};

pub(super) fn empty_events() -> MatchEvents {
    MatchEvents {
        rounds: vec![RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: 5_999,
            winner: Some(2),
            p1_hp_end: 0.0,
            p2_hp_end: 0.5,
        }],
        damage: vec![],
        jumps: vec![],
        throws: vec![],
        throw_actions: vec![],
        drive_impacts: vec![],
        drive_rushes: vec![],
        burnouts: vec![],
        contacts: vec![],
        punishes: vec![],
        reversals: vec![],
        guard_breaks: vec![],
        presses_while_minus: vec![],
        minus_situations: vec![],
        projectiles: vec![],
        teleports: vec![],
        compound_threats: vec![],
        meter_state: [vec![], vec![]],
        meter_confidence: [vec![], vec![]],
        meter_game_frame: [vec![], vec![]],
        segments: [vec![], vec![]],
        hp: [vec![1.0; 6_000], vec![1.0; 6_000]],
    }
}
