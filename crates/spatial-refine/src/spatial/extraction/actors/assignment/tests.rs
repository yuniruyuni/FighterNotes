//! 動きの塊を二人へ割り当てる規則に対するテスト。
//!
//! 画面には動くものが複数ある。二人のキャラクター、飛び道具、演出。
//! そのうちどれが誰なのかを、位置の連続性と大きさから決める。
//!
//! 取り違えると、以降の距離・向き・重なりの判断が全部相手のものになる。
//! 間違いは静かに伝播して、届いていない技が「届いた」ことになる。

use super::*;
use crate::spatial::SpatialRect;

/// 指定の位置と大きさを持つ動きの塊。
///
/// 足元（下辺の中央）が位置の基準になる。キャラクターは足で地面に
/// 立っているので、体の中心より足元の方が動きに対して安定する。
fn region(left: f32, right: f32, bottom: f32, changed_cells: u32) -> MotionRegion {
    MotionRegion {
        bounds: SpatialRect::new(left, bottom - 0.2, right, bottom),
        changed_cells,
        energy: changed_cells as u64 * 100,
        effect_cells: 0,
        effect_x_sum: 0.0,
        effect_y_sum: 0.0,
    }
}

fn config() -> SpatialConfig {
    SpatialConfig::default()
}

// ── 最初の割り当て ───────────────────────────────────────────────────────

/// 何も分かっていない状態では、左が P1、右が P2。
#[test]
fn without_any_hint_the_left_actor_is_the_first_player() {
    let regions = vec![region(0.6, 0.7, 0.9, 100), region(0.2, 0.3, 0.9, 100)];

    let tracks = initial_tracks(&regions, &[0, 1], 0, [false, false], &config())
        .expect("二人を割り当てられる");

    assert!(
        (tracks[0].anchor.x - 0.25).abs() < 1e-6,
        "左を P1 にしていない"
    );
    assert!((tracks[1].anchor.x - 0.65).abs() < 1e-6);
}

/// 近すぎる二つは同じキャラクターの一部かもしれない。二人には
/// 割り当てない。
#[test]
fn two_regions_too_close_together_are_not_two_actors() {
    let apart = vec![region(0.20, 0.30, 0.9, 100), region(0.32, 0.42, 0.9, 100)];
    let together = vec![region(0.20, 0.30, 0.9, 100), region(0.31, 0.41, 0.9, 100)];

    assert!(
        initial_tracks(&apart, &[0, 1], 0, [false, false], &config()).is_some(),
        "離れた二つを割り当てられていない"
    );
    assert!(
        initial_tracks(&together, &[0, 1], 0, [false, false], &config()).is_none(),
        "近すぎる二つを二人にしている"
    );
}

/// 候補が三つ以上あれば、大きく動いた二つを選ぶ。飛び道具や演出は
/// キャラクターより小さく動く。
#[test]
fn the_two_largest_movements_are_the_actors() {
    let regions = vec![
        region(0.50, 0.55, 0.5, 10),
        region(0.20, 0.30, 0.9, 100),
        region(0.70, 0.80, 0.9, 90),
    ];

    let tracks = initial_tracks(&regions, &[0, 1, 2], 0, [false, false], &config())
        .expect("二人を割り当てられる");

    assert!(
        (tracks[0].anchor.x - 0.25).abs() < 1e-6,
        "小さい塊を人にしている"
    );
    assert!((tracks[1].anchor.x - 0.75).abs() < 1e-6);
}

/// 大きさが同じなら、動いた量の多い方を選ぶ。
#[test]
fn equal_sizes_are_broken_by_how_much_moved() {
    let mut regions = vec![
        region(0.20, 0.30, 0.9, 100),
        region(0.70, 0.80, 0.9, 100),
        region(0.45, 0.55, 0.9, 100),
    ];
    regions[2].energy = 1;

    let tracks = initial_tracks(&regions, &[0, 1, 2], 0, [false, false], &config())
        .expect("二人を割り当てられる");

    assert!((tracks[0].anchor.x - 0.25).abs() < 1e-6);
    assert!((tracks[1].anchor.x - 0.75).abs() < 1e-6);
}

// ── 位置が入れ替わっている場面 ───────────────────────────────────────────

/// 短い窓は、既に位置が入れ替わった状態から始まることがある。片側だけが
/// 空中にいると分かっていて、候補も空中と地上に分かれているなら、
/// 左が P1 という仮定より、その意味を優先する。
#[test]
fn a_jump_hint_overrides_the_left_is_first_assumption() {
    // 左が空中、右が地上。P2 だけが飛んでいると分かっている。
    let regions = vec![region(0.20, 0.30, 0.5, 100), region(0.70, 0.80, 0.9, 100)];

    let tracks = initial_tracks(&regions, &[0, 1], 0, [false, true], &config())
        .expect("二人を割り当てられる");

    assert!(
        (tracks[1].anchor.x - 0.25).abs() < 1e-6,
        "飛んでいる側を P2 にしていない"
    );
    assert!((tracks[0].anchor.x - 0.75).abs() < 1e-6);
}

