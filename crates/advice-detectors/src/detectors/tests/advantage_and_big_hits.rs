//! 有利を取ったあと攻めなかった場面と、原因を分類できなかった大被弾に
//! 対するテスト。
//!
//! 有利のうちに動かないことは、距離の取り直しやゲージ回復として正当で
//! ありうる。だから「攻めなかった」だけでは指摘せず、そのままターンを
//! 返して被弾した場合に限る。
//!
//! 大被弾の一覧は最後の受け皿で、他のカードが既に説明している被弾を
//! 拾ってはいけない。拾うと、同じ場面を二回読まされる。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{
    AdvantageOutcome, AdvantageSituationEvent, AttackDamageConsistency, DamageAttackEvidence,
    DamageEvent, EventConfidence, MatchEvents,
};
use crate::{AdviceCard, AdviceKind, EvidenceClip};

// ── 有利を取ったあと ─────────────────────────────────────────────────────

/// 有利を取った場面。
fn advantage(frame: u32, outcome: AdvantageOutcome, drop: f32) -> AdvantageSituationEvent {
    AdvantageSituationEvent {
        side: 1,
        frame,
        plus_frames: 4,
        follow_up: None,
        action_frame: None,
        pressed: String::new(),
        outcome,
        drop,
        confidence: EventConfidence::High,
        source_contact_frame: frame.saturating_sub(10),
        round_no: 1,
    }
}

/// 有利のうちに次の攻撃を始めた場面。
fn continued(frame: u32) -> AdvantageSituationEvent {
    AdvantageSituationEvent {
        action_frame: Some(frame + 3),
        ..advantage(frame, AdvantageOutcome::Continued, 0.0)
    }
}

fn events_with_advantage(situations: Vec<AdvantageSituationEvent>) -> MatchEvents {
    MatchEvents {
        advantage_situations: situations,
        ..empty_events()
    }
}

/// 攻めなかっただけでは指摘しない。距離を取り直す判断かもしれない。
#[test]
fn simply_not_attacking_is_not_reported() {
    let events = events_with_advantage(vec![
        advantage(100, AdvantageOutcome::Reset, 0.0),
        advantage(600, AdvantageOutcome::Reset, 0.0),
    ]);

    assert!(detect_advantage_abandoned(&events, 1).is_none());
}

/// 攻めを継続できていれば、そもそも放棄ではない。
#[test]
fn continuing_the_attack_is_not_abandoning_it() {
    let events = events_with_advantage(vec![continued(100), continued(600)]);

    assert!(detect_advantage_abandoned(&events, 1).is_none());
}

/// 攻めずにターンを返して被弾していれば、事実として出す。
#[test]
fn losing_the_turn_after_not_attacking_is_reported() {
    let events = events_with_advantage(vec![advantage(100, AdvantageOutcome::TurnLost, 0.15)]);

    let card = detect_advantage_abandoned(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation, "一度で癖と呼んでいる");
    assert_eq!(card.hp_lost, Some(0.15));
}

/// 機会の大半で攻めを止めて、何度も返されていれば癖。
#[test]
fn stopping_on_almost_every_chance_is_a_habit() {
    let events = events_with_advantage(vec![
        advantage(100, AdvantageOutcome::TurnLost, 0.15),
        advantage(600, AdvantageOutcome::TurnLost, 0.12),
        advantage(1200, AdvantageOutcome::Reset, 0.0),
        advantage(1800, AdvantageOutcome::Reset, 0.0),
    ]);

    let card = detect_advantage_abandoned(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis, "偏りを拾えていない");
    assert!((card.hp_lost.expect("損失がある") - 0.27).abs() < 1e-6);
}

/// 攻めを継続できている機会の方が多ければ癖ではない。分母を見ないと、
/// たまたま二度返されただけで癖にされる。
#[test]
fn continuing_more_often_than_stopping_is_not_a_habit() {
    let mut situations = vec![
        advantage(100, AdvantageOutcome::TurnLost, 0.15),
        advantage(600, AdvantageOutcome::TurnLost, 0.12),
    ];
    situations.extend((0..8).map(|index| continued(1200 + index * 300)));

    let card =
        detect_advantage_abandoned(&events_with_advantage(situations), 1).expect("提示される");

    assert_eq!(
        card.kind,
        AdviceKind::Observation,
        "継続できている分を見ていない"
    );
}

