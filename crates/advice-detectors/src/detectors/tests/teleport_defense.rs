//! テレポートからの攻撃を迎撃できなかった場面に対するテスト。
//!
//! テレポートは迎撃できる場合とできない場合がはっきり分かれる。飛び道具
//! と挟まれていれば対空を振れないし、硬直中なら振りようがないし、そもそも
//! 昇竜が届かない位置なら振っても当たらない。
//!
//! だからこの指摘は、迎撃できたと言い切れる条件が全部揃った場面だけを
//! 扱う。一つでも緩めると「どうしようもなかった場面」を課題として突き
//! つけることになる。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{
    DefenseResponse, DefenseResponseKind, DpReachability, MatchEvents, TeleportContext,
    TeleportEvent, ThreatOutcome,
};
use crate::AdviceKind;

/// 迎撃できたはずのテレポート攻撃を受けた場面。
fn missed_teleport(input_frame: u32) -> TeleportEvent {
    let followup = input_frame + 40;
    TeleportEvent {
        attacker: 2,
        defender: 1,
        input_frame,
        inv_start_frame: input_frame + 2,
        inv_end_frame: input_frame + 20,
        followup_attack_frame: Some(followup),
        followup_contact_frame: Some(followup + 6),
        airborne: true,
        defender_actionable: true,
        context: TeleportContext::NakedAttack,
        response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.20,
        dp_reachability: DpReachability::Confirmed,
        round_no: 1,
        confidence: 1.0,
    }
}

fn events_with(teleports: Vec<TeleportEvent>) -> MatchEvents {
    MatchEvents {
        teleports,
        ..empty_events()
    }
}

/// テレポートが無ければ何も出さない。
#[test]
fn nothing_is_reported_without_a_teleport() {
    assert!(detect_teleport_defense(&empty_events(), 1).is_none());
}

/// 迎撃できていれば指摘しない。
#[test]
fn a_teleport_that_was_stopped_is_not_reported() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].outcome = ThreatOutcome::Defended;

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// 飛び道具と挟まれていれば、対空を振れる場面ではない。パリィや
/// ガードへ切り替えるのが正解でありうる。
#[test]
fn a_teleport_covered_by_a_projectile_is_not_this_card() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].context = TeleportContext::ProjectileCovered;

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// 攻撃を伴わないテレポートは迎撃する対象ではない。
#[test]
fn a_teleport_without_an_attack_is_not_this_card() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].context = TeleportContext::MovementOnly;
    events.teleports[0].followup_attack_frame = None;

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// 硬直中で動けなかったなら、迎撃できなかったのは当然。
#[test]
fn a_teleport_arriving_while_you_could_not_act_is_not_a_failure() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].defender_actionable = false;

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// 昇竜が届かない位置なら、振っても当たらない。
#[test]
fn a_teleport_out_of_the_anti_airs_reach_is_not_a_failure() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].dp_reachability = DpReachability::OutOfRange;

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// 位置を確認できていない場面では何も言わない。届いたかどうかが
/// 分からないまま「迎撃できたはず」とは言えない。
#[test]
fn an_unmeasured_position_makes_the_card_abstain() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].dp_reachability = DpReachability::Unknown;

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// パリィで受けていれば、対空を選ばなかっただけで回答はしている。
#[test]
fn parrying_it_is_still_an_answer() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].response = Some(DefenseResponse {
        side: 1,
        kind: DefenseResponseKind::Parry,
        start_frame: 130,
        end_frame: 150,
    });

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// 無敵技で受けていれば、対空そのものを振っている。
#[test]
fn answering_with_an_invincible_move_is_still_an_answer() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].response = Some(DefenseResponse {
        side: 1,
        kind: DefenseResponseKind::Invincible,
        start_frame: 130,
        end_frame: 150,
    });

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// ガードで受けたのなら、対空は振っていない。迎撃できたはずの場面
/// として残す。
#[test]
fn blocking_it_still_counts_as_not_intercepting() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].response = Some(DefenseResponse {
        side: 1,
        kind: DefenseResponseKind::Guard,
        start_frame: 130,
        end_frame: 150,
    });

    assert!(
        detect_teleport_defense(&events, 1).is_some(),
        "ガードを対空と同じ扱いにしている"
    );
}

/// HP が減っていなければ被弾した場面ではない。
#[test]
fn a_teleport_that_cost_nothing_is_not_reported() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].damage = 0.0;

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// 相手が受けたテレポートは自分の話ではない。
#[test]
fn a_teleport_the_opponent_defended_is_not_yours() {
    let mut events = events_with(vec![missed_teleport(100)]);
    events.teleports[0].defender = 2;

    assert!(detect_teleport_defense(&events, 1).is_none());
}

/// 条件が揃っていれば、一度でも指摘する。迎撃できたと言い切れる場面
/// なので、読み合いの結果ではない。
#[test]
fn one_confirmed_miss_is_already_worth_saying() {
    let events = events_with(vec![missed_teleport(100)]);

    let card = detect_teleport_defense(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert_eq!(card.hp_lost, Some(0.20));
}

/// 繰り返していれば見出しを書き分ける。一度の見落としと、対空が
/// 間に合っていない状態は違う。
#[test]
fn the_wording_changes_when_it_repeats() {
    let once = events_with(vec![missed_teleport(100)]);
    let twice = events_with(vec![missed_teleport(100), missed_teleport(1000)]);

    let once = detect_teleport_defense(&once, 1).expect("提示される");
    let twice = detect_teleport_defense(&twice, 1).expect("提示される");

    assert_eq!(once.id, twice.id);
    assert_ne!(once.title, twice.title, "見出しを書き分けていない");
    assert!((twice.hp_lost.expect("損失がある") - 0.40).abs() < 1e-6);
}

/// 回数も重みに効く。
#[test]
fn missing_more_often_weighs_more() {
    let once = events_with(vec![TeleportEvent {
        damage: 0.40,
        ..missed_teleport(100)
    }]);
    let twice = events_with(vec![missed_teleport(100), missed_teleport(1000)]);

    let once = detect_teleport_defense(&once, 1).expect("提示される");
    let twice = detect_teleport_defense(&twice, 1).expect("提示される");

    assert_eq!(once.hp_lost, twice.hp_lost, "損失は同じはず");
    assert!(twice.severity > once.severity, "回数が重みに効いていない");
}

/// 説明では、除いた場面があることまで言う。全部のテレポートを課題に
/// しているわけではない。
#[test]
fn the_description_says_what_was_left_out() {
    let card =
        detect_teleport_defense(&events_with(vec![missed_teleport(100)]), 1).expect("提示される");

    assert!(
        card.description.contains("含めていません"),
        "除いた場面の話をしていない: {}",
        card.description
    );
}

/// クリップはテレポートの入力から、攻撃を受けた後まで。入力を映さないと
/// 反応する時間があったのかが分からない。
#[test]
fn the_clip_starts_at_the_teleport_input() {
    let card =
        detect_teleport_defense(&events_with(vec![missed_teleport(100)]), 1).expect("提示される");
    let clip = &card.evidence[0];

    assert_eq!(clip.frame, 100, "入力から始まっていない");
    assert_eq!(clip.end_frame, Some(170), "攻撃の後まで映していない");
}
