//! ガード入力が崩れて被弾した場面を切り出す条件に対するテスト。
//!
//! 「ガードしていたのに入力が外れて喰らった」と言うには三つの一致が要る。
//! 実際にガード方向を握って硬直していたこと、その入力が上か前へ外れた
//! こと、そして外れた状態で被弾したこと。
//!
//! どれかを緩めると、コンボの二発目や、暴れて狩られた被弾が「入力の癖」
//! として並ぶ。直しようのない指摘になる。

use crate::test_support::*;

const LENGTH: usize = 200;

/// ガード方向を握って硬直し、被弾の直前に上へ外した観測列。
///
/// f70..100 ガード硬直（↘ を保持）、f97 以降 ↗ へ外れ、f100 被弾。
fn broke_guard_upward() -> (
    Vec<MeterState>,
    Vec<f32>,
    Vec<TrackedInput>,
    Vec<DamageEvent>,
) {
    let mut meter = vec![MeterState::Free; LENGTH];
    meter[70..100].fill(MeterState::Stun);
    let mut hp = vec![1.0; LENGTH];
    hp[100..].fill(0.85);
    let inputs: Vec<_> = (0..LENGTH)
        .map(|frame| {
            let direction = if frame < 97 {
                InputDir::DownRight
            } else {
                InputDir::UpRight
            };
            tracked(frame as u32 + 1, direction, vec![], false, false)
        })
        .collect();
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 100,
        pre_freeze_frame: 100,
        end_frame: 115,
        hp_before: 1.0,
        hp_after: 0.85,
        drop: 0.15,
        round_no: 1,
    }];
    (meter, hp, inputs, damage)
}

#[allow(clippy::too_many_arguments)]
fn extract(
    meter: Vec<MeterState>,
    hp: Vec<f32>,
    inputs: Vec<TrackedInput>,
    damage: Vec<DamageEvent>,
    jumps: Vec<JumpEvent>,
    throws: Vec<ThrowEvent>,
    reversals: Vec<ReversalEvent>,
) -> Vec<GuardBreakEvent> {
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: LENGTH as u32 - 1,
        winner: None,
        p1_hp_end: 0.85,
        p2_hp_end: 1.0,
    }];

    crate::guard_breaks::extract_guard_breaks(
        &damage,
        &[meter, vec![MeterState::Free; LENGTH]],
        &[hp, vec![1.0; LENGTH]],
        [&inputs, &[]],
        &jumps,
        &throws,
        &reversals,
        &rounds,
    )
}

// ── 三点の一致 ───────────────────────────────────────────────────────────

/// 握っていたガード方向と、外れた先の両方を記録する。
#[test]
fn a_guard_released_upward_is_recorded_with_both_directions() {
    let (meter, hp, inputs, damage) = broke_guard_upward();

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert_eq!(breaks.len(), 1);
    assert_eq!(breaks[0].guard_dir, "DR");
    assert_eq!(breaks[0].broke_to, "UR");
    assert!((breaks[0].drop - 0.15).abs() < 1e-6);
}

/// 前上へ外れた場合も崩れ。相手の向きから見て前がどちらかは、握って
/// いた方向から決まる。
#[test]
fn jumping_forward_out_of_guard_is_also_a_break() {
    let (meter, hp, mut inputs, damage) = broke_guard_upward();
    for input in inputs.iter_mut().skip(97) {
        *input = tracked(1, InputDir::UpLeft, vec![], false, false);
    }

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert_eq!(breaks.len(), 1);
    assert_eq!(breaks[0].broke_to, "UL");
}

/// 真横へ歩いた場合は判定しない。握っていた方向と反対の横は、
/// 向き直った後のガード方向でもある。入力だけでは区別できないので、
/// 崩れとは断定しない。
#[test]
fn walking_straight_into_the_other_guard_direction_is_ambiguous() {
    let (meter, hp, mut inputs, damage) = broke_guard_upward();
    for input in inputs.iter_mut().skip(97) {
        *input = tracked(1, InputDir::Left, vec![], false, false);
    }

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty(), "曖昧な入力から崩れを断定している");
}

/// 後ろへ握り直したのは崩れではない。ガードは継続している。
#[test]
fn holding_guard_all_the_way_is_not_a_break() {
    let (meter, hp, mut inputs, damage) = broke_guard_upward();
    for input in inputs.iter_mut().skip(97) {
        *input = tracked(1, InputDir::Right, vec![], false, false);
    }

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty(), "ガードを握り続けたのを崩れにしている");
}

