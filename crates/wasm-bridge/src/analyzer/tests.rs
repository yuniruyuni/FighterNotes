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
    // SA ゲージは等倍で置いた帯から読む。
    paint_vertical_one(&mut analyzer.super_buf, 68);
    paint_vertical_one(&mut analyzer.super_buf, 22 + 1830);

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

/// 数えた画素の割合がそのままスコアになる。ここが崩れると試合画面の
/// 判定が全フレームで狂う。
#[test]
fn the_applied_score_is_the_share_of_matching_pixels() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_scores();
    for frame_index in 0..4 {
        analyzer.push_hud_features_inplace(1920, 1080, frame_index);
    }

    analyzer
        .apply_hp_score_counts_impl(&[
            // 両方ちょうど境界。境目は「以上」で通る。
            35, 1000, 25,
            1000, // 左だけ境界を割る。片方欠けたら試合画面ではない。
            34, 1000, 25, 1000, // 右だけ境界を割る。
            35, 1000, 24, 1000, // 数えた画素が無いときは 0 割りにしない。
            0, 0, 0, 0,
        ])
        .unwrap();

    let scores: Vec<(f32, f32, bool)> = analyzer
        .features
        .iter()
        .map(|feature| {
            (
                feature.left_hp_score,
                feature.right_hp_score,
                feature.is_match_screen,
            )
        })
        .collect();

    assert_eq!(
        scores,
        vec![
            (0.035, 0.025, true),
            (0.034, 0.025, false),
            (0.035, 0.024, false),
            (0.0, 0.0, false),
        ]
    );
}

/// 数が合わない結果は断る。黙って詰めるとフレームと特徴量がずれる。
#[test]
fn a_mismatched_count_length_is_refused() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_scores();
    analyzer.push_hud_features_inplace(1920, 1080, 0);

    assert!(analyzer.apply_hp_score_counts_impl(&[0, 1]).is_err());
}

/// GPU が分類した列を渡した解析器は、画素を走査していた頃と同じ特徴量に
/// なる。CA の判定は HP を見るので、後から入れる順番を間違えると
/// ここだけが静かにずれる。
#[test]
fn applied_gpu_columns_match_what_the_pixel_scan_produced() {
    let mut scanned = Analyzer::new("p1");
    let mut applied = Analyzer::new("p1");
    applied.use_gpu_hp_scores();
    applied.use_gpu_hp_columns();

    for frame_index in 0..3 {
        scanned.push_hud_features_inplace(1920, 1080, frame_index);
        applied.push_hud_features_inplace(1920, 1080, frame_index);
    }

    // strip は空なので、走査した側と同じ「読めない」列になる。
    let width = video_analyzer::hp_column_scan("p1")[1] as usize;
    // 5 は「空き」。空の strip を走査したときと同じ分類になる。
    let columns: Vec<u32> = vec![5; 3 * width * 2];
    applied.apply_hp_columns_impl(0, &columns).unwrap();
    applied
        .apply_hp_score_counts_impl(&(0..3).flat_map(|_| [0, 100, 0, 100]).collect::<Vec<_>>())
        .unwrap();
    applied.apply_hp_fills().unwrap();

    assert_eq!(applied.get_features_json(), scanned.get_features_json());
}

/// フレーム数の合わない列は断る。詰めると以降のフレームが全部ずれる。
#[test]
fn columns_that_do_not_fill_whole_frames_are_refused() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();

    assert!(analyzer.apply_hp_columns_impl(0, &[0, 1, 2]).is_err());
}

