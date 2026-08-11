//! ジャンプの攻防に対するテスト。
//!
//! 二つの指摘は鏡像になっている。相手の飛びを通したのなら対空の話、
//! 自分の飛びを落とされたのなら接近手段の話。どちらも一度きりでは
//! 読み負けと区別が付かず、決着した飛びの半分以上で負けて初めて
//! 傾向として扱う。
//!
//! 後退ジャンプと、離陸を確認できていない候補は、どちらの話にも
//! 入れない。入れると分母が水増しされて割合が狂う。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{DamageEvent, JumpDirection, JumpEvent, JumpOutcome, MatchEvents};
use crate::AdviceKind;

fn jump(side: u8, frame: u32, outcome: JumpOutcome) -> JumpEvent {
    JumpEvent {
        side,
        frame,
        outcome,
        input_dir: "UR".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(frame + 20),
        takeoff_confirmed: true,
        air_end: frame + 45,
        round_no: 1,
    }
}

/// 自分が受けた被弾。
fn damage(frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame: frame,
        end_frame: frame + 30,
        pre_freeze_frame: frame,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

fn events_with(jumps: Vec<JumpEvent>, damages: Vec<DamageEvent>) -> MatchEvents {
    MatchEvents {
        jumps,
        damage: damages,
        ..empty_events()
    }
}

// ── 相手の飛びへの対応 ───────────────────────────────────────────────────

/// 相手が飛んでいなければ話にならない。
#[test]
fn nothing_is_reported_without_the_opponent_jumping() {
    assert!(detect_anti_air(&empty_events(), 1, 2).is_none());
}

/// 全部迎撃できていれば指摘しない。
#[test]
fn jumps_that_were_all_stopped_are_not_reported() {
    let events = events_with(
        vec![
            jump(2, 100, JumpOutcome::GotHit),
            jump(2, 600, JumpOutcome::GotHit),
        ],
        vec![],
    );

    assert!(detect_anti_air(&events, 1, 2).is_none());
}

/// 一度通されただけでは、地上へ意識を割いた結果かもしれない。
#[test]
fn a_single_jump_getting_through_stays_an_observation() {
    let events = events_with(
        vec![
            jump(2, 100, JumpOutcome::LandedHit),
            jump(2, 600, JumpOutcome::GotHit),
            jump(2, 1200, JumpOutcome::GotHit),
        ],
        vec![damage(120, 0.15)],
    );

    let card = detect_anti_air(&events, 1, 2).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation, "一度で診断にしている");
    assert_eq!(card.hp_lost, Some(0.15));
    assert!((card.severity - 0.17).abs() < 1e-6);
}

/// 決着した飛びの半分以上を通されて、しかも複数回なら対空の課題。
#[test]
fn letting_most_jumps_through_becomes_a_diagnosis() {
    let events = events_with(
        vec![
            jump(2, 100, JumpOutcome::LandedHit),
            jump(2, 600, JumpOutcome::LandedHit),
            jump(2, 1200, JumpOutcome::GotHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.12)],
    );

    let card = detect_anti_air(&events, 1, 2).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis, "傾向を拾えていない");
    assert!((card.hp_lost.expect("損失がある") - 0.27).abs() < 1e-6);
    assert!((card.severity - 0.31).abs() < 1e-6);
}

/// 決着した飛びがちょうど半々でも、同じ失敗が複数回なら診断にする。
/// 成功数との足し算と 50% の境界を同時に固定する。
#[test]
fn exactly_half_of_resolved_jumps_getting_through_is_inclusive() {
    let mut jumps = Vec::new();
    for (index, outcome) in [
        JumpOutcome::LandedHit,
        JumpOutcome::LandedHit,
        JumpOutcome::LandedHit,
        JumpOutcome::GotHit,
        JumpOutcome::GotHit,
        JumpOutcome::GotHit,
    ]
    .into_iter()
    .enumerate()
    {
        jumps.push(jump(2, 100 + index as u32 * 300, outcome));
    }

    let card = detect_anti_air(&events_with(jumps, vec![]), 1, 2).expect("提示される");

    assert_eq!(card.kind, AdviceKind::Diagnosis);
}

