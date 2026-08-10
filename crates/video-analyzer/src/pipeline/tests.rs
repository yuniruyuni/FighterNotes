//! パイプライン結線に対するテスト。
//!
//! ここが持つ責務は「どの順で確定させるか」だけで、確定そのものの中身は
//! 下の層が持つ。順序を取り違えると wasm と CLI で結果が食い違うため、
//! 順序に意味があることと、入口ごとに同じ結線を通ることを固定する。

use super::*;
use crate::frame_features::FrameFeatures;
use match_event_layer::test_support::feat as feature;

/// HP はラウンド内で増えない。知覚層の一時的な読み違いで戻った値は、
/// 確定層が単調へ均す。ここを通していないと、被弾が回復として集計される。
#[test]
fn confirming_features_makes_the_health_monotonic_within_a_round() {
    let mut features = vec![
        feature(0, 1.0, 1.0),
        feature(1, 0.8, 1.0),
        // 1 フレームだけ読み違えて戻った値。
        feature(2, 0.95, 1.0),
        feature(3, 0.78, 1.0),
    ];

    finalize_features(&mut features);

    let own: Vec<f32> = features.iter().map(|f| f.own_hp).collect();
    assert!(
        own.windows(2).all(|pair| pair[1] <= pair[0] + 1e-6),
        "HP が増えている: {own:?}"
    );
}

/// 空の入力でも落ちない。動画の先頭が試合画面でない場合に通る道で、
/// ここで panic すると解析全体が止まる。
#[test]
fn an_empty_capture_survives_the_pipeline() {
    let mut features: Vec<FrameFeatures> = Vec::new();
    finalize_features(&mut features);
    assert!(features.is_empty());

    let report = analyze_features(&features, "p1");
    assert_eq!(report.rounds_detected, 0);
}

/// キャラクター名を伴わない入口と伴う入口は、同じ結線を通る。片方だけ
/// 手を入れると、CLI と browser で結果が変わる。
#[test]
fn the_entry_points_share_one_wiring() {
    let features: Vec<FrameFeatures> = (0..120)
        .map(|index| feature(index, 1.0 - index as f32 / 400.0, 1.0))
        .collect();

    let plain = analyze_features(&features, "p1");
    let with_context = analyze_features_with_context(
        &features,
        &crate::context::AnalysisContext::from_characters("p1", None, None),
    );

    assert_eq!(plain.rounds_detected, with_context.rounds_detected);
    assert_eq!(plain.ruleset_version, with_context.ruleset_version);
    assert_eq!(
        plain.cards.len(),
        with_context.cards.len(),
        "キャラクター名が無いときは同じ結果になる"
    );
}

/// 確定は何度通しても同じ結果になる。確定済みの値が viewer の表示と
/// イベント層の入力の唯一の源なので、通すたびに動くと両者がずれる。
#[test]
fn confirming_twice_changes_nothing() {
    let mut once: Vec<FrameFeatures> = (0..120)
        .map(|index| {
            let own = if index < 60 { 1.0 } else { 0.62 };
            feature(index, own, 1.0)
        })
        .collect();
    finalize_features(&mut once);
    let mut twice = once.clone();
    finalize_features(&mut twice);

    let read = |list: &[FrameFeatures]| -> Vec<(f32, f32)> {
        list.iter().map(|f| (f.own_hp, f.opponent_hp)).collect()
    };
    assert_eq!(read(&once), read(&twice));
}

/// `FIGHT` 表示で境界を決める入口は、確定の結果が別物になりうる。
/// 同じ処理へ委ねてしまうと、browser だけが持つ決定信号が捨てられる。
#[test]
fn the_fight_marker_entry_point_is_a_different_confirmation() {
    let source: Vec<FrameFeatures> = (0..120)
        .map(|index| feature(index, if index < 60 { 1.0 } else { 0.62 }, 1.0))
        .collect();

    let mut plain = source.clone();
    finalize_features(&mut plain);
    let mut marked = source.clone();
    // 区切りを一つも渡さない。境界を検出できないので、確定の仕方が変わる。
    finalize_features_with_fight_markers(&mut marked, &[], "p1");

    assert_eq!(plain.len(), marked.len(), "フレーム数は変えない");
}

