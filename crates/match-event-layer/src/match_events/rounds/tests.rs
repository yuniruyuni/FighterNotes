//! ラウンドの切れ目を見つけるところに対するテスト。
//!
//! ラウンドの開始は「両者が全快で、その状態がしばらく続いている」瞬間。
//! ただし全快のバーはラウンド前の演出にも映る。リプレイ冒頭のキャラクター
//! 紹介やイントロでも、両者のゲージは満タンで表示されている。
//!
//! そこで、二つの全快区間の間に実ダメージがあったかどうかを見る。
//! 無ければまだ戦いが始まっていないので、後の方を開始点に採る。

use super::*;
use match_event_model::test_support::feat;

/// HP 列から観測列を作る。
fn features_for(left: &[f32], right: &[f32]) -> Vec<FrameFeatures> {
    (0..left.len())
        .map(|frame| feat(frame as u32, left[frame], right[frame]))
        .collect()
}

fn detect(left: &[f32], right: &[f32]) -> Vec<RoundInfo> {
    let features = features_for(left, right);
    let hp = [left.to_vec(), right.to_vec()];
    detect_rounds_from_hp(&features, &hp)
}

/// 全快 → 削られて KO、を繰り返す HP 列を作る。
fn two_rounds() -> (Vec<f32>, Vec<f32>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    // R1: P2 が KO される
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(1.0, 100));
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(0.0, 100));
    // R2: P1 が KO される
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(1.0, 100));
    left.extend(std::iter::repeat_n(0.0, 100));
    right.extend(std::iter::repeat_n(1.0, 100));
    (left, right)
}

/// 全快の持続ごとにラウンドが始まる。
#[test]
fn each_run_of_full_health_starts_a_round() {
    let (left, right) = two_rounds();

    let rounds = detect(&left, &right);

    assert_eq!(rounds.len(), 2, "{rounds:?}");
    assert_eq!(rounds[0].start_frame, 0);
    assert_eq!(rounds[1].start_frame, 200);
    assert_eq!(rounds[0].winner, Some(1));
    assert_eq!(rounds[1].winner, Some(2));
}

/// 全快が一瞬しか映らなければラウンドの開始ではない。演出の切り替わりで
/// 一瞬だけ満タンのバーが見えることがある。
#[test]
fn a_flash_of_full_health_does_not_start_a_round() {
    let (mut left, mut right) = two_rounds();
    // R1 の途中に、全快が数フレームだけ映る。
    for frame in 150..155 {
        left[frame] = 1.0;
        right[frame] = 1.0;
    }

    let rounds = detect(&left, &right);

    assert_eq!(
        rounds.len(),
        2,
        "一瞬の全快をラウンドにしている: {rounds:?}"
    );
}

/// 全快がどこにも無ければラウンドは取れない。
#[test]
fn without_any_full_health_there_are_no_rounds() {
    let left = vec![0.5f32; 300];
    let right = vec![0.5f32; 300];

    assert!(detect(&left, &right).is_empty());
}

/// 間に実ダメージが無い全快の並びは、同じラウンドの開始前。後の方を
/// 開始点に採る。イントロ画面をラウンドの頭にしない。
#[test]
fn full_health_without_damage_in_between_moves_the_start_later() {
    let mut left = Vec::new();
    let mut right = Vec::new();
    // イントロ: 全快が 100 フレーム
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(1.0, 100));
    // 画面が切り替わって読めない 30 フレーム（全快ではない）
    left.extend(std::iter::repeat_n(0.9, 30));
    right.extend(std::iter::repeat_n(0.9, 30));
    // 本編: 全快が 100 フレーム → P2 が KO される
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(1.0, 100));
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(0.0, 100));

    let rounds = detect(&left, &right);

    assert_eq!(
        rounds.len(),
        1,
        "イントロを別ラウンドにしている: {rounds:?}"
    );
    assert_eq!(rounds[0].start_frame, 130, "後の全快を開始点にしていない");
}