/// 飛んでいるのが P1 側なら、割り当ても逆になる。
#[test]
fn the_hint_works_for_the_first_player_too() {
    let regions = vec![region(0.20, 0.30, 0.5, 100), region(0.70, 0.80, 0.9, 100)];

    let tracks = initial_tracks(&regions, &[0, 1], 0, [true, false], &config())
        .expect("二人を割り当てられる");

    assert!((tracks[0].anchor.x - 0.25).abs() < 1e-6);
    assert!((tracks[1].anchor.x - 0.75).abs() < 1e-6);
}

/// 両方が飛んでいる、あるいは両方が地上なら、意味では分けられない。
/// 左が P1 の仮定に戻る。
#[test]
fn the_hint_does_nothing_when_it_applies_to_both() {
    let regions = vec![region(0.20, 0.30, 0.5, 100), region(0.70, 0.80, 0.9, 100)];

    let tracks = initial_tracks(&regions, &[0, 1], 0, [true, true], &config())
        .expect("二人を割り当てられる");

    assert!(
        (tracks[0].anchor.x - 0.25).abs() < 1e-6,
        "左が P1 に戻っていない"
    );
}

/// 候補が空中と地上に分かれていなければ、意味では分けられない。
#[test]
fn the_hint_does_nothing_when_both_candidates_are_grounded() {
    let regions = vec![region(0.20, 0.30, 0.9, 100), region(0.70, 0.80, 0.9, 100)];

    let tracks = initial_tracks(&regions, &[0, 1], 0, [false, true], &config())
        .expect("二人を割り当てられる");

    assert!((tracks[0].anchor.x - 0.25).abs() < 1e-6);
}

// ── 続きの割り当て ───────────────────────────────────────────────────────

fn track_at(x: f32, y: f32) -> ActorTrack {
    from_region(&region(x - 0.05, x + 0.05, y, 100), 0, 0.72)
}

/// 前のフレームの位置に近い塊を、その人のものとして続ける。
#[test]
fn the_nearest_region_continues_the_track() {
    let p1 = track_at(0.30, 0.9);
    let p2 = track_at(0.70, 0.9);
    let regions = vec![region(0.68, 0.78, 0.9, 100), region(0.28, 0.38, 0.9, 100)];

    let assigned = assign_regions(
        [Some(&p1), Some(&p2)],
        [false, false],
        [false, false],
        &regions,
        &[0, 1],
        &config(),
    );

    assert_eq!(assigned, [Some(1), Some(0)], "近い方を選んでいない");
}

/// 同じ塊を二人に割り当てない。二人が一つに見えているとき、片方は
/// 見失ったことにする。
#[test]
fn one_region_is_never_given_to_both() {
    let p1 = track_at(0.48, 0.9);
    let p2 = track_at(0.52, 0.9);
    let regions = vec![region(0.45, 0.55, 0.9, 100)];

    let assigned = assign_regions(
        [Some(&p1), Some(&p2)],
        [false, false],
        [false, false],
        &regions,
        &[0],
        &config(),
    );

    assert_ne!(assigned[0], assigned[1], "同じ塊を二人に割り当てている");
    assert!(assigned[0].is_some() || assigned[1].is_some());
}

/// 大きく離れた塊は、その人のものではない。瞬間移動は起きない。
#[test]
fn a_region_too_far_away_is_not_the_same_actor() {
    let p1 = track_at(0.30, 0.9);
    let regions = vec![region(0.65, 0.75, 0.9, 100)];

    let assigned = assign_regions(
        [Some(&p1), None],
        [false, false],
        [false, false],
        &regions,
        &[0],
        &config(),
    );

    assert_eq!(assigned[0], None, "離れすぎた塊を同じ人にしている");
}

/// 画面が入れ替わったと分かっていれば、離れた塊でも続きとして受け入れる。
#[test]
fn an_explicit_discontinuity_allows_a_distant_reacquire() {
    let p1 = track_at(0.30, 0.9);
    let regions = vec![region(0.65, 0.75, 0.9, 100)];

    let assigned = assign_regions(
        [Some(&p1), None],
        [true, false],
        [false, false],
        &regions,
        &[0],
        &config(),
    );

    assert_eq!(assigned[0], Some(0), "入れ替わりを受け入れていない");
}