/// 通した回数が同じでも、迎撃できている回数の方が多ければ課題ではない。
/// 割合を見ないと、飛びの多い相手ほど対空が下手に見える。
#[test]
fn stopping_more_than_you_let_through_is_not_a_problem() {
    let events = events_with(
        vec![
            jump(2, 100, JumpOutcome::LandedHit),
            jump(2, 600, JumpOutcome::LandedHit),
            jump(2, 1200, JumpOutcome::GotHit),
            jump(2, 1800, JumpOutcome::GotHit),
            jump(2, 2400, JumpOutcome::GotHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.12)],
    );

    let card = detect_anti_air(&events, 1, 2).expect("提示される");

    assert_eq!(
        card.kind,
        AdviceKind::Observation,
        "迎撃できている分を見ていない"
    );
}

/// 決着しなかった飛びは割合の分母に入れない。触れずに終わった飛びを
/// 「対空できなかった」に数えると、割合が実際より悪く出る。
#[test]
fn jumps_that_resolved_into_nothing_do_not_count_against_the_share() {
    let events = events_with(
        vec![
            jump(2, 100, JumpOutcome::LandedHit),
            jump(2, 600, JumpOutcome::LandedHit),
            jump(2, 1200, JumpOutcome::GotHit),
            jump(2, 1800, JumpOutcome::UnverifiedHit),
            jump(2, 2400, JumpOutcome::UnverifiedHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.12)],
    );

    let card = detect_anti_air(&events, 1, 2).expect("提示される");

    assert_eq!(
        card.kind,
        AdviceKind::Diagnosis,
        "決着しない飛びで割合が薄まっている"
    );
    assert!(
        card.description.contains("残り 2 回"),
        "決着しなかった飛びを数えていない: {}",
        card.description
    );
}

/// 後退ジャンプは飛び込みではない。逃げの飛びを対空の話に混ぜると、
/// 触りようのない飛びまで課題に数える。
#[test]
fn a_backward_jump_is_not_an_approach() {
    let mut events = events_with(
        vec![
            jump(2, 100, JumpOutcome::LandedHit),
            jump(2, 600, JumpOutcome::LandedHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.12)],
    );
    events.jumps[1].direction = JumpDirection::Backward;

    let card = detect_anti_air(&events, 1, 2).expect("提示される");

    assert_eq!(card.evidence.len(), 1, "逃げの飛びを数えている");
}

/// 離陸を確認できていない候補は使わない。飛んだかどうか曖昧なまま
/// 「対空できていない」とは言えない。
#[test]
fn an_unconfirmed_takeoff_is_not_used() {
    let mut events = events_with(
        vec![jump(2, 100, JumpOutcome::LandedHit)],
        vec![damage(120, 0.15)],
    );
    events.jumps[0].takeoff_confirmed = false;

    assert!(detect_anti_air(&events, 1, 2).is_none());
}

/// 自分の飛びは対空の話ではない。
#[test]
fn your_own_jumps_are_not_the_anti_air_story() {
    let events = events_with(
        vec![jump(1, 100, JumpOutcome::LandedHit)],
        vec![damage(120, 0.15)],
    );

    assert!(detect_anti_air(&events, 1, 2).is_none());
}

/// 被弾は飛びの接触時刻に結び付ける。離れた被弾まで拾うと、地上戦の
/// 損失が対空のせいになる。
#[test]
fn only_damage_near_the_contact_is_attributed_to_the_jump() {
    let near = events_with(
        vec![jump(2, 100, JumpOutcome::LandedHit)],
        vec![damage(145, 0.15)],
    );
    let far = events_with(
        vec![jump(2, 100, JumpOutcome::LandedHit)],
        vec![damage(146, 0.15)],
    );

    let near = detect_anti_air(&near, 1, 2).expect("提示される");
    let far = detect_anti_air(&far, 1, 2).expect("提示される");

    assert_eq!(near.hp_lost, Some(0.15), "接触時刻の被弾を落としている");
    assert_eq!(far.hp_lost, Some(0.0), "離れた被弾を拾っている");
}