/// 間に実ダメージがあれば、それは別のラウンド。
#[test]
fn real_damage_in_between_makes_it_a_separate_round() {
    let (left, right) = two_rounds();

    let rounds = detect(&left, &right);

    assert_eq!(rounds.len(), 2, "{rounds:?}");
}

/// 削られただけでは別ラウンドにしない。ラウンド間に少し削れた表示が
/// 残っていても、それは同じラウンドの続き。
#[test]
fn a_shallow_dip_does_not_split_the_rounds() {
    let mut left = Vec::new();
    let mut right = Vec::new();
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(1.0, 100));
    // 全快を割るが、実ダメージとは呼べない程度の落ち込み
    left.extend(std::iter::repeat_n(0.9, 30));
    right.extend(std::iter::repeat_n(0.9, 30));
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(1.0, 100));
    left.extend(std::iter::repeat_n(1.0, 100));
    right.extend(std::iter::repeat_n(0.0, 100));

    assert_eq!(detect(&left, &right).len(), 1);
}

/// 実ダメージの線引きはちょうどの値で決まる。閾値と同じだけ減った
/// だけでは別ラウンドにしない。
#[test]
fn a_dip_exactly_at_the_threshold_is_not_real_damage() {
    let between = |floor: f32| {
        let mut left = Vec::new();
        let mut right = Vec::new();
        left.extend(std::iter::repeat_n(1.0, 100));
        right.extend(std::iter::repeat_n(1.0, 100));
        left.extend(std::iter::repeat_n(1.0, 30));
        right.extend(std::iter::repeat_n(floor, 30));
        left.extend(std::iter::repeat_n(1.0, 100));
        right.extend(std::iter::repeat_n(1.0, 100));
        left.extend(std::iter::repeat_n(1.0, 100));
        right.extend(std::iter::repeat_n(0.0, 100));
        detect(&left, &right).len()
    };

    assert_eq!(
        between(MERGE_MIN_HP),
        1,
        "閾値ちょうどを実ダメージにしている"
    );
    assert_eq!(
        between(MERGE_MIN_HP - 0.01),
        2,
        "実ダメージを見落としている"
    );
}

/// 全快の持続はちょうどの長さから数える。
#[test]
fn a_full_health_run_of_exactly_the_minimum_starts_a_round() {
    let starts = |run: usize| {
        let mut left = Vec::new();
        let mut right = Vec::new();
        left.extend(std::iter::repeat_n(1.0, 100));
        right.extend(std::iter::repeat_n(1.0, 100));
        left.extend(std::iter::repeat_n(1.0, 100));
        right.extend(std::iter::repeat_n(0.0, 100));
        // 2 ラウンド目の頭になる全快の持続。
        left.extend(std::iter::repeat_n(1.0, run));
        right.extend(std::iter::repeat_n(1.0, run));
        left.extend(std::iter::repeat_n(0.0, 100));
        right.extend(std::iter::repeat_n(1.0, 100));
        detect(&left, &right).len()
    };

    assert_eq!(starts(FULL_MIN_RUN), 2, "ちょうどの持続を落としている");
    assert_eq!(starts(FULL_MIN_RUN - 1), 1, "短すぎる全快を開始にしている");
}

// ── FIGHT 表示から割る場合 ───────────────────────────────────────────────

use crate::round_start::FightMarker;

fn marker(first_frame: u32) -> FightMarker {
    FightMarker {
        first_frame,
        last_frame: first_frame + 5,
        peak_frame: first_frame + 2,
        peak_score: 1.0,
    }
}

/// FIGHT の表示があれば、HP を使わずにそこで割る。
#[test]
fn the_fight_banner_decides_the_boundaries() {
    let (left, right) = two_rounds();
    let features = features_for(&left, &right);
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0), marker(190)]);

    assert_eq!(rounds.len(), 2);
    assert_eq!(rounds[0].start_frame, 5, "安定表示の末尾から始めていない");
    assert_eq!(rounds[1].start_frame, 195);
    // KO が先に来るので、終端は次の表示の手前ではなく KO 演出の末尾。
    assert!(rounds[0].end_frame < 190, "次のラウンドへ食い込んでいる");
    assert_eq!(rounds[0].winner, Some(1));
    assert_eq!(rounds[1].winner, Some(2));
}