/// ガード方向を握っていた時間が短ければ、そもそも守っていたとは
/// 言えない。
#[test]
fn a_guard_held_too_briefly_does_not_count_as_defending() {
    let (mut meter, hp, inputs, damage) = broke_guard_upward();
    meter[70..100].fill(MeterState::Free);
    meter[93..97].fill(MeterState::Stun);
    let long_enough = extract(
        meter.clone(),
        hp.clone(),
        inputs.clone(),
        damage.clone(),
        vec![],
        vec![],
        vec![],
    );
    assert_eq!(long_enough.len(), 1, "ちょうどの長さを落としている");

    meter[93] = MeterState::Free;
    let too_short = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);
    assert!(too_short.is_empty(), "短すぎる硬直を守りにしている");
}

/// 硬直していなければガードではない。同じ方向を握っていても、
/// 何も来ていなければただ立っているだけ。
#[test]
fn holding_back_without_blockstun_is_not_defending() {
    let (mut meter, hp, inputs, damage) = broke_guard_upward();
    meter[70..100].fill(MeterState::Free);

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty());
}

/// 入力欄が読めていない場面では判定しない。補修した推測値で「外した」
/// とは言えない。
#[test]
fn a_repaired_input_cannot_show_the_guard_being_released() {
    let (meter, hp, mut inputs, damage) = broke_guard_upward();
    for input in inputs.iter_mut().skip(97) {
        let mut repaired = tracked(1, InputDir::UpRight, vec![], false, false);
        repaired.repaired = true;
        *input = repaired;
    }

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty(), "補修値で崩れを判定している");
}

/// 読み取りが怪しい入力も同じ。
#[test]
fn an_uncertain_input_cannot_show_it_either() {
    let (meter, hp, mut inputs, damage) = broke_guard_upward();
    for input in inputs.iter_mut().skip(97) {
        let mut uncertain = tracked(1, InputDir::UpRight, vec![], false, false);
        uncertain.uncertain = true;
        *input = uncertain;
    }

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty());
}

/// 外れたのが被弾から離れていれば、その被弾の原因ではない。
#[test]
fn a_release_long_before_the_hit_is_not_its_cause() {
    let (meter, hp, mut inputs, damage) = broke_guard_upward();
    // f90 で外し、f93 からガードへ戻る。
    for input in inputs.iter_mut().take(93).skip(90) {
        *input = tracked(1, InputDir::UpRight, vec![], false, false);
    }
    for input in inputs.iter_mut().skip(93) {
        *input = tracked(1, InputDir::DownRight, vec![], false, false);
    }

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty(), "離れた入力を原因にしている");
}

// ── コンボの途中を崩れにしない ───────────────────────────────────────────

/// 被弾直前に HP が減っていれば、それはコンボの継続。ガードしていた
/// わけではない。
#[test]
fn a_hit_inside_an_ongoing_combo_is_not_a_guard_break() {
    let (meter, mut hp, inputs, damage) = broke_guard_upward();
    hp[80..].fill(0.95);
    hp[100..].fill(0.85);

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty(), "コンボの継続を崩れにしている");
}

/// 削りの無い場面は指摘しない。ごく小さい被弾まで拾うと、指摘が
/// 埋もれる。
#[test]
fn a_hit_too_small_to_matter_is_not_reported() {
    let (meter, mut hp, inputs, mut damage) = broke_guard_upward();
    hp[100..].fill(0.99);
    damage[0].drop = 0.019;
    damage[0].hp_after = 0.981;

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty());
}

/// 自分が技を振っていれば、ガードが崩れたのではなく暴れて狩られている。
#[test]
fn swinging_a_move_means_you_were_not_guarding() {
    let (mut meter, hp, inputs, damage) = broke_guard_upward();
    meter[98] = MeterState::Startup;

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty(), "暴れて狩られた場面を崩れにしている");
}

/// 移動やジャンプの動作は崩れそのものなので、技を振ったことにしない。
#[test]
fn moving_is_the_break_itself_not_an_attack() {
    let (mut meter, hp, inputs, damage) = broke_guard_upward();
    meter[98] = MeterState::MotionRecovery;

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert_eq!(breaks.len(), 1, "崩れの動作を攻撃と読んでいる");
}

