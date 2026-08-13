//! 解析を始める前の門に対するテスト。
//!
//! ラウンド境界も入力の対応も、間違ったまま進むと以降の全ての判断が
//! ずれる。しかも結果は「それらしい値」になるので、出力からは気づけない。

use super::*;

/// 1 試合は 2 本先取。ラウンド開始演出は 2 回か 3 回出る。
#[test]
fn a_match_shows_the_round_start_two_or_three_times() {
    assert!(marker_count_is_valid(2), "2 本で決着した試合");
    assert!(marker_count_is_valid(3), "3 本目まで行った試合");
}

/// それ以外の数は、動画が途中から始まっているか、中央が隠れているか、
/// 別の何かを拾っている。ラウンド境界には使えない。
#[test]
fn any_other_count_cannot_be_the_round_boundaries() {
    assert!(!marker_count_is_valid(0), "一つも見つかっていない");
    assert!(!marker_count_is_valid(1), "途中から始まる動画");
    assert!(!marker_count_is_valid(4), "誤検出を含んでいる");
}

/// 断った理由には、実際に見つかった数を入れる。数が分からないと、
/// 動画のどこを直せばよいか判断できない。
#[test]
fn the_refusal_says_how_many_were_found() {
    let message = marker_count_error(1);

    assert!(
        message.contains('1'),
        "見つかった数を出していない: {message}"
    );
    assert!(!message.is_empty());
}

/// 入力欄が一つも無ければ使えない。数が合っていても、空同士では
/// 何も対応させられない。
#[test]
fn no_input_rows_at_all_cannot_be_used() {
    let analyzer = Analyzer::new("p1");

    assert!(
        !analyzer.input_rows_are_usable(),
        "空同士を使えることにしている"
    );
}

// ── 解析を組み立てる道筋 ─────────────────────────────────────────────────

use video_analyzer::{
    AttackAttribute, AttackInfoObservation, AttackInfoSide, FightMarker, FightObservation,
    FrameFeatures, InputDir, InputRow,
};

/// 1 フレーム分の観測。
fn feature(frame_index: u32, own_hp: f32) -> FrameFeatures {
    FrameFeatures {
        frame_index,
        fps: 60.0,
        own_hp,
        opponent_hp: 1.0,
        is_match_screen: true,
        own_meter_state: None,
        opponent_meter_state: None,
        left_hp_score: 0.1,
        right_hp_score: 0.1,
        left_drive_ratio: 1.0,
        right_drive_ratio: 1.0,
        left_burnout: false,
        right_burnout: false,
        left_drive_uncertain: false,
        right_drive_uncertain: false,
        left_super_value: 0.0,
        right_super_value: 0.0,
        left_super_uncertain: true,
        right_super_uncertain: true,
        left_ca_ready: false,
        right_ca_ready: false,
        left_hp_raw: own_hp,
        right_hp_raw: 1.0,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 0.0,
    }
}

/// 前半で全快、後半で削られる 400 フレームの試合。
fn analyzer_with_features(own_side: &str) -> Analyzer {
    let mut analyzer = Analyzer::new(own_side);
    analyzer.features = (0..400)
        .map(|frame_index| {
            let own_hp = if frame_index < 200 { 1.0 } else { 0.4 };
            feature(frame_index, own_hp)
        })
        .collect();
    analyzer
}

fn marker(first_frame: u32) -> FightMarker {
    FightMarker {
        first_frame,
        last_frame: first_frame + 10,
        peak_frame: first_frame + 5,
        peak_score: 1.0,
    }
}

fn attack_info_side(last_damage: u32, combo_damage: u32) -> AttackInfoSide {
    AttackInfoSide {
        last_damage,
        scaling_percent: 100,
        combo_damage,
        max_combo_damage: combo_damage,
        attribute: AttackAttribute::Middle,
    }
}

fn readable_row() -> InputRow {
    InputRow {
        count: Some(1),
        dir: InputDir::Neutral,
        badges: Vec::new(),
        auto: false,
        throw: false,
        empty: false,
        uncertain: false,
    }
}

/// ラウンド開始演出を必須にした解析は、数が合わなければ断る。途中から
/// 始まる動画を黙って解析すると、ラウンド境界がまるごとずれる。
#[test]
fn a_strict_session_refuses_a_video_without_the_round_starts() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.require_fight_markers = true;
    analyzer.fight_markers = Some(vec![marker(50)]);

    let error = analyzer.ensure_events().expect_err("断られる");

    assert!(error.contains('1'), "見つかった数を伝えていない: {error}");
    assert!(analyzer.events.is_none(), "断ったのに解析を残している");
}