/// 地上にいた人が急に空中の塊になるのは、飛んだと分かっているときだけ。
/// 分かっていなければ、別の何かが動いている。
#[test]
fn leaving_the_ground_needs_a_jump_hint() {
    let p1 = track_at(0.30, 0.9);
    let regions = vec![region(0.28, 0.38, 0.62, 100)];

    let without_hint = assign_regions(
        [Some(&p1), None],
        [false, false],
        [false, false],
        &regions,
        &[0],
        &config(),
    );
    let with_hint = assign_regions(
        [Some(&p1), None],
        [false, false],
        [true, false],
        &regions,
        &[0],
        &config(),
    );

    assert_eq!(without_hint[0], None, "ヒント無しで空中へ飛ばしている");
    assert_eq!(with_hint[0], Some(0), "ヒントがあるのに受け入れていない");
}

/// 地面に接している塊を優先する。キャラクターは足で立っている。
#[test]
fn a_region_touching_the_ground_is_preferred() {
    let p1 = track_at(0.30, 0.9);
    let regions = vec![region(0.28, 0.38, 0.80, 100), region(0.28, 0.38, 0.88, 100)];

    let assigned = assign_regions(
        [Some(&p1), None],
        [false, false],
        [false, false],
        &regions,
        &[0, 1],
        &config(),
    );

    assert_eq!(assigned[0], Some(1), "地面に接した塊を選んでいない");
}

/// 大きい塊を優先する。小さい塊は演出の破片。
#[test]
fn a_larger_region_is_preferred_at_the_same_distance() {
    let p1 = track_at(0.30, 0.9);
    let regions = vec![region(0.28, 0.38, 0.9, 10), region(0.28, 0.38, 0.9, 200)];

    let assigned = assign_regions(
        [Some(&p1), None],
        [false, false],
        [false, false],
        &regions,
        &[0, 1],
        &config(),
    );

    assert_eq!(assigned[0], Some(1), "小さい塊を選んでいる");
}

/// 追っていない側には何も割り当てない。
#[test]
fn a_side_without_a_track_gets_nothing() {
    let regions = vec![region(0.28, 0.38, 0.9, 100)];

    let assigned = assign_regions(
        [None, None],
        [false, false],
        [false, false],
        &regions,
        &[0],
        &config(),
    );

    assert_eq!(assigned, [None, None]);
}

/// 候補が無ければ、二人とも見失う。
#[test]
fn no_candidates_means_both_are_lost() {
    let p1 = track_at(0.30, 0.9);
    let p2 = track_at(0.70, 0.9);

    let assigned = assign_regions(
        [Some(&p1), Some(&p2)],
        [false, false],
        [false, false],
        &[],
        &[],
        &config(),
    );

    assert_eq!(assigned, [None, None]);
}

/// 入れ替わりを待っている側は、見失ったままにしない。画面が入れ替わった
/// 直後に取り直せないと、その人を追えなくなる。二人が一つの塊に見えて
/// いるときは、待っている側が先に取る。
#[test]
fn a_side_expecting_a_discontinuity_takes_the_region_first() {
    let p1 = track_at(0.50, 0.9);
    let p2 = track_at(0.52, 0.9);
    let regions = vec![region(0.48, 0.58, 0.9, 100)];

    let waiting_on_the_second = assign_regions(
        [Some(&p1), Some(&p2)],
        [false, true],
        [false, false],
        &regions,
        &[0],
        &config(),
    );
    let waiting_on_the_first = assign_regions(
        [Some(&p1), Some(&p2)],
        [true, false],
        [false, false],
        &regions,
        &[0],
        &config(),
    );

    assert_eq!(
        waiting_on_the_second,
        [None, Some(0)],
        "待っている側が取れていない"
    );
    assert_eq!(waiting_on_the_first, [Some(0), None]);
}

#[test]
fn initial_tracks_keep_the_frame_and_include_the_exact_separation() {
    let regions = vec![region(-0.05, 0.05, 0.9, 100), region(0.07, 0.17, 0.9, 100)];

    let tracks = initial_tracks(&regions, &[0, 1], 42, [false, false], &config())
        .expect("exactly separated actors should initialize");

    assert_eq!(tracks[0].last_observed_frame, 42);
    assert_eq!(tracks[1].last_observed_frame, 42);
    assert_eq!(tracks[0].confidence, 0.55);
    assert_eq!(tracks[1].confidence, 0.55);
}

#[test]
fn the_second_side_is_scored_even_when_the_first_has_no_track() {
    let p2 = track_at(0.7, 0.9);
    let regions = vec![region(0.66, 0.76, 0.9, 100)];

    let assigned = assign_regions(
        [None, Some(&p2)],
        [false, false],
        [false, false],
        &regions,
        &[0],
        &config(),
    );

    assert_eq!(assigned, [None, Some(0)]);
}

