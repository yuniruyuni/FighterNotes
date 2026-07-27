use super::support::*;

#[test]
fn throw_action_links_delayed_damage_and_deduplicates_held_input() {
    use MeterState::*;
    let n = 260usize;
    let mut p1 = vec![Free; n];
    let mut p2 = vec![Free; n];
    p1[100..105].fill(Startup);
    p1[105..111].fill(Active);
    p2[105..111].fill(Stun);
    let damage = vec![DamageEvent {
        victim: 2,
        start_frame: 170,
        pre_freeze_frame: 170,
        end_frame: 185,
        hp_before: 1.0,
        hp_after: 0.88,
        drop: 0.12,
        round_no: 1,
    }];
    let throw = |frame| InputSegment {
        start_frame: frame,
        end_frame: frame + 3,
        dir: "N".to_string(),
        badges: vec![],
        auto: false,
        throw: true,
        evidence: Default::default(),
    };
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: n as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 0.88,
    }];
    let events = super::actions::extract_throw_actions(
        &[p1, p2],
        &[vec![0; n], vec![0; n]],
        &[],
        &damage,
        &[vec![throw(100), throw(108)], vec![]],
        &rounds,
    );
    assert_eq!(events.len(), 1, "保持入力を別の投げにしない: {events:?}");
    assert_eq!(events[0].outcome, ThrowOutcome::Hit);
    assert_eq!(events[0].confidence, EventConfidence::High);
    assert!((events[0].damage - 0.12).abs() < 1e-6);
}
