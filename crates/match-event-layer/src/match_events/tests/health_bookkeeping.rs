//! HP 列をラウンドごとに整えるところに対するテスト。
//!
//! 体力ゲージには灰色の部分がある。ドライブインパクトで回復するし、
//! 演出でバーが覆われれば読み取り自体も揺れる。そのままではラウンドの
//! 途中で HP が増えたように見え、増えた分だけ次の被弾が水増しされる。
//!
//! ラウンドの中では HP は増えない、という一点だけを使って均す。
//! ラウンドをまたぐと全快に戻るので、均す範囲はラウンドの中に閉じる。

use super::support::*;
use crate::round_start::FightMarker;

/// 2 ラウンドの試合。R1 は P2 が、R2 は P1 が KO される。
fn two_rounds() -> Vec<FrameFeatures> {
    let mut features: Vec<FrameFeatures> = Vec::new();
    let push = |features: &mut Vec<FrameFeatures>, count: usize, left: f32, right: f32| {
        for _ in 0..count {
            let frame = features.len() as u32;
            features.push(feat(frame, left, right));
        }
    };
    push(&mut features, 100, 1.0, 1.0);
    for step in 1..=30 {
        let frame = features.len() as u32;
        features.push(feat(frame, 1.0, (1.0 - step as f32 / 30.0).max(0.0)));
    }
    push(&mut features, 80, 1.0, 0.0);
    push(&mut features, 100, 1.0, 1.0);
    for step in 1..=30 {
        let frame = features.len() as u32;
        features.push(feat(frame, (1.0 - step as f32 / 30.0).max(0.0), 1.0));
    }
    push(&mut features, 80, 0.0, 1.0);
    features
}

fn build(features: &[FrameFeatures]) -> MatchEvents {
    build_match_events(features, &[], &[], None, "p1")
}

/// ラウンドの途中で HP が戻って見えても、戻さない。
#[test]
fn health_never_climbs_back_inside_a_round() {
    let mut features = two_rounds();
    // KO 後の演出中に、読み取りが半分まで戻ったことにする。
    for feature in &mut features[150..160] {
        feature.opponent_hp = 0.5;
    }

    let events = build(&features);

    assert_eq!(events.rounds.len(), 2, "{:?}", events.rounds);
    assert_eq!(events.hp[1][155], 0.0, "ラウンド内で HP を戻している");
}

/// 次のラウンドは全快から始まる。前のラウンドの底で抑え込まない。
#[test]
fn the_next_round_starts_from_full_health() {
    let events = build(&two_rounds());
    let second = events.rounds[1].start_frame as usize;

    assert_eq!(
        events.hp[1][second + 10],
        1.0,
        "前のラウンドの HP で次のラウンドを抑えている"
    );
}

/// ラウンドの最後のフレームまで均す。終端が抜けると、そこだけ
/// 戻った値が残る。
#[test]
fn the_last_frame_of_a_round_is_smoothed_too() {
    let boundary = build(&two_rounds()).rounds[0].end_frame as usize;
    let mut features = two_rounds();
    features[boundary].opponent_hp = 0.5;

    let events = build(&features);

    assert_eq!(
        events.hp[1][boundary], 0.0,
        "ラウンド終端のフレームを均していない"
    );
}

/// 倒れた側の HP はゼロのまま。「読めなかった」と混同しない。
#[test]
fn a_knocked_out_side_stays_at_zero() {
    let events = build(&two_rounds());

    assert_eq!(events.hp[1][200], 0.0, "P2 の KO を全快にしている");
    assert_eq!(events.hp[0][400], 0.0, "P1 の KO を全快にしている");
}

/// 冒頭の読めなかった HP は全快として扱う。試合前のロゴやフェードで
/// バーがまだ出ていないだけで、削れているわけではない。
#[test]
fn unreadable_health_at_the_start_counts_as_full() {
    let mut features = two_rounds();
    for feature in &mut features[0..20] {
        feature.own_hp = -1.0;
        feature.opponent_hp = -1.0;
    }

    let events = build(&features);

    assert_eq!(events.hp[0][10], 1.0);
    assert_eq!(events.hp[1][10], 1.0);
    assert_eq!(events.rounds.len(), 2, "{:?}", events.rounds);
}

/// 自分が P2 側なら、観測の左右も入れ替わる。
#[test]
fn the_sides_follow_who_the_viewer_is() {
    let features = two_rounds();

    let as_p1 = build(&features);
    let as_p2 = build_match_events(&features, &[], &[], None, "p2");

    assert_eq!(as_p1.hp[0], as_p2.hp[1], "左右を入れ替えていない");
    assert_eq!(as_p1.hp[1], as_p2.hp[0], "左右を入れ替えていない");
    assert_eq!(as_p1.rounds[0].winner, Some(1));
    assert_eq!(as_p2.rounds[0].winner, Some(2));
}

// ── 実ラウンドの選別 ─────────────────────────────────────────────────────

/// 被弾が一度も無い区間はラウンドではない。リプレイ冒頭のイントロや
/// キャラクター選択の画面にも全快のバーが映る。
#[test]
fn a_stretch_without_any_damage_is_not_a_round() {
    let mut features = Vec::new();
    // 何も起きない 200 フレーム。
    for frame in 0..200u32 {
        features.push(feat(frame, 1.0, 1.0));
    }
    let offset = features.len() as u32;
    for feature in two_rounds() {
        let mut moved = feature;
        moved.frame_index += offset;
        features.push(moved);
    }
    // 開始位置は FIGHT 表示で確定させ、HP を使わずに 3 つ区切る。
    let markers = [0u32, offset, offset + 210]
        .iter()
        .map(|&first_frame| FightMarker {
            first_frame,
            last_frame: first_frame + 5,
            peak_frame: first_frame + 2,
            peak_score: 1.0,
        })
        .collect::<Vec<_>>();

    let context = crate::context::AnalysisContext::new("p1");
    let events = build_match_events_with_context_and_fight_markers(
        &features,
        &[],
        &[],
        None,
        &context,
        &markers,
    );

    assert_eq!(events.rounds.len(), 2, "{:?}", events.rounds);
    assert_eq!(
        events.rounds.iter().map(|r| r.round_no).collect::<Vec<_>>(),
        vec![1, 2],
        "残ったラウンドに番号を振り直していない"
    );
    assert!(
        events.damage.iter().all(|d| d.round_no <= 2),
        "被弾の番号を振り直していない: {:?}",
        events.damage.iter().map(|d| d.round_no).collect::<Vec<_>>()
    );
    assert!(
        events.damage.iter().any(|d| d.round_no == 1),
        "1 ラウンド目の被弾が消えている"
    );
    assert!(
        events.damage.iter().any(|d| d.round_no == 2),
        "2 ラウンド目の被弾が消えている"
    );
}
