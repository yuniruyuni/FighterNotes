//! 確定反撃が距離的に届いたかの判断に対するテスト。
//!
//! フレーム上の有利だけでは、長い技の先端をガードした場面を確反と
//! 断定できない。位置を見て初めて「届いた」と言える。
//!
//! ただし、どこまで言えるかは場面によって違う。反撃を出していない
//! 見逃しには技の情報が無いので、体が重なっていた場面しか断定できない。
//! 空振りには技を出した記録があるので、近〜中距離でも判断材料になる。

use super::*;

/// 同じ距離帯が続いた観測列。
fn bands(band: DistanceBand, count: usize) -> Vec<DistanceBand> {
    vec![band; count]
}

// ── 反撃を出さなかった見逃し ─────────────────────────────────────────────

/// 体が重なっていれば、どの技でも届く。断定してよい。
#[test]
fn overlapping_bodies_confirm_a_missed_punish() {
    let confirmed = reachability(PunishOutcome::Missed, &bands(DistanceBand::Overlap, 2));

    assert_eq!(confirmed, PunishReachability::Confirmed);
}

/// 重なりが一度しかなければ、たまたま近づいた瞬間かもしれない。
#[test]
fn a_single_overlapping_frame_is_not_enough() {
    let unknown = reachability(PunishOutcome::Missed, &bands(DistanceBand::Overlap, 1));

    assert_eq!(unknown, PunishReachability::Unknown);
}

/// 重なりの中に離れた瞬間が混じっていれば断定しない。押し合いで
/// 一瞬重なっただけかもしれない。
#[test]
fn a_mixture_with_any_separation_is_not_confirmed() {
    let mut mixed = bands(DistanceBand::Overlap, 3);
    mixed.push(DistanceBand::Close);

    assert_eq!(
        reachability(PunishOutcome::Missed, &mixed),
        PunishReachability::Unknown,
        "離れた瞬間を無視して断定している"
    );
}

#[test]
fn opposite_separation_bands_cannot_cancel_each_other() {
    let mixed = [
        DistanceBand::Overlap,
        DistanceBand::Overlap,
        DistanceBand::Mid,
        DistanceBand::Far,
    ];

    assert_eq!(
        reachability(PunishOutcome::Missed, &mixed),
        PunishReachability::Unknown
    );
}

/// 近距離でも、反撃を出していないなら断定しない。どの技なら届いたのかが
/// 分からない。
#[test]
fn close_spacing_alone_does_not_confirm_a_missed_punish() {
    let unknown = reachability(PunishOutcome::Missed, &bands(DistanceBand::Close, 5));

    assert_eq!(unknown, PunishReachability::Unknown);
}

/// 中〜遠距離が続いていれば、届かなかったと断定できる。
#[test]
fn mid_or_far_spacing_puts_a_missed_punish_out_of_range() {
    assert_eq!(
        reachability(PunishOutcome::Missed, &bands(DistanceBand::Mid, 2)),
        PunishReachability::OutOfRange
    );
    assert_eq!(
        reachability(PunishOutcome::Missed, &bands(DistanceBand::Far, 2)),
        PunishReachability::OutOfRange
    );
}

/// 一度でも重なっていれば、届かなかったとは言えない。
#[test]
fn one_overlapping_frame_blocks_the_out_of_range_verdict() {
    let mut mixed = bands(DistanceBand::Far, 5);
    mixed.push(DistanceBand::Overlap);

    assert_eq!(
        reachability(PunishOutcome::Missed, &mixed),
        PunishReachability::Unknown
    );
}

/// 近距離だけでは、届かなかったとも言えない。
#[test]
fn close_spacing_alone_does_not_put_it_out_of_range() {
    assert_eq!(
        reachability(PunishOutcome::Missed, &bands(DistanceBand::Close, 5)),
        PunishReachability::Unknown
    );
}

// ── 出したが届かなかった反撃 ─────────────────────────────────────────────