/// 必須にしていなければ、演出が見つからなくても HP から割って進む。
#[test]
fn a_lenient_session_falls_back_to_the_health_bars() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50)]);

    analyzer.ensure_events().expect("進む");

    let events = analyzer.events.as_ref().expect("解析がある");
    assert_eq!(
        events.rounds.first().map(|round| round.start_frame),
        Some(0),
        "HP から割っていない: {:?}",
        events.rounds
    );
}

/// 演出が揃っていれば、そこをラウンドの頭にする。
#[test]
fn the_round_starts_where_the_fight_banner_settles() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);

    analyzer.ensure_events().expect("進む");

    let events = analyzer.events.as_ref().expect("解析がある");
    assert_eq!(
        events.rounds.first().map(|round| round.start_frame),
        Some(60),
        "演出の位置から始めていない: {:?}",
        events.rounds
    );
}

/// 入力欄がフレーム数と揃っていれば、入力も解析へ渡す。
#[test]
fn matching_input_rows_are_carried_into_the_analysis() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);
    analyzer.input_rows = (0..analyzer.features.len())
        .map(|_| (readable_row(), readable_row()))
        .collect();

    analyzer.ensure_events().expect("進む");

    assert!(analyzer.tracked_json.is_some(), "入力を残していない");
    let events = analyzer.events.as_ref().expect("解析がある");
    assert!(
        !events.segments.iter().all(Vec::is_empty),
        "入力が解析へ渡っていない"
    );
}

/// 数の合わない入力欄は使わない。どの入力がどのフレームのものか
/// 決まらないまま進むと、入力と場面が全部ずれる。
#[test]
fn mismatched_input_rows_are_dropped_instead_of_guessed() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);
    analyzer.input_rows = (0..10).map(|_| (readable_row(), readable_row())).collect();

    analyzer.ensure_events().expect("進む");

    assert!(
        analyzer.tracked_json.is_none(),
        "数の合わない入力を使っている"
    );
    let events = analyzer.events.as_ref().expect("解析がある");
    assert!(events.segments.iter().all(Vec::is_empty));
}

/// 使い終わった入力欄は手放す。ブラウザ側で長い動画を扱うので、
/// 抱えたままだと解析の終盤で足りなくなる。
#[test]
fn the_input_rows_are_released_after_the_analysis() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);
    analyzer.input_rows = (0..analyzer.features.len())
        .map(|_| (readable_row(), readable_row()))
        .collect();

    analyzer.ensure_events().expect("進む");

    assert!(analyzer.input_rows.is_empty(), "入力欄を抱えたまま");
}

/// 二度目の呼び出しでは組み立て直さない。
#[test]
fn the_analysis_is_only_built_once() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);
    analyzer.ensure_events().expect("進む");
    let rounds = analyzer.events.as_ref().expect("解析がある").rounds.len();

    // 二度目までに観測を足しても、既にある解析は変わらない。
    analyzer.features.push(feature(400, 0.4));
    analyzer.ensure_events().expect("進む");

    assert_eq!(
        analyzer.events.as_ref().expect("解析がある").rounds.len(),
        rounds,
        "解析を組み立て直している"
    );
}

/// 演出が見つかっていなければ、その場で探しに行く。
#[test]
fn the_fight_markers_are_detected_when_they_are_still_unknown() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_observations = vec![
        FightObservation {
            frame: 100,
            score: 0.7,
        },
        FightObservation {
            frame: 104,
            score: 0.8,
        },
        FightObservation {
            frame: 108,
            score: 0.75,
        },
    ];

    analyzer.ensure_fight_markers();

    assert_eq!(
        analyzer.fight_markers,
        Some(vec![FightMarker {
            first_frame: 100,
            last_frame: 108,
            peak_frame: 104,
            peak_score: 0.8,
        }]),
        "観測から演出区間を組み立てていない"
    );
}

