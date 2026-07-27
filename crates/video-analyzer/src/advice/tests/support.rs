pub(super) use super::super::*;
pub(super) use crate::match_events::{
    BurnoutPeriod, CompoundThreat, DamageEvent, DefenseResponse, JumpEvent, PunishChance,
    PunishOrigin, RoundInfo, TeleportEvent, ThrowEvent,
};

pub(super) fn empty_events() -> MatchEvents {
    MatchEvents {
        rounds: vec![RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: 5999,
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
        hp: [vec![1.0; 6000], vec![1.0; 6000]],
    }
}

pub(super) fn assert_invites_user_review(card: &AdviceCard) {
    assert_eq!(
        OBSERVATION_REVIEW_CAVEAT,
        "断定できませんが、検討の対象にしてもよいかもしれません"
    );
    assert!(
        card.description.contains(OBSERVATION_REVIEW_CAVEAT),
        "確認場面が利用者の検討を促していない: {}",
        card.description
    );
}

pub(super) fn basic_mashing_events() -> MatchEvents {
    use crate::match_events::InputSegment;

    let mut events = empty_events();
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 880,
        pre_freeze_frame: 880,
        end_frame: 900,
        hp_before: 1.0,
        hp_after: 0.96,
        drop: 0.04,
        round_no: 1,
    });
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1000,
        pre_freeze_frame: 1000,
        end_frame: 1020,
        hp_before: 0.9,
        hp_after: 0.78,
        drop: 0.12,
        round_no: 1,
    });
    events.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1200,
        pre_freeze_frame: 1200,
        end_frame: 1220,
        hp_before: 0.78,
        hp_after: 0.66,
        drop: 0.12,
        round_no: 1,
    });
    let press = |start_frame| InputSegment {
        start_frame,
        end_frame: start_frame + 5,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    events.segments[0] = vec![press(990), press(1190)];
    events
}
