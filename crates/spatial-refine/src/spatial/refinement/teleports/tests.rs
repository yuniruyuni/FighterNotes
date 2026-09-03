//! テレポートを迎撃できたかの判断に対するテスト。
//!
//! 「昇竜が届いた」と言うには、そのキャラクターが昇竜を持っていて、
//! その瞬間に撃てる状態で、位置も届く範囲だった、の三つが要る。
//!
//! 溜め系の昇竜は、直前に下方向を溜め続けていなければ撃てない。溜めを
//! 確かめずに「迎撃できたはず」と言うと、撃ちようのない場面を課題に
//! することになる。

use super::*;
use crate::match_events::ThreatOutcome;
use crate::spatial::{ActorObservation, SpatialPoint, SpatialRect};

/// 昇竜をコマンドで撃てるキャラクター。
const MOTION_REVERSAL: &str = "CAMMY";
/// 昇竜に溜めが要るキャラクター。
const CHARGE_REVERSAL: &str = "BLANKA";
/// 昇るリバーサルを持たないキャラクター。
const NO_REVERSAL: &str = "ZANGIEF";

/// 相手が仕掛けたテレポート攻撃。f100 入力、f130 接触。
fn teleport() -> TeleportEvent {
    TeleportEvent {
        attacker: 2,
        defender: 1,
        input_frame: 100,
        inv_start_frame: 102,
        inv_end_frame: 120,
        followup_attack_frame: Some(126),
        followup_contact_frame: Some(130),
        airborne: true,
        defender_actionable: true,
        context: TeleportContext::NakedAttack,
        response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.20,
        dp_reachability: DpReachability::Unknown,
        round_no: 1,
        confidence: 1.0,
    }
}

/// 距離帯の分かる 1 フレーム分の観測。
fn observation(frame_index: u32, band: DistanceBand, confidence: f32) -> SpatialObservation {
    let actor = |x: f32| ActorObservation {
        anchor: SpatialPoint::new(x, 0.9),
        bounds: SpatialRect::new(x - 0.03, 0.5, x + 0.03, 0.92),
        confidence,
        observed: true,
        ground_anchor: true,
        discontinuity: false,
    };
    SpatialObservation {
        frame_index,
        p1: Some(actor(0.50)),
        p2: Some(actor(0.56)),
        screen_distance: Some(0.06),
        distance_band: Some(band),
        horizontal_order: None,
        projectile_candidates: Vec::new(),
        motion_regions: Vec::new(),
        contact: None,
        camera: None,
    }
}

/// 下方向を握り続けた入力。
fn holding_down(start: u32, end: u32) -> InputSegment {
    InputSegment {
        start_frame: start,
        end_frame: end,
        dir: "D".to_string(),
        badges: Vec::new(),
        auto: false,
        throw: false,
        evidence: Default::default(),
    }
}

/// 観測列を判断へ通す。
fn refine_with(
    character: &str,
    segments: Vec<InputSegment>,
    observations: Vec<SpatialObservation>,
) -> DpReachability {
    let mut teleports = vec![teleport()];
    let length = 400usize;
    let game_frames = [
        (0..length as i64).collect::<Vec<_>>(),
        (0..length as i64).collect::<Vec<_>>(),
    ];
    let context = AnalysisContext::from_characters("p1", Some(character), Some("DHALSIM"));

    refine(
        &mut teleports,
        &[segments, Vec::new()],
        &game_frames,
        &observations,
        &context,
    );
    teleports[0].dp_reachability
}

// ── キャラクターの持ち技 ─────────────────────────────────────────────────

