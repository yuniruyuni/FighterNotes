use super::super::*;
use super::support::*;

#[test]
fn projectile_and_teleport_remain_independent_compound_threats() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let p1 = timeline("left", &[(160, 174, "parry"), (175, 180, "stun")]);
    let p2 = timeline(
        "right",
        &[
            (100, 150, "projectile_active"),
            (170, 176, "inv_full"),
            (190, 195, "active"),
        ],
    );
    let meter = [state_per_frame(&p1, 400), state_per_frame(&p2, 400)];
    let segments = [vec![], vec![teleport_segment(160)]];
    let jumps = vec![JumpEvent {
        side: 2,
        frame: 155,
        outcome: JumpOutcome::Neutral,
        input_dir: "U".to_string(),
        direction: crate::JumpDirection::Neutral,
        contact_frame: None,
        takeoff_confirmed: true,
        air_end: 210,
        round_no: 1,
    }];
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 200,
        end_frame: 205,
        pre_freeze_frame: 200,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    }];
    let contacts = vec![ContactEvent {
        frame: 190,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    }];

    let (projectiles, teleports, compounds) = extract_test_threats!(
        &features,
        [&p1, &p2],
        &meter,
        &segments,
        &jumps,
        &contacts,
        &damage,
        &[round()],
        [Some("BLANKA"), Some("DHALSIM")],
    );

    assert_eq!(projectiles.len(), 1);
    assert_eq!(projectiles[0].observed_end_frame, 150);
    assert!(projectiles[0].threat_end_frame >= 170);
    assert_eq!(teleports.len(), 1);
    assert_eq!(teleports[0].context, TeleportContext::ProjectileCovered);
    assert!(teleports[0].airborne);
    assert!(teleports[0].defender_actionable);
    assert_eq!(teleports[0].damage, 0.1);
    assert_eq!(teleports[0].outcome, ThreatOutcome::Hit);
    assert!(teleports[0].response.is_none());
    assert_eq!(
        compounds[0]
            .projectile_response
            .as_ref()
            .map(|response| response.kind),
        Some(DefenseResponseKind::Parry)
    );
    assert_eq!(compounds.len(), 1);
}

#[test]
fn later_contact_owns_delayed_damage() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let p1 = timeline("left", &[(160, 174, "parry"), (175, 180, "stun")]);
    let p2 = timeline(
        "right",
        &[
            (100, 150, "projectile_active"),
            (170, 176, "inv_full"),
            (190, 195, "active"),
        ],
    );
    let meter = [state_per_frame(&p1, 400), state_per_frame(&p2, 400)];
    let contacts = vec![
        ContactEvent {
            frame: 190,
            attacker: 2,
            victim: 1,
            hit: true,
            projectile: false,
            round_no: 1,
        },
        ContactEvent {
            frame: 199,
            attacker: 2,
            victim: 1,
            hit: true,
            projectile: false,
            round_no: 1,
        },
    ];
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 200,
        end_frame: 205,
        pre_freeze_frame: 200,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    }];

    let (_, teleports, compounds) = extract_test_threats!(
        &features,
        [&p1, &p2],
        &meter,
        &[vec![], vec![teleport_segment(160)]],
        &[],
        &contacts,
        &damage,
        &[round()],
        [Some("BLANKA"), Some("DHALSIM")],
    );

    assert_eq!(teleports[0].outcome, ThreatOutcome::Defended);
    assert_eq!(teleports[0].damage, 0.0);
    assert_eq!(compounds[0].outcome, ThreatOutcome::Defended);
}

/// P1 側の projectile/teleport と P2 側の parry/stun も同じ結線を通る。
/// 左右を入れ替えても各 state run と response の side が落ちないことを固定する。
#[test]
fn player_one_compound_threat_keeps_player_two_responses() {
    let features: Vec<_> = (0..400).map(feature).collect();
    let p1 = timeline(
        "left",
        &[
            (100, 150, "projectile_active"),
            (170, 176, "inv_full"),
            (190, 195, "active"),
        ],
    );
    let p2 = timeline(
        "right",
        &[(160, 174, "parry"), (175, 180, "stun"), (188, 192, "parry")],
    );
    let meter = [state_per_frame(&p1, 400), state_per_frame(&p2, 400)];
    let contacts = vec![ContactEvent {
        frame: 190,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    let (projectiles, teleports, compounds) = extract_test_threats!(
        &features,
        [&p1, &p2],
        &meter,
        &[vec![teleport_segment(160)], vec![]],
        &[],
        &contacts,
        &[],
        &[round()],
        [Some("DHALSIM"), Some("BLANKA")],
    );

    assert_eq!(projectiles.len(), 1);
    assert_eq!(projectiles[0].owner, 1);
    assert_eq!(projectiles[0].contact_frame, Some(175));
    assert_eq!(teleports.len(), 1);
    assert_eq!(teleports[0].followup_contact_frame, Some(190));
    assert_eq!(teleports[0].outcome, ThreatOutcome::Defended);
    assert_eq!(
        teleports[0]
            .response
            .as_ref()
            .map(|response| (response.side, response.kind)),
        Some((2, DefenseResponseKind::Parry))
    );
    assert_eq!(compounds.len(), 1);
}