/// 空振りには技を出した記録がある。近〜中距離で安定していれば、
/// 届く位置にいたと断定してよい。
#[test]
fn stable_close_to_mid_spacing_confirms_a_whiffed_punish() {
    for band in [
        DistanceBand::Overlap,
        DistanceBand::Close,
        DistanceBand::Mid,
    ] {
        assert_eq!(
            reachability(PunishOutcome::WhiffFail, &bands(band, 2)),
            PunishReachability::Confirmed,
            "{band:?} を断定していない"
        );
    }
}

/// 一度でも遠ければ断定しない。技が届く距離だったのか分からなくなる。
#[test]
fn any_far_frame_blocks_confirming_a_whiffed_punish() {
    let mut mixed = bands(DistanceBand::Close, 5);
    mixed.push(DistanceBand::Far);

    assert_eq!(
        reachability(PunishOutcome::WhiffFail, &mixed),
        PunishReachability::Unknown
    );
}

/// 観測が足りなければ断定しない。
#[test]
fn a_single_frame_does_not_confirm_a_whiffed_punish() {
    assert_eq!(
        reachability(PunishOutcome::WhiffFail, &bands(DistanceBand::Close, 1)),
        PunishReachability::Unknown
    );
}

/// 遠距離だけが続いていれば、届かなかったと断定できる。
#[test]
fn only_far_spacing_puts_a_whiffed_punish_out_of_range() {
    assert_eq!(
        reachability(PunishOutcome::WhiffFail, &bands(DistanceBand::Far, 2)),
        PunishReachability::OutOfRange
    );
}

/// 近い瞬間が一度でもあれば、届かなかったとは言えない。
#[test]
fn one_close_frame_blocks_the_out_of_range_verdict() {
    let mut mixed = bands(DistanceBand::Far, 5);
    mixed.push(DistanceBand::Close);

    assert_eq!(
        reachability(PunishOutcome::WhiffFail, &mixed),
        PunishReachability::Unknown
    );
}

#[test]
fn close_and_mid_samples_cannot_cancel_when_deciding_a_whiff_is_out_of_range() {
    let mixed = [
        DistanceBand::Close,
        DistanceBand::Mid,
        DistanceBand::Far,
        DistanceBand::Far,
    ];

    assert_eq!(
        reachability(PunishOutcome::WhiffFail, &mixed),
        PunishReachability::Unknown
    );
}

/// 観測が無ければ何も言えない。
#[test]
fn without_any_observation_nothing_is_decided() {
    assert_eq!(
        reachability(PunishOutcome::Missed, &[]),
        PunishReachability::Unknown
    );
    assert_eq!(
        reachability(PunishOutcome::WhiffFail, &[]),
        PunishReachability::Unknown
    );
}

// ── 反撃が当たった場合 ───────────────────────────────────────────────────

/// 当たっているのだから届いている。観測を見るまでもない。
#[test]
fn a_landed_punish_is_confirmed_without_looking_at_the_distance() {
    assert_eq!(
        reachability(PunishOutcome::Success, &[]),
        PunishReachability::Confirmed
    );
    assert_eq!(
        reachability(PunishOutcome::Success, &bands(DistanceBand::Far, 10)),
        PunishReachability::Confirmed
    );
}

// ── 観測を集める範囲 ─────────────────────────────────────────────────────

use crate::spatial::{ActorObservation, SpatialPoint, SpatialRect};

/// 距離帯の分かる 1 フレーム分の観測。
fn observation(frame_index: u32, band: DistanceBand) -> SpatialObservation {
    let actor = |x: f32| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.05, 0.7, x + 0.05, 0.9),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    SpatialObservation {
        frame_index,
        p1: Some(actor(0.4)),
        p2: Some(actor(0.6)),
        screen_distance: Some(0.2),
        distance_band: Some(band),
        horizontal_order: None,
        projectile_candidates: Vec::new(),
        motion_regions: Vec::new(),
        contact: None,
        camera: None,
    }
}

/// 反撃の機会。
fn chance(outcome: PunishOutcome) -> PunishChance {
    PunishChance {
        frame: 100,
        side: 1,
        advantage: 10,
        outcome,
        origin: crate::match_events::PunishOrigin::BlockedMove,
        recovery_start_frame: 90,
        recovery_end_frame: 110,
        source_contact_frame: Some(90),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    }
}