/// KO を読み取れないラウンドは、次の表示の手前まで。
#[test]
fn a_round_without_a_knockout_runs_up_to_the_next_banner() {
    let left = vec![1.0f32; 400];
    let mut right = vec![1.0f32; 400];
    for value in &mut right[100..] {
        *value = 0.5;
    }
    let features = features_for(&left, &right);
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0), marker(190)]);

    assert_eq!(rounds[0].end_frame, 189, "次の表示の手前で切っていない");
    assert_eq!(rounds[0].winner, Some(1), "残 HP で勝者を決めていない");
}

/// FIGHT の表示が無ければ何も割れない。
#[test]
fn without_a_banner_nothing_is_split() {
    let (left, right) = two_rounds();
    let features = features_for(&left, &right);
    let hp = [left, right];

    assert!(detect_rounds_from_fight_markers(&features, &hp, &[]).is_empty());
}

/// 観測が無ければ何も割れない。
#[test]
fn without_any_frame_nothing_is_split() {
    let hp = [Vec::new(), Vec::new()];

    assert!(detect_rounds_from_fight_markers(&[], &hp, &[marker(0)]).is_empty());
}

// ── 終端の決め方 ─────────────────────────────────────────────────────────

/// KO の演出を少し含めたところで切る。倒れた瞬間で切ると、最後の一撃の
/// クリップが途中で終わる。
#[test]
fn a_round_ends_a_little_after_the_knockout() {
    let left = vec![1.0f32; 600];
    let mut right = vec![1.0f32; 600];
    for value in &mut right[200..] {
        *value = 0.0;
    }
    let features = features_for(&left, &right);
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0)]);

    assert_eq!(rounds[0].end_frame, 200 + KO_MIN_RUN as u32 + 45);
    assert_eq!(rounds[0].p2_hp_end, 0.0, "KO 時点の HP を終値にしていない");
}

/// ゼロが一瞬しか続かなければ KO ではない。演出の切り替わりで
/// バーが一瞬消えることがある。KO なら演出の分だけで終わり、
/// そうでなければ最後まで続く。
#[test]
fn a_flash_of_zero_health_is_not_a_knockout() {
    let ending = |zero_frames: usize| {
        let left = vec![1.0f32; 600];
        let mut right = vec![1.0f32; 600];
        for value in &mut right[200..200 + zero_frames] {
            *value = 0.0;
        }
        let features = features_for(&left, &right);
        let hp = [left, right];
        detect_rounds_from_fight_markers(&features, &hp, &[marker(0)])[0].end_frame
    };

    assert_eq!(ending(KO_MIN_RUN - 1), 599, "一瞬のゼロを KO にしている");
    assert_eq!(
        ending(KO_MIN_RUN),
        200 + KO_MIN_RUN as u32 + 45,
        "続いたゼロを KO にしていない"
    );
}

/// 長さの見方は左右で同じ。
#[test]
fn the_knockout_length_is_read_the_same_on_both_sides() {
    let ending = |ko_side: usize, zero_frames: usize| {
        let mut sides = [vec![1.0f32; 600], vec![1.0f32; 600]];
        for value in &mut sides[ko_side][200..200 + zero_frames] {
            *value = 0.0;
        }
        let features = features_for(&sides[0], &sides[1]);
        detect_rounds_from_fight_markers(&features, &sides, &[marker(0)])[0].end_frame
    };

    assert_eq!(
        ending(0, KO_MIN_RUN - 1),
        599,
        "P1 の一瞬のゼロを KO にしている"
    );
    assert_eq!(ending(0, KO_MIN_RUN), 200 + KO_MIN_RUN as u32 + 45);
}

