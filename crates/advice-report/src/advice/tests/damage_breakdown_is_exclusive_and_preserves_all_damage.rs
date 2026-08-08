use super::support::*;

#[test]
fn damage_breakdown_is_exclusive_and_preserves_all_damage() {
    use crate::match_events::{
        BurnoutCause, ContactEvent, DriveImpactEvent, DriveImpactOutcome, DriveRushEvent,
        DriveRushOutcome, GuardBreakEvent, JumpDirection, ThrowActionEvent, ThrowApproach,
        ThrowOutcome,
    };

    let mut ev = empty_events();
    let damage = |frame: u32, drop: f32| DamageEvent {
        victim: 1,
        start_frame: frame,
        pre_freeze_frame: frame,
        end_frame: frame + 12,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    };
    ev.damage = vec![
        damage(120, 0.12),
        damage(320, 0.18),
        damage(390, 0.02),
        damage(520, 0.16),
        damage(720, 0.10),
        damage(900, 0.08),
        damage(1100, 0.06),
        damage(1300, 0.04),
    ];
    ev.jumps.push(JumpEvent {
        side: 1,
        frame: 80,
        outcome: JumpOutcome::GotHit,
        input_dir: "UR".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(115),
        takeoff_confirmed: true,
        air_end: 130,
        round_no: 1,
    });
    ev.throw_actions.push(ThrowActionEvent {
        thrower: 2,
        input_frame: 295,
        startup_frame: Some(298),
        active_frame: Some(300),
        outcome: ThrowOutcome::Hit,
        damage: 0.20,
        approach: ThrowApproach::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    ev.drive_impacts.push(DriveImpactEvent {
        side: 2,
        input_frame: 480,
        active_frame: Some(500),
        contact_frame: Some(500),
        outcome: DriveImpactOutcome::Hit,
        damage: 0.16,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    ev.drive_rushes.push(DriveRushEvent {
        side: 2,
        frame: 670,
        raw: true,
        outcome: DriveRushOutcome::Hit,
        contact_frame: Some(700),
        damage: 0.10,
        confidence: EventConfidence::Medium,
        round_no: 1,
    });
    ev.contacts = vec![
        ContactEvent {
            frame: 115,
            attacker: 2,
            victim: 1,
            hit: true,
            projectile: true,
            round_no: 1,
        },
        ContactEvent {
            frame: 895,
            attacker: 2,
            victim: 1,
            hit: true,
            projectile: true,
            round_no: 1,
        },
        ContactEvent {
            frame: 1095,
            attacker: 2,
            victim: 1,
            hit: true,
            projectile: false,
            round_no: 1,
        },
    ];
    ev.guard_breaks.push(GuardBreakEvent {
        side: 1,
        frame: 1100,
        drop: 0.06,
        guard_dir: "L".to_string(),
        broke_to: "N".to_string(),
        round_no: 1,
    });
    ev.burnouts.push(BurnoutPeriod {
        side: 1,
        start_frame: 850,
        end_frame: 950,
        hp_lost: 0.08,
        hp_dealt: 0.0,
        cause: BurnoutCause::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    });

    let breakdown = super::damage_origins::build_damage_breakdown(&[], &ev, 1, None);
    let origins: Vec<_> = breakdown.events.iter().map(|event| event.origin).collect();
    assert_eq!(
        origins,
        [
            DamageOrigin::OwnJumpCaught,
            DamageOrigin::Throw,
            DamageOrigin::Throw,
            DamageOrigin::DriveImpact,
            DamageOrigin::RawDriveRush,
            DamageOrigin::Projectile,
            DamageOrigin::Strike,
            DamageOrigin::Unclassified,
        ]
    );
    assert_eq!(
        breakdown
            .events
            .iter()
            .map(|event| event.sequence_no)
            .collect::<Vec<_>>(),
        (1..=8).collect::<Vec<_>>()
    );
    assert!((breakdown.total_hp_lost - 0.76).abs() < 1e-6);
    assert!((breakdown.classified_hp_lost - 0.72).abs() < 1e-6);
    assert!(breakdown.events[5]
        .contexts
        .contains(&DamageContext::Burnout));
    assert!(breakdown.events[6]
        .contexts
        .contains(&DamageContext::GuardBreak));
}