/// CA は「SA ゲージが溜まっている」かつ「HP が残り少ない」で灯る。
/// どちらか一方では灯らないし、CA 発動中の表示は HP に関わらず灯る。
#[test]
fn the_critical_art_light_needs_both_a_full_gauge_and_low_health() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();
    for frame_index in 0..4 {
        analyzer.push_hud_features_inplace(1920, 1080, frame_index);
    }
    analyzer.hp_fills = vec![
        // 溜まっている + 瀕死 → 灯る
        (0.2, false, 0.2, false),
        // 溜まっている + 余裕あり → 灯らない
        (0.9, false, 0.9, false),
        // 溜まっていない + 瀕死 → 灯らない
        (0.2, false, 0.2, false),
        // CA 発動中の表示は HP を問わない
        (0.9, false, 0.9, false),
    ];
    analyzer.ca_gates = vec![
        (false, true, false, true),
        (false, true, false, true),
        (false, false, false, false),
        (true, false, true, false),
    ];

    analyzer.apply_hp_fills().unwrap();

    let lights: Vec<(bool, bool)> = analyzer
        .features
        .iter()
        .map(|feature| (feature.left_ca_ready, feature.right_ca_ready))
        .collect();
    assert_eq!(
        lights,
        vec![(true, true), (false, false), (false, false), (true, true)]
    );
}

/// 境目は 25.5%。ここを跨ぐと灯り方が変わる。
#[test]
fn the_health_gate_sits_at_a_quarter_of_the_bar() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();
    for frame_index in 0..2 {
        analyzer.push_hud_features_inplace(1920, 1080, frame_index);
    }
    analyzer.hp_fills = vec![(0.255, false, 0.256, false), (0.0, false, 0.0, false)];
    analyzer.ca_gates = vec![(false, true, false, true), (false, true, false, true)];

    analyzer.apply_hp_fills().unwrap();

    assert_eq!(
        (
            analyzer.features[0].left_ca_ready,
            analyzer.features[0].right_ca_ready
        ),
        (true, false),
        "ちょうど 25.5% は含み、それを超えたら含まない"
    );
    assert!(analyzer.features[1].left_ca_ready, "HP 0 でも灯る");
}

/// 数の合わない受け取りは断る。詰めるとフレームと HP がずれる。
#[test]
fn fills_and_gates_must_cover_every_frame() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();
    analyzer.push_hud_features_inplace(1920, 1080, 0);

    analyzer.hp_fills = Vec::new();
    analyzer.ca_gates = vec![(false, false, false, false)];
    assert!(analyzer.apply_hp_fills().is_err(), "充填率が足りていない");

    analyzer.hp_fills = vec![(0.5, false, 0.5, false)];
    analyzer.ca_gates = Vec::new();
    assert!(
        analyzer.apply_hp_fills().is_err(),
        "CA の条件が足りていない"
    );
}

/// GPU を使っていない解析器は、後から入れる手続きを素通りする。
#[test]
fn an_analyzer_reading_pixels_itself_needs_nothing_applied() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.push_hud_features_inplace(1920, 1080, 0);

    assert!(analyzer.apply_hp_fills().is_ok());
}

/// 白枠から充填が続く列を渡せば、その割合が充填率になる。
#[test]
fn applied_columns_become_the_fill_of_that_frame() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();
    let width = video_analyzer::hp_column_scan("p2")[1] as usize;
    let mut frame = vec![5u32; width * 2];
    // P2 側だけ、端の白枠・充填・充填端の白線・遠端の白枠を並べる。
    for (at, value) in [
        (0..3, 0u32),
        (3..width / 2, 1),
        (width / 2..width / 2 + 2, 0),
    ] {
        frame[width + at.start..width + at.end].fill(value);
    }
    frame[width * 2 - 3..].fill(0);

    analyzer.apply_hp_columns_impl(0, &frame).unwrap();

    let (_, _, right, right_uncertain) = analyzer.hp_fills[0];
    assert!(!right_uncertain, "読めなかった");
    assert!((0.45..=0.55).contains(&right), "充填が {right} になった");
}

