use super::support::*;
use crate::match_events::{SuperArtContext, SuperArtEvent, SuperArtOutcome};

#[test]
fn super_art_stats_cover_both_players_and_exact_punished_sa_wording() {
    let mut events = empty_events();
    events.super_arts = vec![
        super_event(1, 100, 2, false, SuperArtOutcome::Blocked, true),
        super_event(2, 300, 3, true, SuperArtOutcome::Hit, false),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);
    assert_eq!(stats.sa2_used, 1);
    assert_eq!(stats.super_blocked, 1);
    assert_eq!(stats.super_punished, 1);
    assert_eq!(stats.super_combo_uses, 1);
    assert_eq!(stats.opponent_ca_used, 1);
    assert_eq!(stats.opponent_super_hits, 1);

    let card = detect_reversal_punished(&events, 1).expect("punished SA card");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert!(card.title.contains("SA/CA"));
    assert!(card.description.contains("SA2"));
    assert!(card.evidence[0].label.contains("SA2"));
}

fn super_event(
    side: u8,
    frame: u32,
    level: u8,
    critical_art: bool,
    outcome: SuperArtOutcome,
    punished: bool,
) -> SuperArtEvent {
    SuperArtEvent {
        side,
        frame,
        gauge_drop_frame: frame + 1,
        level,
        critical_art,
        gauge_before: level as f32,
        gauge_after: 0.0,
        context: SuperArtContext::Combo,
        outcome,
        contact_frame: Some(frame + 20),
        damage: if outcome == SuperArtOutcome::Hit {
            0.2
        } else {
            0.0
        },
        ko: false,
        punished,
        punished_damage: if punished { 0.18 } else { 0.0 },
        confidence: EventConfidence::High,
        round_no: 1,
    }
}