/// ラウンドの末尾ぴったりで倒れても KO。最後の一撃を取りこぼさない。
#[test]
fn a_knockout_that_ends_exactly_at_the_boundary_still_counts() {
    let hard_end = 300usize;
    let start = hard_end - KO_MIN_RUN;
    let left = vec![1.0f32; 600];
    let mut right = vec![1.0f32; 600];
    for value in &mut right[start..] {
        *value = 0.0;
    }
    let features = features_for(&left, &right);
    let hp = [left, right];

    let rounds =
        detect_rounds_from_fight_markers(&features, &hp, &[marker(0), marker(hard_end as u32 + 1)]);

    assert_eq!(
        rounds[0].winner,
        Some(1),
        "末尾ぴったりの KO を落としている"
    );
}

/// 開始が終端と同じフレームのラウンドは残す。1 フレームでもラウンドの
/// 中に入っていれば、そこで起きたことを捨てない。
#[test]
fn a_round_that_is_a_single_frame_is_kept() {
    let left = vec![1.0f32; 600];
    let right = vec![1.0f32; 600];
    let features = features_for(&left, &right);
    let hp = [left, right];

    // 1 つ目の表示の末尾（=開始）と、2 つ目の表示の直前が同じフレーム。
    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(100), marker(106)]);

    assert_eq!(rounds.len(), 2, "{rounds:?}");
    assert_eq!(rounds[0].start_frame, 105);
    assert_eq!(rounds[0].end_frame, 105);
}

/// 両者同時に倒れれば勝者は決まらない。
#[test]
fn a_double_knockout_has_no_winner() {
    let mut left = vec![1.0f32; 600];
    let mut right = vec![1.0f32; 600];
    for value in &mut left[200..] {
        *value = 0.0;
    }
    for value in &mut right[200..] {
        *value = 0.0;
    }
    let features = features_for(&left, &right);
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0)]);

    assert_eq!(rounds[0].winner, None);
}

/// 倒れた側が勝者ではない。左右のどちらでも。
#[test]
fn the_side_that_fell_is_the_loser() {
    let make = |ko_side: usize| {
        let mut sides = [vec![1.0f32; 600], vec![1.0f32; 600]];
        for value in &mut sides[ko_side][200..] {
            *value = 0.0;
        }
        let features = features_for(&sides[0], &sides[1]);
        detect_rounds_from_fight_markers(&features, &sides, &[marker(0)])[0].winner
    };

    assert_eq!(make(0), Some(2));
    assert_eq!(make(1), Some(1));
}

/// KO が読めないラウンドの終値は、その区間で最も減ったところ。終端
/// フレームの読みだけを見ると、演出で戻ったバーを終値にしてしまう。
#[test]
fn without_a_knockout_the_end_health_is_the_lowest_seen() {
    let left = vec![1.0f32; 400];
    let mut right = vec![1.0f32; 400];
    for value in &mut right[100..200] {
        *value = 0.4;
    }
    for value in &mut right[200..] {
        *value = 0.9;
    }
    let features = features_for(&left, &right);
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0)]);

    assert!((rounds[0].p2_hp_end - 0.4).abs() < 1e-5, "{rounds:?}");
    assert!((rounds[0].p1_hp_end - 1.0).abs() < 1e-5);
    assert_eq!(rounds[0].winner, Some(1), "残 HP の多い側が勝ち");
}

/// 残り HP がほぼ同じなら勝敗を決めない。読み取りの揺れで勝者を
/// でっち上げない。
#[test]
fn an_even_finish_leaves_the_winner_unknown() {
    let left = vec![0.5f32; 400];
    let mut right = vec![0.5f32; 400];
    for value in &mut right[100..] {
        *value = 0.49;
    }
    let features = features_for(&left, &right);
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0)]);

    assert_eq!(rounds[0].winner, None);
}

/// 試合画面が途切れたところで終わる。リザルト画面まで含めない。
#[test]
fn the_round_stops_where_the_match_screen_stops() {
    let left = vec![1.0f32; 400];
    let mut right = vec![1.0f32; 400];
    for value in &mut right[100..] {
        *value = 0.5;
    }
    let mut features = features_for(&left, &right);
    for feature in &mut features[300..] {
        feature.is_match_screen = false;
    }
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0)]);

    assert_eq!(rounds[0].end_frame, 299);
}