#[test]
fn movement_limits_are_inclusive_and_each_axis_is_required() {
    let mut limits = config();
    limits.actor_ground_y = 1.0;
    limits.max_track_dx = 0.25;
    limits.max_track_dy = 0.25;
    let track = track_at(0.25, 0.5);
    let exact_x = region(0.45, 0.55, 0.5, 100);
    let exact_y = region(0.20, 0.30, 0.75, 100);
    let beyond_y = region(0.20, 0.30, 0.76, 100);

    assert_eq!(
        assign_regions(
            [Some(&track), None],
            [false, false],
            [false, false],
            &[exact_x],
            &[0],
            &limits,
        )[0],
        Some(0)
    );
    assert_eq!(
        assign_regions(
            [Some(&track), None],
            [false, false],
            [false, false],
            &[exact_y],
            &[0],
            &limits,
        )[0],
        Some(0)
    );
    assert_eq!(
        assign_regions(
            [Some(&track), None],
            [false, false],
            [false, false],
            &[beyond_y],
            &[0],
            &limits,
        )[0],
        None
    );
}

#[test]
fn rejecting_one_airborne_candidate_does_not_hide_a_later_grounded_one() {
    let track = track_at(0.3, 0.9);
    let regions = vec![region(0.28, 0.38, 0.7, 100), region(0.28, 0.38, 0.9, 100)];

    let assigned = assign_regions(
        [Some(&track), None],
        [false, false],
        [false, false],
        &regions,
        &[0, 1],
        &config(),
    );

    assert_eq!(assigned[0], Some(1));
}

#[test]
fn region_score_uses_distance_discontinuity_size_and_ground_terms() {
    let scored_region = region(0.0, 0.1, 0.86, 100);
    let score = region_score(0.2, 0.1, &scored_region, &config());

    assert!((score - 0.275).abs() < 1e-6, "score={score}");

    let exact_dx = region_score(config().max_track_dx, 0.0, &scored_region, &config());
    let expected = config().max_track_dx * 1.8 - 0.12 - 0.18;
    assert!((exact_dx - expected).abs() < 1e-6);
}

#[test]
fn the_airborne_hint_treats_the_ground_threshold_as_grounded_for_either_candidate() {
    let ground = config().actor_ground_y;
    let first_ground = vec![
        region(0.70, 0.80, ground, 100),
        region(0.20, 0.30, 0.60, 100),
    ];
    let tracks = initial_tracks(&first_ground, &[0, 1], 0, [false, true], &config()).unwrap();
    assert!((tracks[1].anchor.x - 0.25).abs() < 1e-6);

    let second_ground = vec![
        region(0.70, 0.80, 0.60, 100),
        region(0.20, 0.30, ground, 100),
    ];
    let tracks = initial_tracks(&second_ground, &[0, 1], 0, [true, false], &config()).unwrap();
    assert!((tracks[0].anchor.x - 0.75).abs() < 1e-6);
}

#[test]
fn staying_exactly_on_the_ground_threshold_needs_no_jump_hint() {
    let ground = config().actor_ground_y;
    let track = track_at(0.30, ground);
    let candidate = region(0.28, 0.38, ground, 100);

    assert_eq!(
        assign_regions(
            [Some(&track), None],
            [false, false],
            [false, false],
            &[candidate],
            &[0],
            &config(),
        )[0],
        Some(0)
    );
}

/// 接地の判定はどちらの側も境界を含む。トラックが基準ちょうどに立って
/// いれば地上であり、塊の足元が基準ちょうどに届いていれば空中ではない。
#[test]
fn ground_boundary_is_inclusive_on_both_sides() {
    let ground = config().actor_ground_y;

    // 基準ちょうどのトラックは地上扱いで、空中の塊にはヒントが要る。
    let at_ground = track_at(0.30, ground);
    let airborne = vec![region(0.28, 0.38, 0.42, 100)];
    let assigned = assign_regions(
        [Some(&at_ground), None],
        [false, false],
        [false, false],
        &airborne,
        &[0],
        &config(),
    );
    assert_eq!(assigned[0], None, "基準ちょうどを地上と数えていない");

    // 足元が基準ちょうどに届く塊は空中ではないので、ヒント無しで続く。
    let grounded_region = vec![region(0.28, 0.38, ground, 100)];
    let from_high = track_at(0.30, 0.9);
    let assigned = assign_regions(
        [Some(&from_high), None],
        [false, false],
        [false, false],
        &grounded_region,
        &[0],
        &config(),
    );
    assert_eq!(assigned[0], Some(0), "基準ちょうどの塊を空中にしている");
}