/// ローカル tracker の開いた区間は、解析直前に閉じてからイベント層へ渡す。
/// さらに marker 無しの互換経路でも、その timeline を `Some` で渡す。
#[test]
fn the_local_meter_is_finished_and_used_on_the_fallback_path() {
    use frame_meter::{BrightClass, CellState, RowObs};

    let row = |edge: i32, state: CellState| {
        let mut observation = RowObs::empty();
        observation.v.fill(100.0);
        observation.bright.fill(BrightClass::Fresh);
        observation.states.fill(state);
        observation.fresh_edge = edge;
        observation
    };

    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50)]);
    analyzer.input_rows = (0..analyzer.features.len())
        .map(|_| (readable_row(), readable_row()))
        .collect();
    for video_frame in 200..=211 {
        analyzer.tracker.update(
            video_frame,
            row(5, CellState::Active),
            row(5, CellState::Stun),
        );
    }

    analyzer.ensure_events().expect("進む");

    assert_eq!(
        analyzer.tracker.left.segments[0].entries[0].state, "active",
        "開いた tracker 区間を確定していない"
    );
    assert!(
        !analyzer
            .events
            .as_ref()
            .expect("解析がある")
            .contacts
            .is_empty(),
        "fallback 経路で確定済み timeline を渡していない"
    );
}

/// 外から渡されたフレームメーターを使う。分割セッションでは、メーターを
/// 別の走査で作って持ち込む。
#[test]
fn an_imported_meter_is_used_instead_of_the_local_tracker() {
    use meter_tracker::{MeterTimeline, TimelineEntry, TimelineSegment};

    // f100 で 10 フレーム止まる。この停止の重なりが接触の印になる。
    let timeline = |side: &str, state: &str| {
        let mut entries: Vec<TimelineEntry> = (0..100)
            .map(|frame| TimelineEntry {
                game_frame: frame,
                state: "empty".to_string(),
                video_frame_first: frame,
                video_frame_last: frame,
                confidence: 1.0,
            })
            .collect();
        entries.push(TimelineEntry {
            game_frame: 100,
            state: state.to_string(),
            video_frame_first: 100,
            video_frame_last: 109,
            confidence: 1.0,
        });
        entries.extend((110..400).map(|frame| TimelineEntry {
            game_frame: frame - 9,
            state: "empty".to_string(),
            video_frame_first: frame,
            video_frame_last: frame,
            confidence: 1.0,
        }));
        MeterTimeline {
            side: side.to_string(),
            segments: vec![TimelineSegment {
                segment_id: 0,
                entries,
            }],
        }
    };

    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);
    analyzer.imported_meter = Some((timeline("left", "active"), timeline("right", "stun")));
    analyzer.input_rows = (0..analyzer.features.len())
        .map(|_| (readable_row(), readable_row()))
        .collect();

    analyzer.ensure_events().expect("進む");

    let events = analyzer.events.as_ref().expect("解析がある");
    assert!(
        !events.contacts.is_empty(),
        "持ち込んだメーターを使っていない"
    );
}

/// 外から渡された中央攻撃表示を使う。渡されていなければ自分で追った
/// ものを使う。
#[test]
fn a_set_attack_info_fills_the_timeline_it_was_split_from() {
    let mut analyzer = Analyzer::new("p1");
    let mut meter = Analyzer::new("p1");
    analyzer
        .set_meter_timeline(&meter.finish_meter_timeline())
        .unwrap();
    let observations = serde_json::to_string(&vec![AttackInfoObservation {
        frame_index: 7,
        p1: attack_info_side(0, 0),
        p2: attack_info_side(800, 800),
    }])
    .unwrap();
    analyzer.set_attack_info_json(&observations).unwrap();

    let timeline: serde_json::Value = serde_json::from_str(&analyzer.get_timeline()).unwrap();
    assert_eq!(timeline["attack_info"][0]["frame_index"], 7);
    assert_eq!(timeline["left"]["side"], "left");
    assert_eq!(analyzer.get_attack_info_json(), observations);
}

#[test]
fn an_imported_attack_info_replaces_the_tracked_one() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);
    let idle = attack_info_side(0, 0);
    analyzer.imported_attack_info = Some(vec![
        AttackInfoObservation {
            frame_index: 198,
            p1: idle.clone(),
            p2: idle.clone(),
        },
        AttackInfoObservation {
            frame_index: 202,
            p1: idle.clone(),
            p2: attack_info_side(800, 800),
        },
        AttackInfoObservation {
            frame_index: 212,
            p1: idle.clone(),
            p2: attack_info_side(600, 1_400),
        },
        AttackInfoObservation {
            frame_index: 260,
            p1: idle.clone(),
            p2: idle,
        },
    ]);

    analyzer.ensure_events().expect("進む");

    assert!(
        !analyzer
            .events
            .as_ref()
            .expect("解析がある")
            .attack_evidence
            .sequences
            .is_empty(),
        "持ち込んだ中央攻撃表示をイベント層へ渡していない"
    );
}

