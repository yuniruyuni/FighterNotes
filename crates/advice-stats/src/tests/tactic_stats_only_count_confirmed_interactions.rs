use super::support::*;

#[test]
fn tactic_stats_only_count_confirmed_interactions() {
    use crate::match_events::{
        BurnoutCause, DriveImpactEvent, DriveImpactOutcome, DriveRushEvent, DriveRushOutcome,
        JumpDirection, JumpEvent, MinusPressEvent, MinusPressOutcome, MinusSituationEvent,
        ThrowActionEvent, ThrowApproach, ThrowOutcome,
    };
    let mut ev = empty_events();
    let jump = |frame, outcome| JumpEvent {
        side: 2,
        frame,
        outcome,
        input_dir: "UL".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(frame + 20),
        takeoff_confirmed: true,
        air_end: frame + 44,
        round_no: 1,
    };
    ev.jumps = vec![
        jump(100, JumpOutcome::GotHit),
        jump(200, JumpOutcome::LandedHit),
        jump(300, JumpOutcome::Neutral),
    ];
    let impact = |frame, outcome, confidence| DriveImpactEvent {
        side: 2,
        input_frame: frame,
        active_frame: Some(frame + 10),
        contact_frame: Some(frame + 12),
        outcome,
        damage: 0.0,
        confidence,
        round_no: 1,
    };
    ev.drive_impacts = vec![
        impact(400, DriveImpactOutcome::Countered, EventConfidence::High),
        impact(500, DriveImpactOutcome::Hit, EventConfidence::High),
        impact(
            600,
            DriveImpactOutcome::Unconfirmed,
            EventConfidence::Medium,
        ),
    ];
    ev.drive_rushes = vec![
        DriveRushEvent {
            side: 2,
            frame: 700,
            raw: true,
            outcome: DriveRushOutcome::Blocked,
            contact_frame: Some(730),
            damage: 0.0,
            confidence: EventConfidence::High,
            round_no: 1,
        },
        DriveRushEvent {
            side: 2,
            frame: 800,
            raw: true,
            outcome: DriveRushOutcome::Unconfirmed,
            contact_frame: None,
            damage: 0.0,
            confidence: EventConfidence::Medium,
            round_no: 1,
        },
    ];
    ev.throw_actions = vec![
        ThrowActionEvent {
            thrower: 2,
            input_frame: 900,
            startup_frame: Some(902),
            active_frame: Some(907),
            outcome: ThrowOutcome::Hit,
            damage: 0.12,
            approach: ThrowApproach::ForwardDash,
            confidence: EventConfidence::High,
            round_no: 1,
        },
        ThrowActionEvent {
            thrower: 1,
            input_frame: 1000,
            startup_frame: Some(1002),
            active_frame: Some(1007),
            outcome: ThrowOutcome::ExecutedWhiff,
            damage: 0.0,
            approach: ThrowApproach::Unknown,
            confidence: EventConfidence::High,
            round_no: 1,
        },
        // 認識デバッグ用に残ったラウンド外イベントは統計へ混ぜない。
        ThrowActionEvent {
            thrower: 1,
            input_frame: 6100,
            startup_frame: Some(6102),
            active_frame: Some(6107),
            outcome: ThrowOutcome::ExecutedWhiff,
            damage: 0.0,
            approach: ThrowApproach::Unknown,
            confidence: EventConfidence::High,
            // 番号だけは有効でも、フレームがそのラウンド外なら除外する。
            round_no: 1,
        },
    ];
    ev.presses_while_minus = vec![
        MinusPressEvent {
            side: 1,
            frame: 1100,
            minus_frames: 2,
            pressed: "弱".to_string(),
            action_kind: DefensiveActionKind::Strike,
            outcome: MinusPressOutcome::CounterHit,
            drop: 0.08,
            confidence: EventConfidence::High,
            source_contact_frame: 1080,
            round_no: 1,
        },
        MinusPressEvent {
            side: 1,
            frame: 1200,
            minus_frames: 1,
            pressed: "投げ".to_string(),
            action_kind: DefensiveActionKind::Throw,
            outcome: MinusPressOutcome::GotAway,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: 1180,
            round_no: 1,
        },
    ];
    ev.minus_situations = ev
        .presses_while_minus
        .iter()
        .map(|event| MinusSituationEvent {
            side: event.side,
            frame: event.frame,
            minus_frames: event.minus_frames,
            fastest_action: Some(event.action_kind),
            action_frame: Some(event.frame),
            pressed: event.pressed.clone(),
            outcome: Some(event.outcome),
            drop: event.drop,
            confidence: event.confidence,
            source_contact_frame: event.source_contact_frame,
            round_no: event.round_no,
        })
        .chain(std::iter::once(MinusSituationEvent {
            side: 1,
            frame: 1250,
            minus_frames: 3,
            fastest_action: None,
            action_frame: None,
            pressed: String::new(),
            outcome: None,
            drop: 0.0,
            confidence: EventConfidence::High,
            source_contact_frame: 1230,
            round_no: 1,
        }))
        .collect();
    ev.burnouts.push(BurnoutPeriod {
        side: 1,
        start_frame: 1300,
        end_frame: 1420,
        hp_lost: 0.2,
        hp_dealt: 0.1,
        cause: BurnoutCause::ForcedByGuard,
        confidence: EventConfidence::High,
        round_no: 1,
    });

    let stats = build_tactic_stats(&[], &ev, 1, 2);
    assert_eq!(
        (stats.anti_air_successes, stats.anti_air_opportunities),
        (1, 2)
    );
    assert_eq!(stats.jump_ins_allowed, 1);
    assert_eq!(
        (stats.di_returned, stats.di_faced, stats.di_unconfirmed),
        (1, 2, 1)
    );
    assert_eq!(
        (
            stats.raw_drive_rushes_defended,
            stats.raw_drive_rushes_faced
        ),
        (1, 1)
    );
    assert_eq!((stats.dash_throws_faced, stats.throw_whiffs), (1, 1));
    assert_eq!(
        (stats.fastest_strike_losses, stats.fastest_strike_challenges),
        (1, 1)
    );
    assert_eq!(
        (stats.fastest_throw_losses, stats.fastest_throw_challenges),
        (0, 1)
    );
    assert_eq!(stats.minus_defense_opportunities, 3);
    assert_eq!((stats.burnout_count, stats.burnout_forced), (1, 1));
    assert!((stats.burnout_seconds - 2.0).abs() < 1e-6);
}