/// 左右で違う並びを渡せば、左右で違う充填率になる。取り違えると
/// 相手の残量を自分のものとして読む。
#[test]
fn each_sides_columns_land_on_that_side() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();
    let width = video_analyzer::hp_column_scan("p1")[1] as usize;
    let mut frame = vec![5u32; width * 2];
    // P1 はアンカーが右端なので、白枠と充填を右から並べる。
    frame[width - 3..width].fill(0);
    frame[width / 2..width - 3].fill(1);
    frame[width / 2 - 2..width / 2].fill(0);
    frame[0..3].fill(0);

    analyzer.apply_hp_columns_impl(0, &frame).unwrap();

    let (left, left_uncertain, right, right_uncertain) = analyzer.hp_fills[0];
    assert!(!left_uncertain, "P1 が読めなかった");
    assert!((0.45..=0.55).contains(&left), "P1 の充填が {left} になった");
    assert!(right_uncertain, "空きだけの P2 が読めたことになっている");
    assert_eq!(right, 0.0);
}

/// まとまりが飛んで届いても、間のフレームは「読めなかった」にする。
/// 0% と言い切ると、欠けた区間が瀕死として扱われる。
#[test]
fn frames_that_never_arrived_stay_unread() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();
    let width = video_analyzer::hp_column_scan("p1")[1] as usize;

    analyzer
        .apply_hp_columns_impl(2, &vec![5u32; width * 2])
        .unwrap();

    assert_eq!(analyzer.hp_fills.len(), 3, "3 フレーム目までの場所ができる");
    assert_eq!(analyzer.hp_fills[0], (0.0, true, 0.0, true));
    assert_eq!(analyzer.hp_fills[1], (0.0, true, 0.0, true));
}

/// 試合画面かどうかは左右どちらが欠けても成り立たない。
#[test]
fn the_match_screen_needs_both_bars() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_scores();
    for frame_index in 0..2 {
        analyzer.push_hud_features_inplace(1920, 1080, frame_index);
    }

    analyzer
        .apply_hp_score_counts_impl(&[35, 1000, 24, 1000, 34, 1000, 25, 1000])
        .unwrap();

    assert!(!analyzer.features[0].is_match_screen, "右が足りていない");
    assert!(!analyzer.features[1].is_match_screen, "左が足りていない");
}

/// 後から前のまとまりが届いても、既に入った分を壊さない。
#[test]
fn a_late_batch_does_not_wipe_the_frames_after_it() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();
    let width = video_analyzer::hp_column_scan("p1")[1] as usize;

    analyzer
        .apply_hp_columns_impl(2, &vec![5u32; width * 2])
        .unwrap();
    analyzer
        .apply_hp_columns_impl(0, &vec![5u32; width * 2])
        .unwrap();

    assert_eq!(analyzer.hp_fills.len(), 3, "後の分を切り落としている");
}

/// 自分がどちら側かで、左右のどちらを自分の残量として読むかが決まる。
#[test]
fn the_players_own_bar_depends_on_which_side_they_are() {
    let mut analyzer = Analyzer::new("p2");
    analyzer.use_gpu_hp_columns();
    analyzer.push_hud_features_inplace(1920, 1080, 0);
    analyzer.hp_fills = vec![(0.25, false, 0.75, false)];
    analyzer.ca_gates = vec![(false, false, false, false)];

    analyzer.apply_hp_fills().unwrap();

    assert_eq!(analyzer.features[0].own_hp, 0.75, "P2 は右側が自分");
    assert_eq!(analyzer.features[0].opponent_hp, 0.25);
    assert_eq!(analyzer.features[0].left_hp_raw, 0.25);
    assert_eq!(analyzer.features[0].right_hp_raw, 0.75);
}

/// 読めなかった側は、読めた側と区別できるように印を残す。
#[test]
fn an_unread_bar_is_marked_as_such() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_columns();
    analyzer.push_hud_features_inplace(1920, 1080, 0);
    analyzer.hp_fills = vec![(0.5, true, 0.5, false)];
    analyzer.ca_gates = vec![(false, false, false, false)];

    analyzer.apply_hp_fills().unwrap();

    assert_eq!(analyzer.features[0].left_hp_raw_quality, 1.0);
    assert_eq!(analyzer.features[0].right_hp_raw_quality, 0.0);
    assert_ne!(
        analyzer.features[0].own_hp, 0.5,
        "読めなかった側をそのまま残量にしている"
    );
}