/// FIGHT 直後の raw HP は画面左右の値なので、P2 視点では右側を自分の
/// 開始 HP として使う。空文字（P1 fallback）へ落ちても同じにならない列で固定する。
#[test]
fn the_context_side_selects_the_raw_health_at_a_fight_opening() {
    let mut analyzer = Analyzer::new("p2");
    analyzer.features = (0..120)
        .map(|frame_index| {
            let mut observed = feature(frame_index, if frame_index < 20 { 0.4 } else { 0.75 });
            observed.left_hp_raw = 0.4;
            observed.right_hp_raw = if (20..40).contains(&frame_index) {
                1.0
            } else {
                0.75
            };
            observed
        })
        .collect();
    analyzer.fight_markers = Some(vec![marker(20), marker(80)]);

    analyzer.ensure_events().expect("進む");

    assert_eq!(
        analyzer.features[45].own_hp, 0.75,
        "P2 の自 HP に左側 raw HP を使っている"
    );
}

/// 入力の補修結果をそのまま持ち出す。ブラウザ側の表示はこの JSON を読む。
#[test]
fn the_repaired_inputs_are_published_for_both_sides() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);
    analyzer.input_rows = (0..analyzer.features.len())
        .map(|_| (readable_row(), readable_row()))
        .collect();

    analyzer.ensure_events().expect("進む");

    assert_eq!(
        analyzer
            .events
            .as_ref()
            .and_then(|events| events.rounds.first())
            .map(|round| round.start_frame),
        Some(60),
        "入力があるときに演出の位置を使っていない"
    );
    let json = analyzer.tracked_json.as_deref().expect("入力の JSON");
    let parsed: serde_json::Value = serde_json::from_str(json).expect("JSON として読める");
    assert_eq!(
        parsed["p1"].as_array().map(Vec::len),
        Some(analyzer.features.len()),
        "自分側の入力が揃っていない"
    );
    assert_eq!(
        parsed["p2"].as_array().map(Vec::len),
        Some(analyzer.features.len()),
        "相手側の入力が揃っていない"
    );
}

/// レポートは JSON として読める形で返す。
#[test]
fn the_report_is_returned_as_readable_json() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50), marker(250)]);
    analyzer.ensure_events().expect("進む");

    let json = analyzer.report_json();
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("JSON として読める");

    assert!(parsed["error"].is_null(), "レポートが失敗している: {json}");
    assert!(parsed["totalFrames"].is_number() || parsed["total_frames"].is_number());
    assert!(parsed["summary"].is_string());
}

/// 演出が使えないときは、入力があっても HP からラウンドを割る。
#[test]
fn without_usable_markers_the_input_path_still_falls_back_to_health() {
    let mut analyzer = analyzer_with_features("p1");
    analyzer.fight_markers = Some(vec![marker(50)]);
    analyzer.input_rows = (0..analyzer.features.len())
        .map(|_| (readable_row(), readable_row()))
        .collect();

    analyzer.ensure_events().expect("進む");

    let events = analyzer.events.as_ref().expect("解析がある");
    assert_eq!(
        events.rounds.first().map(|round| round.start_frame),
        Some(0),
        "HP から割っていない"
    );
    assert!(
        !events.segments.iter().all(Vec::is_empty),
        "入力を捨てている"
    );
}

/// 演出が使えるかどうかで、HP の確定のしかたそのものが変わる。
/// ラウンド境界を HP から推し量る必要がなくなるため。
#[test]
fn the_markers_change_how_the_health_is_confirmed() {
    let confirmed = |markers: Vec<FightMarker>| {
        let mut analyzer = analyzer_with_features("p1");
        analyzer.fight_markers = Some(markers);
        analyzer.ensure_events().expect("進む");
        analyzer
            .features
            .iter()
            .map(|feature| feature.own_hp)
            .collect::<Vec<_>>()
    };

    assert_ne!(
        confirmed(vec![marker(50), marker(250)]),
        confirmed(vec![marker(50)]),
        "演出の有無で確定のしかたが変わっていない"
    );
}