/// 接触時刻が取れていない飛びは、空中攻撃が当たりうる範囲で探す。
#[test]
fn a_jump_without_a_contact_frame_falls_back_to_the_attack_window() {
    let mut events = events_with(
        vec![jump(2, 100, JumpOutcome::LandedHit)],
        vec![damage(120, 0.15)],
    );
    events.jumps[0].contact_frame = None;

    let card = detect_anti_air(&events, 1, 2).expect("提示される");

    assert_eq!(card.hp_lost, Some(0.15), "接触時刻が無いと被弾を落とす");
}

/// 相手が受けた被弾は自分の損失ではない。
#[test]
fn damage_the_opponent_took_is_not_your_loss() {
    let mut events = events_with(
        vec![jump(2, 100, JumpOutcome::LandedHit)],
        vec![damage(120, 0.15)],
    );
    events.damage[0].victim = 2;

    let card = detect_anti_air(&events, 1, 2).expect("提示される");

    assert_eq!(card.hp_lost, Some(0.0), "相手の被弾を自分に付けている");
}

// ── 自分の飛びの結果 ─────────────────────────────────────────────────────

/// 落とされていなければ話にならない。
#[test]
fn jumps_that_were_never_stopped_are_not_reported() {
    let events = events_with(vec![jump(1, 100, JumpOutcome::LandedHit)], vec![]);

    assert!(detect_own_jumps(&events, 1).is_none());
}

/// 一度落とされただけなら、相手の対空を試した一回かもしれない。
#[test]
fn a_single_stopped_jump_stays_an_observation() {
    let events = events_with(
        vec![
            jump(1, 100, JumpOutcome::GotHit),
            jump(1, 600, JumpOutcome::LandedHit),
            jump(1, 1200, JumpOutcome::LandedHit),
        ],
        vec![damage(120, 0.15)],
    );

    let card = detect_own_jumps(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.hp_lost, Some(0.15));
    assert!((card.severity - 0.17).abs() < 1e-6);
}

