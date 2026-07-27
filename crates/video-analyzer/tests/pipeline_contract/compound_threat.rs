use crate::pipeline_scenarios::{
    classic_punch, full_match, meter_pause, meter_run, neutral_inputs, set_input_run, timeline,
};
use video_analyzer::advice::build_report_with_context;
use video_analyzer::{
    build_match_events_with_context, spatial_candidate_windows, AnalysisContext, BadgeColor,
    DefenseResponseKind, InputDir, TeleportContext, ThreatOutcome,
};

#[test]
fn synthetic_dhalsim_sequence_preserves_compound_threat_contract() {
    let mut features = full_match(320);
    for feature in features.iter_mut().skip(250) {
        feature.opponent_hp = 0.9;
        feature.right_hp_raw = 0.9;
    }
    let p1_inputs = neutral_inputs(features.len());
    let mut p2_inputs = neutral_inputs(features.len());
    set_input_run(
        &mut p2_inputs,
        160..=162,
        InputDir::Neutral,
        vec![
            classic_punch(BadgeColor::Green),
            classic_punch(BadgeColor::Yellow),
        ],
    );
    let (p1_meter, p2_meter) = compound_threat_meters();
    let context = AnalysisContext::from_characters("p1", Some("BLANKA"), Some("DHALSIM"));

    let events = build_match_events_with_context(
        &features,
        &p1_inputs,
        &p2_inputs,
        Some((&p1_meter, &p2_meter)),
        &context,
    );
    let report = build_report_with_context(&features, &events, &context);

    assert_eq!(events.projectiles.len(), 1);
    assert_eq!(events.teleports.len(), 1);
    assert_eq!(events.compound_threats.len(), 1);
    let projectile = &events.projectiles[0];
    assert_eq!(
        (
            projectile.observed_start_frame,
            projectile.observed_end_frame,
            projectile.contact_frame,
        ),
        (100, 150, Some(175))
    );
    let threat = &events.compound_threats[0];
    assert_eq!((threat.attacker, threat.defender), (2, 1));
    assert_eq!(
        (threat.projectile_start_frame, threat.teleport_frame),
        (100, 160)
    );
    assert_eq!(
        (threat.followup_attack_frame, threat.followup_contact_frame),
        (190, Some(190))
    );
    assert_eq!(
        threat.projectile_response.as_ref().unwrap().kind,
        DefenseResponseKind::Parry
    );
    assert_eq!(
        threat.followup_response.as_ref().unwrap().kind,
        DefenseResponseKind::Guard
    );
    assert_eq!(
        (threat.outcome, threat.damage),
        (ThreatOutcome::Defended, 0.0)
    );
    assert_eq!(
        events.teleports[0].context,
        TeleportContext::ProjectileCovered
    );
    let spatial_windows = spatial_candidate_windows(&events);
    let spatial_window = &spatial_windows[0];
    assert_eq!(
        (spatial_window.start_frame, spatial_window.end_frame),
        (95, 215)
    );
    assert!(!report.cards.iter().any(|card| matches!(
        card.id.as_str(),
        "layered_defense" | "teleport_defense" | "punish_missed"
    )));
}

fn compound_threat_meters() -> (meter_tracker::MeterTimeline, meter_tracker::MeterTimeline) {
    let mut p1_entries = meter_run("parry", 146..=174);
    p1_entries.push(meter_pause("stun", 175, 180));
    p1_entries.push(meter_pause("stun", 190, 195));

    let mut p2_entries = meter_run("projectile_active", 100..=150);
    p2_entries.extend(meter_run("inv_full", 170..=176));
    p2_entries.push(meter_pause("active", 190, 195));
    (timeline("left", p1_entries), timeline("right", p2_entries))
}