/// まとまりごとに届く数え上げは、先頭のフレーム番号の位置から入る。
#[test]
fn each_batch_of_counts_lands_at_the_frame_it_starts_at() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_scores();
    analyzer.use_gpu_hp_columns();
    for frame_index in 0..3 {
        analyzer.push_hud_features_inplace(1920, 1080, frame_index);
    }
    analyzer.hp_fills = vec![(0.0, true, 0.0, true); 3];
    analyzer.ca_gates = vec![(false, false, false, false); 3];

    // 後のまとまりから先に届いても、置き場所は変わらない。
    analyzer
        .push_hp_score_counts_impl(2, &[35, 1000, 25, 1000])
        .unwrap();
    analyzer
        .push_hp_score_counts_impl(0, &[0, 1000, 0, 1000])
        .unwrap();
    analyzer.apply_hp_fills().unwrap();

    let screens: Vec<bool> = analyzer
        .features
        .iter()
        .map(|feature| feature.is_match_screen)
        .collect();
    assert_eq!(screens, vec![false, false, true]);
    assert_eq!(analyzer.features[2].left_hp_score, 0.035);
}

/// フレーム 1 枚分に満たない切れ端は断る。詰めると以降が全部ずれる。
#[test]
fn counts_that_do_not_fill_whole_frames_are_refused() {
    let mut analyzer = Analyzer::new("p1");

    assert!(analyzer.push_hp_score_counts_impl(0, &[1, 2, 3]).is_err());
}

/// 届かなかったフレームは数えていない扱いのまま。0 と言い切らない。
#[test]
fn frames_whose_counts_never_arrived_score_nothing() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_scores();

    analyzer
        .push_hp_score_counts_impl(1, &[35, 1000, 25, 1000])
        .unwrap();

    assert_eq!(analyzer.hp_score_counts[0], [0, 0, 0, 0]);
    assert_eq!(analyzer.hp_score_counts[1], [35, 1000, 25, 1000]);
}

/// 続けて届くまとまりは、フレームを飛ばさずに並ぶ。
#[test]
fn consecutive_batches_fill_consecutive_frames() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_hp_scores();

    analyzer
        .push_hp_score_counts_impl(0, &[1, 10, 2, 20, 3, 30, 4, 40])
        .unwrap();
    analyzer
        .push_hp_score_counts_impl(2, &[5, 50, 6, 60, 7, 70, 8, 80])
        .unwrap();

    assert_eq!(
        analyzer.hp_score_counts,
        vec![
            [1, 10, 2, 20],
            [3, 30, 4, 40],
            [5, 50, 6, 60],
            [7, 70, 8, 80],
        ]
    );
}

/// GPU が分類したドライブの列は、画素を走査していた頃と同じ特徴量になる。
#[test]
fn applied_gpu_drive_columns_match_what_the_pixel_scan_produced() {
    let mut scanned = Analyzer::new("p1");
    let mut applied = Analyzer::new("p1");
    applied.use_gpu_drive();

    for frame_index in 0..3 {
        scanned.push_hud_features_inplace(1920, 1080, frame_index);
        applied.push_hud_features_inplace(1920, 1080, frame_index);
    }

    // strip は空なので、走査した側と同じ「ROI に収まらない」列になる。
    let width = video_analyzer::drive_column_scan("left")[1] as usize;
    applied
        .apply_drive_columns_impl(0, &vec![4u32; 3 * width * 2])
        .unwrap();
    applied.apply_hp_fills().unwrap();

    assert_eq!(applied.get_features_json(), scanned.get_features_json());
}