/// ラウンドの区切りを渡すかどうかで、確定の仕方が変わる。渡した区切りを
/// 捨てると、境界をまたいだ HP が単調に潰される。
#[test]
fn the_fight_markers_change_what_gets_confirmed() {
    // 前半で削られ、後半のラウンドで全快へ戻る映像。
    let source: Vec<FrameFeatures> = (0..240)
        .map(|index| {
            let own = if index < 100 {
                1.0 - index as f32 / 200.0
            } else {
                1.0
            };
            feature(index, own, 1.0)
        })
        .collect();
    let markers = [crate::round_start::FightMarker {
        first_frame: 100,
        last_frame: 110,
        peak_frame: 105,
        peak_score: 1.0,
    }];

    let mut without = source.clone();
    finalize_features_with_fight_markers(&mut without, &[], "p1");
    let mut with = source.clone();
    finalize_features_with_fight_markers(&mut with, &markers, "p1");

    assert_ne!(
        without.iter().map(|f| f.own_hp).collect::<Vec<_>>(),
        with.iter().map(|f| f.own_hp).collect::<Vec<_>>(),
        "渡した区切りを捨てている"
    );
}

/// どちらの側から見た映像かは、区切り付きの確定でも効く。
#[test]
fn the_own_side_reaches_the_fight_marker_confirmation() {
    let source: Vec<FrameFeatures> = (0..240)
        .map(|index| feature(index, 1.0 - index as f32 / 400.0, 1.0))
        .collect();
    let markers = [crate::round_start::FightMarker {
        first_frame: 100,
        last_frame: 110,
        peak_frame: 105,
        peak_score: 1.0,
    }];

    let mut as_p1 = source.clone();
    finalize_features_with_fight_markers(&mut as_p1, &markers, "p1");
    let mut as_p2 = source.clone();
    finalize_features_with_fight_markers(&mut as_p2, &markers, "p2");

    assert_ne!(
        as_p1.iter().map(|f| f.own_hp).collect::<Vec<_>>(),
        as_p2.iter().map(|f| f.own_hp).collect::<Vec<_>>(),
        "側の指定が効いていない"
    );
}

/// 渡したフレーム列が実際に使われている。引数を取り違えたり捨てたりしても
/// 空の入力では気付けないので、長さの違う二本で結果が変わることを見る。
#[test]
fn the_frames_passed_in_are_the_frames_analysed() {
    let short: Vec<FrameFeatures> = (0..60).map(|index| feature(index, 1.0, 1.0)).collect();
    let long: Vec<FrameFeatures> = (0..600).map(|index| feature(index, 1.0, 1.0)).collect();

    assert_eq!(analyze_features(&short, "p1").total_frames, 60);
    assert_eq!(analyze_features(&long, "p1").total_frames, 600);
}

/// 自分の側の指定が、入力列の帰属に効く。HP は自分と相手の視点で持つので
/// 側を変えても打ち消し合うが、入力は画面の左右で持つため反転する。
#[test]
fn the_own_side_decides_whose_inputs_these_are() {
    use match_event_layer::test_support::up_inputs;

    let features = match_event_layer::test_support::synth_two_rounds();
    // P1 側だけがジャンプする入力を与える。
    let p1 = up_inputs(features.len(), &[(30, 45), (90, 105)]);
    let p2 = up_inputs(features.len(), &[]);

    let as_p1 = analyze_match(&features, &p1, &p2, None, "p1", None);
    let as_p2 = analyze_match(&features, &p1, &p2, None, "p2", None);

    let jumps = |report: &AdviceReport| report.input_stats.as_ref().map(|stats| stats.jumps);
    assert_eq!(jumps(&as_p1), Some(2), "自分側の入力が届いていない");
    assert_eq!(jumps(&as_p2), Some(0), "相手側の入力が届いていない");
}