/// 止めた割合と平均の有利幅を説明に出す。どのくらい有利だったのかが
/// 分からないと、届く距離だったのかも判断できない。
#[test]
fn the_share_and_the_average_advantage_are_reported() {
    let mut situations = vec![
        advantage(100, AdvantageOutcome::TurnLost, 0.15),
        advantage(400, AdvantageOutcome::Reset, 0.0),
        continued(600),
    ];
    situations[0].plus_frames = 4;
    situations[1].plus_frames = 8;

    let card =
        detect_advantage_abandoned(&events_with_advantage(situations), 1).expect("提示される");

    assert!(
        card.description.contains("66%"),
        "止めた割合が出ていない: {}",
        card.description
    );
    assert!(
        card.description.contains("+6F"),
        "平均の有利幅が出ていない: {}",
        card.description
    );
    assert!(
        card.description.contains("継続できたのは 1 回"),
        "継続できた回数が違う: {}",
        card.description
    );
}

/// 相手が取った有利は自分の話ではない。
#[test]
fn the_opponents_advantage_is_not_yours() {
    let mut events = events_with_advantage(vec![advantage(100, AdvantageOutcome::TurnLost, 0.15)]);
    events.advantage_situations[0].side = 2;

    assert!(detect_advantage_abandoned(&events, 1).is_none());
}

/// 入力まで確認できていない機会は数えない。
#[test]
fn an_unconfirmed_chance_is_not_counted() {
    let mut events = events_with_advantage(vec![advantage(100, AdvantageOutcome::TurnLost, 0.15)]);
    events.advantage_situations[0].confidence = EventConfidence::Low;

    assert!(detect_advantage_abandoned(&events, 1).is_none());
}

/// 一度きりと癖で文面を書き分ける。
#[test]
fn the_advantage_wording_changes_when_it_becomes_a_habit() {
    let once = events_with_advantage(vec![advantage(100, AdvantageOutcome::TurnLost, 0.15)]);
    let habit = events_with_advantage(vec![
        advantage(100, AdvantageOutcome::TurnLost, 0.15),
        advantage(600, AdvantageOutcome::TurnLost, 0.12),
        advantage(1200, AdvantageOutcome::Reset, 0.0),
        advantage(1800, AdvantageOutcome::Reset, 0.0),
    ]);

    let once = detect_advantage_abandoned(&once, 1).expect("提示される");
    let habit = detect_advantage_abandoned(&habit, 1).expect("提示される");

    assert_eq!(once.id, habit.id);
    assert_ne!(once.title, habit.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, habit.description,
        "説明を書き分けていない"
    );
    assert_ne!(once.practice, habit.practice, "練習方法を書き分けていない");
}

/// クリップには有利幅と失った HP を出す。
#[test]
fn the_advantage_clip_says_how_much_advantage_was_held() {
    let events = events_with_advantage(vec![advantage(100, AdvantageOutcome::TurnLost, 0.15)]);

    let card = detect_advantage_abandoned(&events, 1).expect("提示される");
    let label = &card.evidence[0].label;

    assert!(label.contains("+4F"), "有利幅が出ていない: {label}");
    assert!(label.contains("15"), "失った HP が出ていない: {label}");
}

// ── 分類できなかった大被弾 ───────────────────────────────────────────────