/// コマンドで昇竜を撃てるなら、溜めは要らない。
#[test]
fn a_motion_reversal_needs_no_charge() {
    let reach = refine_with(
        MOTION_REVERSAL,
        vec![],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(reach, DpReachability::Confirmed);
}

/// 昇るリバーサルを持たないキャラクターでは何も言わない。振れない技を
/// 「振るべきだった」とは言えない。
#[test]
fn a_character_without_a_rising_reversal_stays_unknown() {
    let reach = refine_with(
        NO_REVERSAL,
        vec![],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(reach, DpReachability::Unknown);
}

// ── 溜めが要る昇竜 ───────────────────────────────────────────────────────

/// 溜めが足りていれば、他と同じように判断する。
#[test]
fn a_charged_reversal_with_enough_charge_is_judged() {
    let reach = refine_with(
        CHARGE_REVERSAL,
        vec![holding_down(50, 100)],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(reach, DpReachability::Confirmed);
}

/// 溜めが足りなければ撃てない。位置が届いていても課題にしない。
#[test]
fn a_charged_reversal_without_enough_charge_stays_unknown() {
    let reach = refine_with(
        CHARGE_REVERSAL,
        vec![holding_down(57, 100)],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(
        reach,
        DpReachability::Unknown,
        "溜めを確かめずに断定している"
    );
}

#[test]
fn exactly_forty_five_advancing_frames_complete_the_charge() {
    let segments = [vec![holding_down(56, 100)], vec![]];
    let game_frames = [
        (0..200).map(i64::from).collect::<Vec<_>>(),
        (0..200).map(i64::from).collect::<Vec<_>>(),
    ];

    assert!(rising_reversal_available(
        &segments,
        &game_frames,
        1,
        100,
        RisingReversalKind::Charge,
    ));
}

#[test]
fn charge_inputs_and_game_frames_come_from_the_defending_side() {
    let segments = [vec![], vec![holding_down(56, 100)]];
    let game_frames = [vec![-1; 200], (0..200).map(i64::from).collect::<Vec<_>>()];

    assert!(rising_reversal_available(
        &segments,
        &game_frames,
        2,
        100,
        RisingReversalKind::Charge,
    ));
    assert!(!rising_reversal_available(
        &segments,
        &game_frames,
        0,
        100,
        RisingReversalKind::Charge,
    ));
}

/// 溜めの記録が無ければ撃てない。
#[test]
fn a_charged_reversal_without_any_down_input_stays_unknown() {
    let reach = refine_with(
        CHARGE_REVERSAL,
        vec![],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(reach, DpReachability::Unknown);
}

/// 下方向以外を握っていた時間は溜めにならない。
#[test]
fn holding_a_direction_other_than_down_is_not_a_charge() {
    let mut sideways = holding_down(50, 100);
    sideways.dir = "R".to_string();

    let reach = refine_with(
        CHARGE_REVERSAL,
        vec![sideways],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(reach, DpReachability::Unknown);
}

/// 斜め下も溜めになる。しゃがみガードから撃てる。
#[test]
fn holding_a_diagonal_down_also_charges() {
    for direction in ["DL", "DR"] {
        let mut diagonal = holding_down(50, 100);
        diagonal.dir = direction.to_string();

        let reach = refine_with(
            CHARGE_REVERSAL,
            vec![diagonal],
            vec![observation(130, DistanceBand::Overlap, 0.72)],
        );

        assert_eq!(
            reach,
            DpReachability::Confirmed,
            "{direction} を溜めにしていない"
        );
    }
}

/// 続いた下入力は繋いで数える。入力欄は方向が変わるたびに区切られる
/// ので、繋がないと溜めがいくらあっても足りない。
#[test]
fn consecutive_down_inputs_are_joined_into_one_charge() {
    let reach = refine_with(
        CHARGE_REVERSAL,
        vec![
            holding_down(50, 70),
            holding_down(71, 85),
            holding_down(86, 100),
        ],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(reach, DpReachability::Confirmed, "続いた溜めを繋いでいない");
}

#[test]
fn one_unrecorded_frame_between_down_segments_is_still_joined() {
    let segments = [vec![holding_down(20, 58), holding_down(60, 100)], vec![]];
    let game_frames = [
        (0..200).map(i64::from).collect::<Vec<_>>(),
        (0..200).map(i64::from).collect::<Vec<_>>(),
    ];

    assert!(rising_reversal_available(
        &segments,
        &game_frames,
        1,
        100,
        RisingReversalKind::Charge,
    ));
}

#[test]
fn a_non_down_segment_stops_searching_before_an_older_charge() {
    let mut released = holding_down(59, 59);
    released.dir = "R".into();
    let segments = [
        vec![holding_down(20, 58), released, holding_down(60, 100)],
        vec![],
    ];
    let game_frames = [
        (0..200).map(i64::from).collect::<Vec<_>>(),
        (0..200).map(i64::from).collect::<Vec<_>>(),
    ];

    assert!(!rising_reversal_available(
        &segments,
        &game_frames,
        1,
        100,
        RisingReversalKind::Charge,
    ));
}

/// 途中で下を離していれば溜めは切れる。
#[test]
fn a_gap_in_the_down_input_breaks_the_charge() {
    let reach = refine_with(
        CHARGE_REVERSAL,
        vec![holding_down(20, 70), holding_down(80, 100)],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(reach, DpReachability::Unknown, "切れた溜めを繋いでいる");
}

/// 溜めを離してから少しの間は撃てる。入力表示の粒度で数フレームずれる。
#[test]
fn a_charge_released_a_moment_earlier_still_counts() {
    let released = refine_with(
        CHARGE_REVERSAL,
        vec![holding_down(40, 90)],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );
    let long_gone = refine_with(
        CHARGE_REVERSAL,
        vec![holding_down(30, 89)],
        vec![observation(130, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(
        released,
        DpReachability::Confirmed,
        "直前の溜めを捨てている"
    );
    assert_eq!(long_gone, DpReachability::Unknown, "古い溜めを使っている");
}

#[test]
fn advancing_game_frame_count_checks_bounds_negatives_and_inclusive_edges() {
    assert_eq!(advancing_game_frames(&[10], 0, 0), 1);
    assert_eq!(advancing_game_frames(&[10, 10, 11], 0, 2), 2);
    assert_eq!(advancing_game_frames(&[10, 10, -1], 0, 2), 0);
    assert_eq!(advancing_game_frames(&[10, 11], 0, 2), 0);
    assert_eq!(advancing_game_frames(&[10, 11], 1, 0), 0);
}

// ── 位置 ─────────────────────────────────────────────────────────────────

/// 体が重なっていれば届く。
#[test]
fn overlapping_bodies_confirm_the_reach() {
    assert_eq!(
        refine_with(
            MOTION_REVERSAL,
            vec![],
            vec![observation(130, DistanceBand::Overlap, 0.72)]
        ),
        DpReachability::Confirmed
    );
}

/// 遠ければ届かない。
#[test]
fn far_spacing_puts_the_reach_out_of_range() {
    assert_eq!(
        refine_with(
            MOTION_REVERSAL,
            vec![],
            vec![observation(130, DistanceBand::Far, 0.72)]
        ),
        DpReachability::OutOfRange
    );
}

/// 中間の距離では断定しない。昇竜の間合いは技ごとに違う。
#[test]
fn intermediate_spacing_stays_unknown() {
    for band in [DistanceBand::Close, DistanceBand::Mid] {
        assert_eq!(
            refine_with(MOTION_REVERSAL, vec![], vec![observation(130, band, 0.72)]),
            DpReachability::Unknown,
            "{band:?} を断定している"
        );
    }
}

/// 見るのは攻撃が当たった瞬間の位置。テレポートの入力時点では、まだ
/// 移動していない。
#[test]
fn the_position_is_read_at_the_moment_of_contact() {
    let reach = refine_with(
        MOTION_REVERSAL,
        vec![],
        vec![
            observation(100, DistanceBand::Far, 0.72),
            observation(130, DistanceBand::Overlap, 0.72),
        ],
    );

    assert_eq!(reach, DpReachability::Confirmed, "入力時点の位置を見ている");
}

/// 接触の時刻が分からなければ、攻撃の始まりで見る。
#[test]
fn without_a_contact_frame_the_attack_frame_is_used() {
    let mut teleports = vec![teleport()];
    teleports[0].followup_contact_frame = None;
    let length = 400usize;
    let game_frames = [
        (0..length as i64).collect::<Vec<_>>(),
        (0..length as i64).collect::<Vec<_>>(),
    ];
    let context = AnalysisContext::from_characters("p1", Some(MOTION_REVERSAL), Some("DHALSIM"));

    refine(
        &mut teleports,
        &[Vec::new(), Vec::new()],
        &game_frames,
        &[observation(126, DistanceBand::Overlap, 0.72)],
        &context,
    );

    assert_eq!(teleports[0].dp_reachability, DpReachability::Confirmed);
}

/// 離れた時刻の観測は使わない。
#[test]
fn an_observation_far_from_the_contact_is_not_used() {
    let inside = refine_with(
        MOTION_REVERSAL,
        vec![],
        vec![observation(134, DistanceBand::Overlap, 0.72)],
    );
    let outside = refine_with(
        MOTION_REVERSAL,
        vec![],
        vec![observation(135, DistanceBand::Overlap, 0.72)],
    );

    assert_eq!(inside, DpReachability::Confirmed, "近い観測を捨てている");
    assert_eq!(outside, DpReachability::Unknown, "離れた観測を使っている");
}

/// 観測が複数あれば、二人とも確からしく見えている方を選ぶ。
#[test]
fn the_most_confident_observation_wins() {
    let mut faint = observation(129, DistanceBand::Far, 0.46);
    faint.frame_index = 129;
    let clear = observation(130, DistanceBand::Overlap, 0.90);

    let mut teleports = vec![teleport()];
    let length = 400usize;
    let game_frames = [
        (0..length as i64).collect::<Vec<_>>(),
        (0..length as i64).collect::<Vec<_>>(),
    ];
    let context = AnalysisContext::from_characters("p1", Some(MOTION_REVERSAL), Some("DHALSIM"));

    refine(
        &mut teleports,
        &[Vec::new(), Vec::new()],
        &game_frames,
        &[faint, clear],
        &context,
    );

    assert_eq!(teleports[0].dp_reachability, DpReachability::Confirmed);
}

/// 片方しか見えていない観測は使わない。
#[test]
fn an_observation_missing_an_actor_is_not_used() {
    let mut half = observation(130, DistanceBand::Overlap, 0.72);
    half.p2 = None;

    let reach = refine_with(MOTION_REVERSAL, vec![], vec![half]);

    assert_eq!(reach, DpReachability::Unknown);
}

// ── 対象にしない場面 ─────────────────────────────────────────────────────

/// 飛び道具と挟まれたテレポートは対空の話ではない。
#[test]
fn a_projectile_covered_teleport_is_not_judged() {
    let mut teleports = vec![teleport()];
    teleports[0].context = TeleportContext::ProjectileCovered;
    let length = 400usize;
    let game_frames = [
        (0..length as i64).collect::<Vec<_>>(),
        (0..length as i64).collect::<Vec<_>>(),
    ];
    let context = AnalysisContext::from_characters("p1", Some(MOTION_REVERSAL), Some("DHALSIM"));

    refine(
        &mut teleports,
        &[Vec::new(), Vec::new()],
        &game_frames,
        &[observation(130, DistanceBand::Overlap, 0.72)],
        &context,
    );

    assert_eq!(teleports[0].dp_reachability, DpReachability::Unknown);
}

/// 攻撃を伴わないテレポートも対象にしない。
#[test]
fn a_teleport_without_a_follow_up_is_not_judged() {
    let mut teleports = vec![teleport()];
    teleports[0].followup_attack_frame = None;
    teleports[0].followup_contact_frame = None;
    let length = 400usize;
    let game_frames = [
        (0..length as i64).collect::<Vec<_>>(),
        (0..length as i64).collect::<Vec<_>>(),
    ];
    let context = AnalysisContext::from_characters("p1", Some(MOTION_REVERSAL), Some("DHALSIM"));

    refine(
        &mut teleports,
        &[Vec::new(), Vec::new()],
        &game_frames,
        &[observation(130, DistanceBand::Overlap, 0.72)],
        &context,
    );

    assert_eq!(teleports[0].dp_reachability, DpReachability::Unknown);
}