/// 見る範囲はガードした瞬間から。反撃の起点だけを見ると、その間に
/// 離れていった動きが映らない。
#[test]
fn the_samples_start_at_the_block_that_created_the_chance() {
    let mut punishes = vec![chance(PunishOutcome::Missed)];
    let observations = vec![
        observation(88, DistanceBand::Overlap),
        observation(89, DistanceBand::Overlap),
        observation(120, DistanceBand::Far),
    ];

    refine(&mut punishes, &observations);

    assert_eq!(
        punishes[0].reachability,
        PunishReachability::Confirmed,
        "ガードした瞬間から見ていない"
    );
}

/// ガードの記録が無ければ、機会の起点から見る。
#[test]
fn without_a_block_record_the_samples_start_at_the_chance() {
    let mut punishes = vec![chance(PunishOutcome::Missed)];
    punishes[0].source_contact_frame = None;
    let observations = vec![
        observation(88, DistanceBand::Far),
        observation(89, DistanceBand::Far),
        observation(99, DistanceBand::Overlap),
        observation(100, DistanceBand::Overlap),
    ];

    refine(&mut punishes, &observations);

    assert_eq!(
        punishes[0].reachability,
        PunishReachability::Confirmed,
        "機会の起点より前まで見ている"
    );
}

/// 攻撃判定が出た時点まで見る。技を振り切るまでの距離が要る。
#[test]
fn the_samples_reach_the_attacks_active_frame() {
    let mut punishes = vec![chance(PunishOutcome::WhiffFail)];
    punishes[0].attack_active_frame = Some(140);
    let observations = vec![
        observation(141, DistanceBand::Close),
        observation(142, DistanceBand::Close),
    ];

    refine(&mut punishes, &observations);

    assert_eq!(
        punishes[0].reachability,
        PunishReachability::Confirmed,
        "攻撃判定の時点まで見ていない"
    );
}

/// 範囲の外の観測は使わない。
#[test]
fn observations_outside_the_window_are_not_used() {
    let mut punishes = vec![chance(PunishOutcome::Missed)];
    let observations = vec![
        observation(200, DistanceBand::Overlap),
        observation(201, DistanceBand::Overlap),
    ];

    refine(&mut punishes, &observations);

    assert_eq!(punishes[0].reachability, PunishReachability::Unknown);
}

/// 二人とも見えていない観測は使わない。片方しか追えていない場面の
/// 距離は信用できない。
#[test]
fn an_observation_missing_an_actor_is_not_used() {
    let mut punishes = vec![chance(PunishOutcome::Missed)];
    let mut observations = vec![
        observation(88, DistanceBand::Overlap),
        observation(89, DistanceBand::Overlap),
    ];
    observations[0].p2 = None;

    refine(&mut punishes, &observations);

    assert_eq!(punishes[0].reachability, PunishReachability::Unknown);
}

/// 当たった反撃は触らない。既に届いていると分かっている。
#[test]
fn a_landed_punish_is_left_alone() {
    let mut punishes = vec![chance(PunishOutcome::Success)];
    punishes[0].reachability = PunishReachability::Confirmed;
    let observations = vec![
        observation(88, DistanceBand::Far),
        observation(89, DistanceBand::Far),
    ];

    refine(&mut punishes, &observations);

    assert_eq!(punishes[0].reachability, PunishReachability::Confirmed);
}

// ── めり込みガードによる確認 ─────────────────────────────────────────────

fn guard_spark(x: f32, confidence: f32) -> crate::spatial::ContactObservation {
    crate::spatial::ContactObservation {
        center: SpatialPoint::new(x, 0.75),
        bounds: SpatialRect::new(x - 0.02, 0.7, x + 0.02, 0.8),
        effect_cells: 6,
        confidence,
    }
}

/// 攻撃側(P2)の、身長の物差しとして 2 進で正確に割れる立ち姿勢。
/// anchor 1/2、足元 7/8、頭 3/8 → 身長 1/2。
fn exact_attacker(top: f32) -> ActorObservation {
    ActorObservation {
        anchor: SpatialPoint::new(0.5, 0.875),
        bounds: SpatialRect::new(0.45, top, 0.55, 0.875),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    }
}

