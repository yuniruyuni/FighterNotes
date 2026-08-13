use super::Analyzer;

#[test]
fn set_characters_keeps_legacy_own_opponent_mapping() {
    let mut analyzer = Analyzer::new("p2");
    analyzer.set_characters("BLANKA", "DHALSIM");

    assert_eq!(
        analyzer.analysis_context.p1.character.as_deref(),
        Some("DHALSIM")
    );
    assert_eq!(
        analyzer.analysis_context.p2.character.as_deref(),
        Some("BLANKA")
    );
    assert_eq!(analyzer.analysis_context.own_character(), Some("BLANKA"));
}

#[test]
fn analysis_context_json_preserves_player_metadata() {
    let mut analyzer = Analyzer::new("p1");
    analyzer
        .set_analysis_context(
            r#"{
                "ownSide":"p2",
                "p1":{"character":"KEN","controlType":"classic"},
                "p2":{"character":"DHALSIM","controlType":"modern"},
                "battleVersion":"2026.06"
            }"#,
        )
        .unwrap();

    assert_eq!(analyzer.analysis_context.own_side(), "p1");
    assert_eq!(
        analyzer.analysis_context.p2.control_type.as_deref(),
        Some("modern")
    );
    assert_eq!(
        analyzer.analysis_context.battle_version.as_deref(),
        Some("2026.06")
    );
}

#[test]
fn split_meter_session_matches_combined_analysis() {
    let mut combined = Analyzer::new("p1");
    let mut meter = Analyzer::new("p1");
    let mut attack = Analyzer::new("p1");
    let mut result = Analyzer::new("p1");

    for frame_index in 0..3 {
        combined.analyze_meter_inplace(1920, 1080, frame_index);
        combined.analyze_attack_info_inplace(1920, frame_index);
        combined.push_hud_features_inplace(1920, 1080, frame_index);
        combined.analyze_input_inplace(1920, 1080, frame_index);

        meter.analyze_meter_inplace(1920, 1080, frame_index);
        attack.analyze_attack_info_inplace(1920, frame_index);
        result.push_hud_features_inplace(1920, 1080, frame_index);
        result.analyze_input_inplace(1920, 1080, frame_index);
    }

    let meter_timeline = meter.finish_meter_timeline();
    result.set_meter_timeline(&meter_timeline).unwrap();
    result
        .set_attack_info_json(&attack.get_attack_info_json())
        .unwrap();
    assert_eq!(
        result.get_attack_info_json(),
        combined.get_attack_info_json()
    );

    assert_eq!(result.finish(), combined.finish());
    assert_eq!(result.get_timeline(), combined.get_timeline());
    assert_eq!(result.get_features_json(), combined.get_features_json());
    assert_eq!(result.get_tracked_inputs(), combined.get_tracked_inputs());
    assert_eq!(
        result.get_spatial_windows_json(),
        combined.get_spatial_windows_json()
    );
}

#[test]
fn reads_both_super_gauges_on_every_frame() {
    let mut analyzer = Analyzer::new("p1");
    paint_vertical_one(&mut analyzer.hud_buf, 68);
    paint_vertical_one(&mut analyzer.hud_buf, 22 + 1830);

    // 旧実装の 10 フレーム間引きでは frame 1 は未読になっていた。
    analyzer.push_hud_features_inplace(1920, 1080, 1);

    let feature = &analyzer.features[0];
    assert_eq!(feature.left_super_value, 1.0);
    assert!(!feature.left_super_uncertain);
    assert_eq!(feature.right_super_value, 1.0);
    assert!(!feature.right_super_uncertain);
}

fn paint_vertical_one(rgba: &mut [u8], x: usize) {
    const WIDTH: usize = 1920;
    for y in 8..60 {
        for px in x..x + 8 {
            let index = (y * WIDTH + px) * 4;
            rgba[index..index + 3].fill(245);
            rgba[index + 3] = 255;
        }
    }
}

/// GPU が数えた結果を入れた解析器は、走査していた頃と同じ特徴量になる。
/// ここがずれると試合画面の判定が変わり、ラウンド境界ごと動く。
#[test]
fn applied_gpu_counts_match_what_the_pixel_scan_produced() {
    let mut scanned = Analyzer::new("p1");
    let mut applied = Analyzer::new("p1");
    applied.use_gpu_hp_scores();

    for frame_index in 0..3 {
        scanned.push_hud_features_inplace(1920, 1080, frame_index);
        applied.push_hud_features_inplace(1920, 1080, frame_index);
    }

    // 走査した側と同じ割合になる数を渡す。strip は空なので一致は 0。
    let counts: Vec<u32> = (0..3).flat_map(|_| [0, 100, 0, 100]).collect();
    applied.apply_hp_score_counts_impl(&counts).unwrap();

    assert_eq!(applied.get_features_json(), scanned.get_features_json());
}

/// 数が合わない結果は断る。黙って詰めるとフレームと特徴量がずれる。
#[test]
fn a_mismatched_count_length_is_refused() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_scores();
    analyzer.push_hud_features_inplace(1920, 1080, 0);

    assert!(analyzer.apply_hp_score_counts_impl(&[0, 1]).is_err());
}