/// 左右で別の並びを渡せば、左右で別の値になる。取り違えると相手の
/// ゲージを自分のものとして読む。
#[test]
fn each_sides_drive_columns_land_on_that_side() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_drive();
    analyzer.push_hud_features_inplace(1920, 1080, 0);
    let width = video_analyzer::drive_column_scan("left")[1] as usize;
    let mut left_lit = vec![3u32; width * 2];
    left_lit[..width].fill(0);
    let mut right_lit = vec![3u32; width * 2];
    right_lit[width..].fill(0);

    analyzer.push_hud_features_inplace(1920, 1080, 1);
    analyzer.apply_drive_columns_impl(0, &left_lit).unwrap();
    analyzer.apply_drive_columns_impl(1, &right_lit).unwrap();
    analyzer.apply_hp_fills().unwrap();

    assert_eq!(
        (
            analyzer.features[0].left_drive_ratio,
            analyzer.features[0].right_drive_ratio
        ),
        (1.0, 0.0),
        "左だけ点灯させた結果になっていない"
    );
    assert_eq!(
        (
            analyzer.features[1].left_drive_ratio,
            analyzer.features[1].right_drive_ratio
        ),
        (0.0, 1.0),
        "右だけ点灯させた結果になっていない"
    );
}

/// 続けて届くまとまりは、フレームを飛ばさずに並ぶ。
#[test]
fn consecutive_drive_batches_fill_consecutive_frames() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_drive();
    let width = video_analyzer::drive_column_scan("left")[1] as usize;
    let lit = vec![0u32; width * 2 * 2];
    let rest = vec![3u32; width * 2 * 2];

    analyzer.apply_drive_columns_impl(0, &lit).unwrap();
    analyzer.apply_drive_columns_impl(2, &rest).unwrap();

    assert_eq!(analyzer.drive_reads.len(), 4);
    assert_eq!(analyzer.drive_reads[1].0.value, 6.0, "1 枚目が満タンでない");
    assert_eq!(analyzer.drive_reads[2].0.value, 0.0, "3 枚目が空でない");
}

/// フレーム数の合わない列は断る。詰めると以降が全部ずれる。
#[test]
fn drive_columns_that_do_not_fill_whole_frames_are_refused() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_drive();

    assert!(analyzer.apply_drive_columns_impl(0, &[0, 1, 2]).is_err());
}

/// 届かなかったフレームは「読めなかった」にしておく。
#[test]
fn drive_frames_that_never_arrived_stay_unread() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_drive();
    let width = video_analyzer::drive_column_scan("left")[1] as usize;

    analyzer
        .apply_drive_columns_impl(2, &vec![3u32; width * 2])
        .unwrap();

    assert_eq!(analyzer.drive_reads.len(), 3);
    // 読めなかった印だけを立て、値や状態は勝手に決めない。
    assert!(analyzer.drive_reads[0].0.uncertain);
    assert!(
        !analyzer.drive_reads[0].0.burnout,
        "バーンアウト扱いにしている"
    );
    assert_eq!(analyzer.drive_reads[0].0.value, 0.0);
    assert_eq!(analyzer.drive_reads[0].0.recovery, 0.0);
    assert!(analyzer.drive_reads[1].1.uncertain);
}

/// 後から前のまとまりが届いても、既に入った分を壊さない。
#[test]
fn a_late_drive_batch_does_not_wipe_the_frames_after_it() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_drive();
    let width = video_analyzer::drive_column_scan("left")[1] as usize;

    analyzer
        .apply_drive_columns_impl(2, &vec![0u32; width * 2])
        .unwrap();
    analyzer
        .apply_drive_columns_impl(0, &vec![3u32; width * 2])
        .unwrap();

    assert_eq!(analyzer.drive_reads.len(), 3, "後の分を切り落としている");
    assert_eq!(analyzer.drive_reads[2].0.value, 6.0);
}

/// 数が合わない受け取りは断る。
#[test]
fn drive_reads_must_cover_every_frame() {
    let mut analyzer = Analyzer::new("p1");
    analyzer.use_gpu_drive();
    analyzer.push_hud_features_inplace(1920, 1080, 0);

    assert!(analyzer.apply_hp_fills().is_err());
}
