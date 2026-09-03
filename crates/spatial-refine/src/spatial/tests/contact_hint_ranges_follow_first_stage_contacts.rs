use super::*;
use crate::match_events::ContactEvent;

/// window にかかる contact だけが hitstop のヒント区間になり、window の
/// 境界で切り詰められる。window 外の contact はヒントを作らない。
#[test]
fn contact_hint_ranges_follow_first_stage_contacts() {
    let mut events = empty_events();
    events.punishes.push(PunishChance {
        frame: 200,
        side: 1,
        advantage: 4,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 196,
        recovery_end_frame: 203,
        source_contact_frame: Some(195),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    });
    let contact = |frame: u32| ContactEvent {
        frame,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    };
    // window は 159..208。195 は内側、205 は末尾で切り詰め、300 は外。
    events.contacts.push(contact(195));
    events.contacts.push(contact(205));
    events.contacts.push(contact(300));

    let windows = spatial_candidate_windows(&events);
    assert_eq!(windows.len(), 1);
    let hints: Vec<(u32, u32)> = windows[0]
        .contact_hints
        .iter()
        .map(|range| (range.start_frame, range.end_frame))
        .collect();
    assert_eq!(hints, [(195, 205), (205, 208)]);
}
