use super::super::*;
use super::support::empty_events;

#[test]
fn detectors_abstain_without_relevant_events() {
    let events = empty_events();
    let cases = [
        ("layered_defense", detect_layered_defense(&events, 1)),
        ("teleport_defense", detect_teleport_defense(&events, 1)),
        ("anti_air", detect_anti_air(&events, 1, 2)),
        ("own_jumps", detect_own_jumps(&events, 1)),
        ("burnout", detect_burnout(&events, 1)),
        ("mashing", detect_mashing(&[], &events, 1, 0)),
        (
            "committed_button_vs_di",
            detect_committed_button_vs_di(&events, 1, 0),
        ),
        ("press_while_minus", detect_press_while_minus(&events, 1)),
        ("throw_while_minus", detect_throw_while_minus(&events, 1)),
        (
            "throw_interrupted_by_invincible",
            detect_throw_interrupted_by_invincible(&events, 1),
        ),
        (
            "throw_whiff_punished",
            detect_throw_whiff_punished(&events, 1),
        ),
        ("guard_break", detect_guard_break(&events, 1)),
        ("reversal_punished", detect_reversal_punished(&events, 1)),
        ("punish_missed", detect_punish_missed(&events, 1, None)),
        ("low_conversion", detect_low_conversion(&events, 1)),
        ("punish_fail", detect_punish_fail(&events, 1, None)),
        ("throw_loop", detect_throw_loop(&events, 2)),
        ("early_hits", detect_early_hits(&events, &[], 1)),
        ("lead_loss", detect_lead_loss(&events, &[], 0)),
        ("big_hits", detect_big_hits(&events, 1, &[])),
    ];

    for (name, card) in cases {
        assert!(card.is_none(), "{name} emitted a card without evidence");
    }
}

#[test]
fn direction_labels_cover_every_input_direction() {
    let cases = [
        ("N", "N"),
        ("U", "↑"),
        ("UR", "↗"),
        ("R", "→"),
        ("DR", "↘"),
        ("D", "↓"),
        ("DL", "↙"),
        ("L", "←"),
        ("UL", "↖"),
        ("invalid", "?"),
    ];

    for (direction, expected) in cases {
        assert_eq!(dir_arrow(direction), expected);
    }
}