// ── 他のイベントが扱う被弾 ───────────────────────────────────────────────

/// 空中で対空された飛びは、飛びの指摘が扱う。
#[test]
fn a_jump_that_was_anti_aired_belongs_to_the_jump_events() {
    let (meter, hp, inputs, damage) = broke_guard_upward();
    let jump = JumpEvent {
        side: 1,
        frame: 97,
        outcome: JumpOutcome::GotHit,
        input_dir: "UR".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(100),
        takeoff_confirmed: true,
        air_end: 140,
        round_no: 1,
    };

    let breaks = extract(meter, hp, inputs, damage, vec![jump], vec![], vec![]);

    assert!(breaks.is_empty());
}

/// 離陸を確認できていない飛びは、崩れの側に残す。地上の予備動作を
/// 狩られたのなら、それは入力が外れた話。
#[test]
fn an_unconfirmed_jump_leaves_the_break_in_place() {
    let (meter, hp, inputs, damage) = broke_guard_upward();
    let jump = JumpEvent {
        side: 1,
        frame: 97,
        outcome: JumpOutcome::GotHit,
        input_dir: "UR".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(100),
        takeoff_confirmed: false,
        air_end: 140,
        round_no: 1,
    };

    let breaks = extract(meter, hp, inputs, damage, vec![jump], vec![], vec![]);

    assert_eq!(breaks.len(), 1);
}

/// 投げられたのは入力の崩れではない。ガードしていても投げられる。
#[test]
fn being_thrown_is_not_a_guard_break() {
    let (meter, hp, inputs, damage) = broke_guard_upward();
    let throw = ThrowEvent {
        thrower: 2,
        frame: 98,
        connected: true,
        round_no: 1,
    };

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![throw], vec![]);

    assert!(breaks.is_empty());
}

/// 抜けられた投げは被弾していないので、崩れの判断に影響しない。
#[test]
fn a_throw_that_did_not_connect_changes_nothing() {
    let (meter, hp, inputs, damage) = broke_guard_upward();
    let throw = ThrowEvent {
        thrower: 2,
        frame: 98,
        connected: false,
        round_no: 1,
    };

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![throw], vec![]);

    assert_eq!(breaks.len(), 1);
}

/// 狩られた無敵技は、切り返しの指摘が扱う。
#[test]
fn a_punished_reversal_belongs_to_the_reversal_events() {
    let (meter, hp, inputs, damage) = broke_guard_upward();
    let reversal = ReversalEvent {
        side: 1,
        frame: 96,
        drop: 0.15,
        blocked: false,
        confidence: EventConfidence::High,
        round_no: 1,
    };

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![reversal]);

    assert!(breaks.is_empty());
}

/// 相手の無敵技は自分の崩れを打ち消さない。
#[test]
fn the_opponents_reversal_does_not_excuse_your_break() {
    let (meter, hp, inputs, damage) = broke_guard_upward();
    let reversal = ReversalEvent {
        side: 2,
        frame: 96,
        drop: 0.15,
        blocked: false,
        confidence: EventConfidence::High,
        round_no: 1,
    };

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![reversal]);

    assert_eq!(breaks.len(), 1);
}

// ── 画面の向き ───────────────────────────────────────────────────────────

/// 左を向いているときのガードは左方向。握っていた方向から向きを
/// 決めるので、画面が入れ替わっても判定は変わらない。
#[test]
fn the_guard_side_comes_from_the_direction_that_was_held() {
    let (meter, hp, mut inputs, damage) = broke_guard_upward();
    for input in inputs.iter_mut().take(97) {
        *input = tracked(1, InputDir::DownLeft, vec![], false, false);
    }
    for input in inputs.iter_mut().skip(97) {
        *input = tracked(1, InputDir::UpRight, vec![], false, false);
    }

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert_eq!(breaks.len(), 1);
    assert_eq!(breaks[0].guard_dir, "DL");
    assert_eq!(breaks[0].broke_to, "UR");
}

/// ガード方向でない入力を握っていた場面は、そもそも守っていない。
#[test]
fn holding_a_non_guard_direction_is_not_defending() {
    let (meter, hp, mut inputs, damage) = broke_guard_upward();
    for input in inputs.iter_mut().take(97) {
        *input = tracked(1, InputDir::Neutral, vec![], false, false);
    }

    let breaks = extract(meter, hp, inputs, damage, vec![], vec![], vec![]);

    assert!(breaks.is_empty());
}