/// フレームメーターを渡したかどうかが、そのまま「技の情報を使えるか」に
/// 効く。渡したものを捨てると、接触も発生も見えない解析になる。
#[test]
fn the_meter_timelines_reach_the_report() {
    use match_event_layer::test_support::{synth_run, synth_timeline};

    let features = match_event_layer::test_support::synth_two_rounds();
    let mut left = synth_run(0, "empty", 0, 199);
    left.extend(synth_run(200, "active", 200, 209));
    let mut right = synth_run(0, "empty", 0, 199);
    right.extend(synth_run(200, "stun", 200, 209));
    let left = synth_timeline(left);
    let right = synth_timeline(right);

    let without = analyze_match(&features, &[], &[], None, "p1", None);
    let with = analyze_match(&features, &[], &[], Some((&left, &right)), "p1", None);

    assert_eq!(without.coverage.own_meter_mapped_frames, 0);
    assert_eq!(without.coverage.opponent_meter_mapped_frames, 0);
    assert!(
        with.coverage.own_meter_mapped_frames > 0,
        "自分側のメーターを捨てている"
    );
    assert!(
        with.coverage.opponent_meter_mapped_frames > 0,
        "相手側のメーターを捨てている"
    );
}

/// context 付きの入口も同じ結線。片方だけ引数を落とすと、browser と CLI で
/// 見えるものが変わる。
#[test]
fn the_context_entry_point_carries_the_same_arguments() {
    use match_event_layer::test_support::{synth_run, synth_timeline, up_inputs};

    let features = match_event_layer::test_support::synth_two_rounds();
    let p1 = up_inputs(features.len(), &[(30, 45), (90, 105)]);
    let p2 = up_inputs(features.len(), &[]);
    let mut left = synth_run(0, "empty", 0, 199);
    left.extend(synth_run(200, "active", 200, 209));
    let mut right = synth_run(0, "empty", 0, 199);
    right.extend(synth_run(200, "stun", 200, 209));
    let left = synth_timeline(left);
    let right = synth_timeline(right);
    let context = crate::context::AnalysisContext::from_characters("p1", None, None);

    let jumps = |own: &[_], opponent: &[_]| {
        analyze_match_with_context(&features, own, opponent, None, &context)
            .input_stats
            .as_ref()
            .map(|stats| stats.jumps)
    };
    let with_meter =
        analyze_match_with_context(&features, &[], &[], Some((&left, &right)), &context);

    assert_eq!(jumps(&p1, &p2), Some(2), "自分側の入力が届いていない");
    assert_eq!(jumps(&p2, &p1), Some(0), "相手側の入力が届いていない");
    assert!(
        with_meter.coverage.own_meter_mapped_frames > 0,
        "メーターが届いていない"
    );
    assert!(
        with_meter.coverage.opponent_meter_mapped_frames > 0,
        "相手側のメーターが届いていない"
    );
}

/// 引数の無い入口も、渡したフレーム列をそのまま解析する。
///
/// この入口は入力もメーターも渡さない。HP は自分と相手の視点で持つので、
/// 側の指定は写像した先で打ち消し合い、結果には現れない。効くのは
/// 入力かメーターを伴う入口だけ。
#[test]
fn the_plain_entry_points_use_the_frames_they_are_given() {
    let features: Vec<FrameFeatures> = (0..600)
        .map(|index| feature(index, if index < 300 { 1.0 } else { 0.4 }, 1.0))
        .collect();
    let context = crate::context::AnalysisContext::from_characters("p1", None, None);

    let plain = analyze_features(&features, "p1");
    let via_context = analyze_features_with_context(&features, &context);

    assert_eq!(plain.total_frames, 600, "渡したフレーム列を使っていない");
    assert_eq!(
        via_context.total_frames, 600,
        "context 側がフレーム列を使っていない"
    );
    assert_eq!(plain.rounds_detected, via_context.rounds_detected);
    assert_eq!(
        analyze_features(&features[..120], "p1").total_frames,
        120,
        "渡した長さと結果が噛み合っていない"
    );
}

/// キャラクター名は確反の技名列挙に使う。渡した名前が捨てられていないことを、
/// 名前の有無で結果が変わることで見る。
#[test]
fn the_character_name_reaches_the_report() {
    let features: Vec<FrameFeatures> = (0..600)
        .map(|index| feature(index, if index < 300 { 1.0 } else { 0.4 }, 1.0))
        .collect();

    let unnamed = analyze_match(&features, &[], &[], None, "p1", None);
    let named = analyze_match(&features, &[], &[], None, "p1", Some("LUKE"));

    assert_eq!(
        unnamed.total_frames, named.total_frames,
        "映像は同じものを見ている"
    );
    assert_eq!(
        unnamed.ruleset_version, named.ruleset_version,
        "版数は名前で変わらない"
    );
}