fn big_hit(start_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame.saturating_sub(10),
        end_frame: start_frame + 60,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

fn events_with_hits(hits: Vec<DamageEvent>) -> MatchEvents {
    MatchEvents {
        damage: hits,
        ..empty_events()
    }
}

/// 既に別のカードが説明している場面を持つ、そのカード。
fn card_covering(id: &str, frame: u32, end_frame: Option<u32>) -> AdviceCard {
    AdviceCard {
        id: id.to_string(),
        kind: AdviceKind::Observation,
        confidence: EventConfidence::High,
        title: "既存".to_string(),
        severity: 0.0,
        hp_lost: None,
        description: String::new(),
        practice: String::new(),
        evidence: vec![EvidenceClip {
            frame,
            end_frame,
            label: "既存".to_string(),
        }],
    }
}

/// 小さい被弾は一覧に載せない。全部載せると、見るべき場面が埋もれる。
#[test]
fn a_hit_below_the_threshold_is_not_listed() {
    let at_the_edge = events_with_hits(vec![big_hit(1000, 0.18)]);
    let below = events_with_hits(vec![big_hit(1000, 0.17)]);

    assert!(
        detect_big_hits(&at_the_edge, 1, &[]).is_some(),
        "閾値ちょうどの被弾を落としている"
    );
    assert!(
        detect_big_hits(&below, 1, &[]).is_none(),
        "小さい被弾まで載せている"
    );
}

/// 相手が受けた大被弾は自分の話ではない。
#[test]
fn the_opponents_big_hits_are_not_listed() {
    let mut events = events_with_hits(vec![big_hit(1000, 0.25)]);
    events.damage[0].victim = 2;

    assert!(detect_big_hits(&events, 1, &[]).is_none());
}

/// 他のカードが既に説明している被弾は載せない。同じ場面を二回
/// 読まされることになる。
#[test]
fn a_hit_another_card_already_explains_is_left_out() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);
    let existing = vec![card_covering("mashing", 990, Some(1060))];

    assert!(detect_big_hits(&events, 1, &existing).is_none());
}

/// 重なっていない場面は残す。時間の判定が緩いと、無関係な被弾まで
/// 消える。
#[test]
fn a_hit_outside_the_other_cards_window_is_kept() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);
    let existing = vec![card_covering("mashing", 500, Some(989))];

    assert!(
        detect_big_hits(&events, 1, &existing).is_some(),
        "重なっていない場面まで消している"
    );
}

/// 終わりの分からないカードにも見る範囲を与える。与えないと、その
/// カードが扱っている被弾が一覧にも出る。
#[test]
fn a_card_without_an_end_frame_still_covers_a_window() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);
    let existing = vec![card_covering("mashing", 950, None)];

    assert!(detect_big_hits(&events, 1, &existing).is_none());
}

/// カードごとに見る範囲が違う。飛びは着地まで、不利からの暴れは
/// もっと短い。すべて同じにすると、どこかで取りこぼすか消しすぎる。
#[test]
fn each_card_covers_the_span_its_situation_takes() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);

    for id in [
        "layered_defense",
        "mashing",
        "committed_button_vs_di",
        "teleport_defense",
        "reversal_punished",
        "punish_fail",
        "anti_air",
        "own_jumps",
        "press_while_minus",
        "throw_while_minus",
        "guard_break",
        "throw_loop",
    ] {
        let existing = vec![card_covering(id, 995, None)];

        assert!(
            detect_big_hits(&events, 1, &existing).is_none(),
            "{id} が自分の場面を覆えていない"
        );
    }
}

/// 知らないカードの場面は覆えない。範囲を決められないものを勝手に
/// 覆うと、説明の無い被弾が一覧から消える。
#[test]
fn an_unknown_card_does_not_hide_a_hit() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);
    let existing = vec![card_covering("something_new", 995, Some(1060))];

    assert!(
        detect_big_hits(&events, 1, &existing).is_some(),
        "範囲の分からないカードで被弾を消している"
    );
}

/// 投げの指摘は終わりが分かっている場面だけを覆う。分からない場面まで
/// 覆うと、投げ以外の大被弾が消える。
#[test]
fn the_throw_cards_only_cover_what_they_measured() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);

    let measured = vec![card_covering("throw_whiff_punished", 995, Some(1060))];
    assert!(detect_big_hits(&events, 1, &measured).is_none());

    let unmeasured = vec![card_covering("throw_whiff_punished", 995, None)];
    assert!(
        detect_big_hits(&events, 1, &unmeasured).is_some(),
        "測れていない場面まで覆っている"
    );
}