/// 距離帯では断定できない近距離の見逃しでも、ガードスパークが攻撃側の
/// 体から 1/2 身長以内(めり込み)なら、技は根元で当たっており最速の
/// 反撃が届いたと確認できる。境界ちょうども確認に入る。
#[test]
fn a_deep_guard_spark_confirms_a_missed_punish() {
    let mut punishes = vec![chance(PunishOutcome::Missed)];
    let mut observations = vec![
        observation(89, DistanceBand::Close),
        observation(91, DistanceBand::Close),
    ];
    observations[1].p2 = Some(exact_attacker(0.375));
    // anchor 0.5 から 0.25 = 身長 0.5 のちょうど半分。
    observations[1].contact = Some(guard_spark(0.75, 0.8));

    refine(&mut punishes, &observations);

    assert_eq!(punishes[0].reachability, PunishReachability::Confirmed);
}

/// 先端ガード・弱いスパーク・物差しにならない攻撃側の追跡は、どれも
/// 何も断定しない(Unknown のまま)。めり込みは確認専用で、先端を
/// 「届かない」へ倒すことはない。
#[test]
fn tip_and_unreliable_evidence_stay_unknown() {
    let run = |mutate: &dyn Fn(&mut Vec<SpatialObservation>, &mut PunishChance)| {
        let mut punishes = vec![chance(PunishOutcome::Missed)];
        let mut observations = vec![
            observation(89, DistanceBand::Close),
            observation(91, DistanceBand::Close),
        ];
        observations[1].p2 = Some(exact_attacker(0.375));
        observations[1].contact = Some(guard_spark(0.75, 0.8));
        mutate(&mut observations, &mut punishes[0]);
        refine(&mut punishes, &observations);
        punishes[0].reachability
    };

    // 境界より遠い先端ガード((0.78125-0.5)/0.5 = 0.5625)。
    assert_eq!(
        run(&|o, _| o[1].contact = Some(guard_spark(0.78125, 0.8))),
        PunishReachability::Unknown
    );
    // スパークの確度が足りない(0.5 が境界)。
    assert_eq!(
        run(&|o, _| o[1].contact = Some(guard_spark(0.75, 0.49))),
        PunishReachability::Unknown
    );
    assert_eq!(
        run(&|o, _| o[1].contact = Some(guard_spark(0.75, 0.5))),
        PunishReachability::Confirmed,
        "確度の境界ちょうどは使う"
    );
    // 攻撃側の箱が最低身長(1/8)を割る。ちょうどなら物差しになる。
    assert_eq!(
        run(&|o, _| {
            o[1].p2 = Some(exact_attacker(0.775));
            o[1].contact = Some(guard_spark(0.5625, 0.8));
        }),
        PunishReachability::Unknown
    );
    assert_eq!(
        run(&|o, _| {
            o[1].p2 = Some(exact_attacker(0.75));
            o[1].contact = Some(guard_spark(0.5625, 0.8));
        }),
        PunishReachability::Confirmed,
        "身長 1/8 ちょうど、スパーク 0.0625 = 1/2 身長"
    );
    // 空中の攻撃側は立ち姿勢の物差しにならない。
    assert_eq!(
        run(&|o, _| {
            let mut airborne = exact_attacker(0.375);
            airborne.ground_anchor = false;
            o[1].p2 = Some(airborne);
            o[0].p2 = None;
        }),
        PunishReachability::Unknown
    );
    // ガード起点の接触フレームが無ければ、どのスパークの話か分からない。
    assert_eq!(
        run(&|_, p| p.source_contact_frame = None),
        PunishReachability::Unknown
    );
}

