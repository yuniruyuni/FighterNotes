use super::super::*;

pub(in crate::spatial::tests) fn empty_events() -> MatchEvents {
    MatchEvents {
        rounds: vec![RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: 399,
            winner: None,
            p1_hp_end: 1.0,
            p2_hp_end: 1.0,
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
        super_arts: vec![],
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
        hp: [vec![], vec![]],
    }
}

pub(in crate::spatial::tests) fn teleport(input_frame: u32) -> TeleportEvent {
    TeleportEvent {
        attacker: 2,
        defender: 1,
        input_frame,
        inv_start_frame: input_frame + 10,
        inv_end_frame: input_frame + 16,
        followup_attack_frame: Some(input_frame + 30),
        followup_contact_frame: Some(input_frame + 30),
        airborne: true,
        defender_actionable: true,
        context: TeleportContext::NakedAttack,
        response: None::<DefenseResponse>,
        outcome: ThreatOutcome::Hit,
        damage: 0.1,
        dp_reachability: DpReachability::Unknown,
        round_no: 1,
        confidence: 0.9,
    }
}

pub(in crate::spatial::tests) fn jump(
    frame: u32,
    outcome: JumpOutcome,
    input_dir: &str,
) -> crate::match_events::JumpEvent {
    crate::match_events::JumpEvent {
        side: 2,
        frame,
        outcome,
        input_dir: input_dir.to_string(),
        direction: JumpDirection::Unknown,
        contact_frame: Some(frame + 20),
        takeoff_confirmed: false,
        air_end: frame + 47,
        round_no: 1,
    }
}