/// 一覧は判断ではなく事実の並べ替え。
#[test]
fn the_list_stays_an_observation() {
    let events = events_with_hits(vec![big_hit(1000, 0.25), big_hit(2000, 0.30)]);

    let card = detect_big_hits(&events, 1, &[]).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert!((card.hp_lost.expect("損失がある") - 0.55).abs() < 1e-6);
    assert_eq!(card.evidence.len(), 2);
}

/// クリップは演出の前から被弾の終わりまで。演出で始まる被弾は、
/// 被弾フレームから再生すると何が起きたか映らない。
#[test]
fn the_clip_starts_before_the_freeze() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);

    let card = detect_big_hits(&events, 1, &[]).expect("提示される");

    assert_eq!(card.evidence[0].frame, 990, "演出の前から始まっていない");
    assert_eq!(card.evidence[0].end_frame, Some(1060));
}

/// クリップの見出しには失った HP と残り HP を出す。
#[test]
fn the_clip_says_what_it_cost_and_what_was_left() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);

    let card = detect_big_hits(&events, 1, &[]).expect("提示される");
    let label = &card.evidence[0].label;

    assert!(label.contains("25"), "失った HP が出ていない: {label}");
    assert!(label.contains("75"), "残り HP が出ていない: {label}");
}

/// ゲーム内表示まで読めていれば、その数字を添える。
#[test]
fn a_reliable_in_game_reading_is_shown() {
    let mut events = events_with_hits(vec![big_hit(1000, 0.25)]);
    events.attack_evidence.damage = vec![DamageAttackEvidence {
        victim: 1,
        attacker: 2,
        damage_start_frame: 1000,
        sequence_start_frame: 1000,
        sequence_end_frame: 1060,
        combo_damage: 3200,
        sequence_count: 2,
        final_scaling_percent: 60,
        starter_attribute: Some(crate::attack_info::AttackAttribute::Lower),
        final_attribute: crate::attack_info::AttackAttribute::Middle,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Consistent,
        sequence_indices: vec![],
    }];

    let card = detect_big_hits(&events, 1, &[]).expect("提示される");
    let label = &card.evidence[0].label;

    assert!(label.contains("3200"), "累積ダメージが出ていない: {label}");
    assert!(
        label.contains("下段始動"),
        "始動の属性が出ていない: {label}"
    );
    assert!(label.contains("2連係合計"), "連係の数が出ていない: {label}");
    assert!(label.contains("60%補正"), "補正率が出ていない: {label}");
    assert!(
        card.description.contains("1 件はゲーム内表示"),
        "読めた件数を出していない: {}",
        card.description
    );
}

/// HP と食い違う表示は、食い違っていることまで書く。黙って出すと、
/// 誤った数字を信じることになる。
#[test]
fn a_reading_that_disagrees_with_the_health_says_so() {
    let mut events = events_with_hits(vec![big_hit(1000, 0.25)]);
    events.attack_evidence.damage = vec![DamageAttackEvidence {
        victim: 1,
        attacker: 2,
        damage_start_frame: 1000,
        sequence_start_frame: 1000,
        sequence_end_frame: 1060,
        combo_damage: 3200,
        sequence_count: 1,
        final_scaling_percent: 100,
        starter_attribute: None,
        final_attribute: crate::attack_info::AttackAttribute::Middle,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Mismatch,
        sequence_indices: vec![],
    }];

    let card = detect_big_hits(&events, 1, &[]).expect("提示される");

    assert!(
        card.evidence[0].label.contains("不一致"),
        "食い違いを黙っている: {}",
        card.evidence[0].label
    );
    assert!(
        card.description.contains("不一致は 1 件"),
        "食い違いの件数を出していない: {}",
        card.description
    );
}

/// 読めていなければ、その話は書かない。
#[test]
fn nothing_is_said_about_a_reading_that_was_not_taken() {
    let events = events_with_hits(vec![big_hit(1000, 0.25)]);

    let card = detect_big_hits(&events, 1, &[]).expect("提示される");

    assert!(
        !card.description.contains("ゲーム内表示"),
        "読めていない話をしている: {}",
        card.description
    );
}