/// スパークは接触から hitstop 尾部(10F)までのものだけを使い、複数ある
/// ときは最も確かな観測に従う。攻撃側のサンプルは接触の 20F 前まで遡る。
#[test]
fn spark_and_attacker_sample_windows_are_bounded() {
    let run = |spark_frame: u32, attacker_frame: u32, spark_x: f32| {
        let mut punishes = vec![chance(PunishOutcome::Missed)];
        let mut observations = vec![
            observation(attacker_frame, DistanceBand::Close),
            observation(spark_frame, DistanceBand::Close),
        ];
        observations[0].p2 = Some(exact_attacker(0.375));
        observations[1].p1 = None;
        observations[1].p2 = None;
        observations[1].contact = Some(guard_spark(spark_x, 0.8));
        refine(&mut punishes, &observations);
        punishes[0].reachability
    };

    // 接触(90)から +10F までのスパークは使う。+11F は別の接触かもしれない。
    assert_eq!(run(100, 91, 0.75), PunishReachability::Confirmed);
    assert_eq!(run(101, 91, 0.75), PunishReachability::Unknown);
    // 接触より前のスパークはこのガードのものではない。
    assert_eq!(run(89, 91, 0.75), PunishReachability::Unknown);
    // 攻撃側のサンプルは接触の 20F 前(70)まで。それより古い立ち姿勢は
    // 歩き・しゃがみで既に別の姿勢になっている可能性が高い。
    assert_eq!(run(91, 70, 0.75), PunishReachability::Confirmed);
    assert_eq!(run(91, 69, 0.75), PunishReachability::Unknown);

    // 複数のスパークは最も確かな観測に従う。確かな方が先端なら断定しない。
    let mut punishes = vec![chance(PunishOutcome::Missed)];
    let mut observations = vec![
        observation(91, DistanceBand::Close),
        observation(93, DistanceBand::Close),
    ];
    observations[0].p2 = Some(exact_attacker(0.375));
    observations[0].contact = Some(guard_spark(0.78125, 0.9));
    observations[1].contact = Some(guard_spark(0.75, 0.6));
    refine(&mut punishes, &observations);
    assert_eq!(punishes[0].reachability, PunishReachability::Unknown);
}

/// めり込みの確認は、距離帯で答えが出なかった見逃しにだけ足す。
/// 技を出した空振りは既存の距離帯判断が担い、離れていたと確定した
/// 場面を上書きすることもない。
#[test]
fn deep_contact_only_upgrades_an_unknown_missed_punish() {
    let deep = |outcome, band| {
        let mut punishes = vec![chance(outcome)];
        let mut observations = vec![observation(89, band), observation(91, band)];
        observations[1].p2 = Some(exact_attacker(0.375));
        observations[1].contact = Some(guard_spark(0.75, 0.8));
        refine(&mut punishes, &observations);
        punishes[0].reachability
    };

    // 中〜遠距離が続いた見逃しは届かないままにする。スパークの座標より
    // 距離帯の継続観測の方が強い証拠になる。
    assert_eq!(
        deep(PunishOutcome::Missed, DistanceBand::Mid),
        PunishReachability::OutOfRange
    );
    // 空振りは技の到達距離の話で、めり込みの確認対象ではない。
    // Far 連続で離れていたと確定する場面を触らない。
    assert_eq!(
        deep(PunishOutcome::WhiffFail, DistanceBand::Far),
        PunishReachability::OutOfRange
    );
}

/// P2 側の見逃しでは攻撃側は P1。side を取り違えると、関係ない方の体を
/// 物差しにしてしまう。
#[test]
fn a_deep_spark_is_measured_against_the_p1_attacker_for_a_p2_punish() {
    let mut punishes = vec![PunishChance {
        side: 2,
        ..chance(PunishOutcome::Missed)
    }];
    let mut observations = vec![
        observation(89, DistanceBand::Close),
        observation(91, DistanceBand::Close),
    ];
    // 攻撃側 P1 を 2 進で正確な立ち姿勢に置き、その 1/2 身長以内に
    // スパークを置く。P2(anchor 0.6、身長 0.2)からは遠い。
    let attacker = ActorObservation {
        anchor: SpatialPoint::new(0.25, 0.875),
        bounds: SpatialRect::new(0.2, 0.375, 0.3, 0.875),
        confidence: 0.72,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    observations[1].p1 = Some(attacker);
    observations[1].contact = Some(guard_spark(0.5, 0.8));

    refine(&mut punishes, &observations);

    assert_eq!(punishes[0].reachability, PunishReachability::Confirmed);
}