/// 開始が終端より後ろに来る表示は、そのラウンドだけ捨てる。後ろの
/// 表示まで見るのをやめない。
#[test]
fn an_impossible_boundary_drops_only_that_round() {
    let left = vec![1.0f32; 600];
    let mut right = vec![1.0f32; 600];
    for value in &mut right[300..] {
        *value = 0.5;
    }
    let features = features_for(&left, &right);
    let hp = [left, right];

    // 2 つ目の表示が 1 つ目より前にある。
    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(100), marker(50)]);

    assert_eq!(rounds.len(), 1, "後ろの表示まで捨てている: {rounds:?}");
    assert_eq!(rounds[0].round_no, 2, "表示の順番と番号がずれている");
    assert_eq!(rounds[0].start_frame, 55);
}

/// 読み取りが荒れたところより後ろは、ラウンドの実体として扱わない。
/// 孤立した「読めているように見えるフレーム」はリザルト画面にも混ざる。
#[test]
fn an_isolated_readable_frame_does_not_extend_the_round() {
    let left = vec![1.0f32; 400];
    let mut right = vec![1.0f32; 400];
    for value in &mut right[100..] {
        *value = 0.5;
    }
    let mut features = features_for(&left, &right);
    // f150 以降は読み取りが荒れている。
    for feature in &mut features[150..] {
        feature.left_hp_raw_quality = 1.0;
        feature.right_hp_raw_quality = 1.0;
    }
    // ただし f180 の 1 フレームだけは読めているように見える。
    features[180].left_hp_raw_quality = 0.0;
    features[180].right_hp_raw_quality = 0.0;
    // 試合の HUD は f160 で一度切れる。
    for feature in &mut features[160..170] {
        feature.is_match_screen = false;
    }
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0)]);

    assert_eq!(
        rounds[0].end_frame, 159,
        "孤立した 1 フレームを終端の根拠にしている"
    );
}

/// 一度安定して読めた後は、同じ HUD が続く限り終端を伸ばす。SA の
/// 演出で片側のバーが長く隠れても、ラウンドはそこで終わっていない。
#[test]
fn the_round_follows_the_hud_past_the_last_readable_frame() {
    let left = vec![1.0f32; 400];
    let mut right = vec![1.0f32; 400];
    for value in &mut right[100..] {
        *value = 0.5;
    }
    let mut features = features_for(&left, &right);
    for feature in &mut features[150..] {
        feature.left_hp_raw_quality = 1.0;
    }
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0), marker(190)]);

    assert_eq!(
        rounds[0].end_frame, 189,
        "読めなくなった時点でラウンドを切っている"
    );
}

/// 終値は終端のフレームまで見て決める。伸ばした先で減った分を
/// 取りこぼさない。
#[test]
fn the_end_health_includes_the_last_frame_of_the_round() {
    let mut left = vec![1.0f32; 400];
    let mut right = vec![1.0f32; 400];
    // 終端のフレームでだけ、それまでより低い値になる。
    left[189] = 0.3;
    right[189] = 0.2;
    let features = features_for(&left, &right);
    let hp = [left, right];

    let rounds = detect_rounds_from_fight_markers(&features, &hp, &[marker(0), marker(190)]);

    assert_eq!(rounds[0].end_frame, 189);
    assert!((rounds[0].p1_hp_end - 0.3).abs() < 1e-5, "{rounds:?}");
    assert!((rounds[0].p2_hp_end - 0.2).abs() < 1e-5, "{rounds:?}");
}

/// 残り HP の差が読み取りの揺れ程度なら、勝敗は決めない。
#[test]
fn a_difference_within_the_reading_noise_leaves_the_winner_unknown() {
    let winner_for = |gap: f32| {
        let left = vec![0.5f32; 400];
        let right = vec![0.5f32 - gap; 400];
        let features = features_for(&left, &right);
        let hp = [left, right];
        detect_rounds_from_fight_markers(&features, &hp, &[marker(0)])[0].winner
    };

    assert_eq!(winner_for(0.019), None, "揺れの範囲で勝者を決めている");
    assert_eq!(winner_for(0.05), Some(1));
}
