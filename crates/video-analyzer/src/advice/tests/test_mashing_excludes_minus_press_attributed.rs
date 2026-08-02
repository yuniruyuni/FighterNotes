use super::support::*;

#[test]
fn test_mashing_excludes_minus_press_attributed() {
    use crate::match_events::{InputSegment, MinusPressEvent, MinusPressOutcome};
    let mut ev = empty_events();
    // mashing の検出条件を満たす材料
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 880,
        pre_freeze_frame: 880,
        end_frame: 900,
        hp_before: 1.0,
        hp_after: 0.96,
        drop: 0.04,
        round_no: 1,
    });
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 1000,
        pre_freeze_frame: 1000,
        end_frame: 1020,
        hp_before: 0.9,
        hp_after: 0.78,
        drop: 0.12,
        round_no: 1,
    });
    ev.damage.push(DamageEvent {
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
    ev.segments[0] = vec![press(990), press(1190)];
    // 同じ被弾が press_while_minus に帰属している
    ev.presses_while_minus = [990, 1190]
        .into_iter()
        .map(|frame| MinusPressEvent {
            side: 1,
            frame,
            minus_frames: 5,
            pressed: "弱".to_string(),
            action_kind: DefensiveActionKind::Strike,
            outcome: MinusPressOutcome::CounterHit,
            drop: 0.12,
            confidence: EventConfidence::High,
            source_contact_frame: frame - 20,
            round_no: 1,
        })
        .collect();
    let report = detector_test_report(&ev, "p1");
    assert!(
        report.cards.iter().all(|c| c.id != "mashing"),
        "press_while_minus に帰属した被弾は mashing に出さない"
    );
}