/// 決着した飛びの半分以上を落とされて、しかも複数回なら接近手段の課題。
#[test]
fn being_stopped_most_of_the_time_becomes_a_diagnosis() {
    let events = events_with(
        vec![
            jump(1, 100, JumpOutcome::GotHit),
            jump(1, 600, JumpOutcome::GotHit),
            jump(1, 1200, JumpOutcome::LandedHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.12)],
    );

    let card = detect_own_jumps(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert!((card.hp_lost.expect("損失がある") - 0.27).abs() < 1e-6);
    assert!((card.severity - 0.31).abs() < 1e-6);
}

/// 自分の飛びも、成功と被対空がちょうど半々なら境界を含める。
#[test]
fn exactly_half_of_resolved_own_jumps_getting_stopped_is_inclusive() {
    let mut jumps = Vec::new();
    for (index, outcome) in [
        JumpOutcome::GotHit,
        JumpOutcome::GotHit,
        JumpOutcome::GotHit,
        JumpOutcome::LandedHit,
        JumpOutcome::LandedHit,
        JumpOutcome::LandedHit,
    ]
    .into_iter()
    .enumerate()
    {
        jumps.push(jump(1, 100 + index as u32 * 300, outcome));
    }

    let card = detect_own_jumps(&events_with(jumps, vec![]), 1).expect("提示される");

    assert_eq!(card.kind, AdviceKind::Diagnosis);
}

/// 通っている飛びの方が多ければ課題ではない。
#[test]
fn landing_more_jumps_than_you_lose_is_not_a_problem() {
    let events = events_with(
        vec![
            jump(1, 100, JumpOutcome::GotHit),
            jump(1, 600, JumpOutcome::GotHit),
            jump(1, 1200, JumpOutcome::LandedHit),
            jump(1, 1800, JumpOutcome::LandedHit),
            jump(1, 2400, JumpOutcome::LandedHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.12)],
    );

    let card = detect_own_jumps(&events, 1).expect("提示される");

    assert_eq!(card.kind, AdviceKind::Observation);
}

/// 空中に居たと確認できていない被弾は、落とされた飛びに数えない。
/// 地上で喰らった分をジャンプのせいにすることになる。
#[test]
fn a_hit_taken_on_the_ground_is_not_a_stopped_jump() {
    let mut events = events_with(
        vec![jump(1, 100, JumpOutcome::GotHit)],
        vec![damage(120, 0.15)],
    );
    events.jumps[0].outcome = JumpOutcome::GroundedHit;

    assert!(detect_own_jumps(&events, 1).is_none());
}

/// 相手の飛びは自分の接近手段の話ではない。
#[test]
fn the_opponents_jumps_are_not_your_approach() {
    let events = events_with(
        vec![jump(2, 100, JumpOutcome::GotHit)],
        vec![damage(120, 0.15)],
    );

    assert!(detect_own_jumps(&events, 1).is_none());
}

/// 落とされた回数も重みに効く。同じ被ダメでも、繰り返している方が
/// 見直す価値が高い。
#[test]
fn being_stopped_more_often_weighs_more() {
    let once = events_with(
        vec![jump(1, 100, JumpOutcome::GotHit)],
        vec![damage(120, 0.30)],
    );
    let twice = events_with(
        vec![
            jump(1, 100, JumpOutcome::GotHit),
            jump(1, 600, JumpOutcome::GotHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.15)],
    );

    let once = detect_own_jumps(&once, 1).expect("提示される");
    let twice = detect_own_jumps(&twice, 1).expect("提示される");

    assert_eq!(once.hp_lost, twice.hp_lost, "損失は同じはず");
    assert!(twice.severity > once.severity, "回数が重みに効いていない");
}

/// 二つの指摘は同じ飛びを取り合わない。自分の飛びと相手の飛びは別の話。
#[test]
fn the_two_cards_do_not_claim_the_same_jumps() {
    let events = events_with(
        vec![
            jump(1, 100, JumpOutcome::GotHit),
            jump(2, 600, JumpOutcome::LandedHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.12)],
    );

    let mine = detect_own_jumps(&events, 1).expect("自分の飛び");
    let theirs = detect_anti_air(&events, 1, 2).expect("相手の飛び");

    assert_eq!(mine.evidence.len(), 1);
    assert_eq!(theirs.evidence.len(), 1);
    assert_ne!(mine.id, theirs.id);
    assert_eq!(mine.hp_lost, Some(0.15));
    assert_eq!(theirs.hp_lost, Some(0.12));
}

/// どちらの指摘も、一度きりと傾向で文面を書き分ける。
#[test]
fn both_cards_change_their_wording_when_it_becomes_a_habit() {
    let once = events_with(
        vec![
            jump(1, 100, JumpOutcome::GotHit),
            jump(1, 600, JumpOutcome::LandedHit),
            jump(1, 1200, JumpOutcome::LandedHit),
        ],
        vec![damage(120, 0.15)],
    );
    let habit = events_with(
        vec![
            jump(1, 100, JumpOutcome::GotHit),
            jump(1, 600, JumpOutcome::GotHit),
        ],
        vec![damage(120, 0.15), damage(620, 0.12)],
    );

    let once = detect_own_jumps(&once, 1).expect("提示される");
    let habit = detect_own_jumps(&habit, 1).expect("提示される");

    assert_eq!(once.id, habit.id);
    assert_ne!(once.title, habit.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, habit.description,
        "説明を書き分けていない"
    );
    assert_ne!(once.practice, habit.practice, "練習方法を書き分けていない");
}
