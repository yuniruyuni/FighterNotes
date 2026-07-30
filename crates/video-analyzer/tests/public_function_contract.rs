#[test]
fn crate_root_keeps_public_function_names() {
    let _ = video_analyzer::analyze_features;
    let _ = video_analyzer::analyze_features_with_context;
    let _ = video_analyzer::analyze_match;
    let _ = video_analyzer::analyze_match_with_context;
    let _ = video_analyzer::build_match_events;
    let _ = video_analyzer::build_match_events_with_context;
    let _ = video_analyzer::character_names;
    let _ = video_analyzer::clean_drive_temporal;
    let _ = video_analyzer::confirm_hp;
    let _ = video_analyzer::correct_hp_retroactive;
    let _ = video_analyzer::drive_bar_debug_json;
    let _ = video_analyzer::drive_fill_ratio;
    let _ = video_analyzer::drive_fill_ratio_from_hud_strip;
    let _ = video_analyzer::drive_gauge_read;
    let _ = video_analyzer::drive_gauge_read_from_hud_strip;
    let _ = video_analyzer::finalize_features;
    let _ = video_analyzer::hp_bar_debug_json;
    let _ = video_analyzer::hp_bar_score;
    let _ = video_analyzer::hp_bar_score_from_hud_strip;
    let _ = video_analyzer::hp_col_active;
    let _ = video_analyzer::hp_col_orange;
    let _ = video_analyzer::hp_col_pixel_detail_json;
    let _ = video_analyzer::hp_col_yellow;
    let _ = video_analyzer::hp_damage_fill;
    let _ = video_analyzer::hp_damage_fill_from_hud_strip;
    let _ = video_analyzer::hp_fill_ratio;
    let _ = video_analyzer::hp_fill_ratio_from_hud_strip;
    let _ = video_analyzer::hp_fill_ratio_with_quality;
    let _ = video_analyzer::hp_fill_ratio_with_quality_from_hud_strip;
    let _ = video_analyzer::hp_parallelogram;
    let _ = video_analyzer::input_history_debug_json;
    let _ = video_analyzer::punish_options;
    let _ = video_analyzer::read_input_row0_from_strip;
    let _ = video_analyzer::read_input_rows;
    let _ = video_analyzer::refine_match_events_with_spatial;
    let _ = video_analyzer::repair_row0_sequence;
    let _ = video_analyzer::spatial_candidate_windows;
    let _ = video_analyzer::super_gauge_debug_json;
    let _ = video_analyzer::super_gauge_read;
    let _ = video_analyzer::super_gauge_read_from_hud_strip;
}

#[test]
fn crate_root_keeps_public_geometry_constants() {
    assert_eq!(video_analyzer::HUD_STRIP_Y, 64);
    assert_eq!(video_analyzer::HUD_STRIP_H, 70);
    assert_eq!(video_analyzer::INPUT_STRIP_Y, 232);
    assert_eq!(video_analyzer::INPUT_STRIP_H, 36);
    assert_eq!(video_analyzer::HP_ROI_P1, (172, 853, 64, 95));
    assert_eq!(video_analyzer::HP_ROI_P2, (1067, 1748, 64, 95));
}
