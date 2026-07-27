use crate::pipeline_scenarios::{classic_punch, feature_for_p2, neutral_inputs, set_input_run};
use video_analyzer::advice::build_report_with_context;
use video_analyzer::{
    build_match_events_with_context, AnalysisContext, BadgeColor, FrameFeatures, InputDir,
};

#[test]
fn synthetic_three_round_match_preserves_pipeline_contract() {
    let features = three_round_match_for_p2();
    let p1_inputs = neutral_inputs(features.len());
    let mut p2_inputs = neutral_inputs(features.len());
    for frame in [40, 110, 180] {
        set_input_run(
            &mut p2_inputs,
            frame..=frame + 2,
            InputDir::Neutral,
            vec![classic_punch(BadgeColor::Green)],
        );
    }
    let context = AnalysisContext::from_characters("p2", Some("KEN"), Some("LUKE"));

    let events = build_match_events_with_context(&features, &p1_inputs, &p2_inputs, None, &context);
    let report = build_report_with_context(&features, &events, &context);

    assert_eq!(events.rounds.len(), 3);
    assert_eq!(
        events
            .rounds
            .iter()
            .map(|round| round.winner)
            .collect::<Vec<_>>(),
        vec![Some(2), Some(1), Some(2)]
    );
    assert_eq!(
        report
            .round_summaries
            .iter()
            .map(|round| round.won)
            .collect::<Vec<_>>(),
        vec![Some(true), Some(false), Some(true)]
    );
    assert_eq!(report.input_stats.as_ref().unwrap().button_presses, 3);
    assert_eq!(report.coverage.match_frames, features.len() as u32);
    assert_eq!(report.coverage.analyzed_match_frames, features.len() as u32);
}

fn three_round_match_for_p2() -> Vec<FrameFeatures> {
    let mut features = Vec::new();
    for winner in [2, 1, 2] {
        extend_full_health(&mut features, 30);
        for step in 1..=20 {
            let loser_hp = (1.0_f32 - step as f32 * 0.05).max(0.0);
            let health = if winner == 2 {
                (1.0, loser_hp)
            } else {
                (loser_hp, 1.0)
            };
            features.push(feature_for_p2(features.len() as u32, health.0, health.1));
        }
        let health = if winner == 2 { (1.0, 0.0) } else { (0.0, 1.0) };
        for _ in 0..20 {
            features.push(feature_for_p2(features.len() as u32, health.0, health.1));
        }
    }
    features
}

fn extend_full_health(features: &mut Vec<FrameFeatures>, count: usize) {
    for _ in 0..count {
        features.push(feature_for_p2(features.len() as u32, 1.0, 1.0));
    }
}
