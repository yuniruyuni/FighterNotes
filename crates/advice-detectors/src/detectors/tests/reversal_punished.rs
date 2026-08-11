//! 切り返しが通らず狩られた場面に対するテスト。
//!
//! 無敵技も SA/CA も、打撃重ねへの正しい回答になりうる。狩られたこと
//! そのものは失敗ではなく、同じ回答を繰り返して狩られていることが
//! 見直しの材料になる。
//!
//! 一つの切り返しが無敵技と SA の両方として記録されることがあるので、
//! 二重に数えないことも要る。二重に数えると、一度の被弾が「繰り返し」に
//! 化ける。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{
    EventConfidence, MatchEvents, ReversalEvent, SuperArtContext, SuperArtEvent, SuperArtOutcome,
};
use crate::AdviceKind;

/// 狩られた無敵技。
fn reversal(frame: u32, drop: f32) -> ReversalEvent {
    ReversalEvent {
        side: 1,
        frame,
        drop,
        blocked: true,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

/// 後隙を狩られた SA。
fn punished_super(frame: u32, level: u8, damage: f32) -> SuperArtEvent {
    SuperArtEvent {
        side: 1,
        frame,
        gauge_drop_frame: frame + 5,
        level,
        critical_art: false,
        gauge_before: 3.0,
        gauge_after: 3.0 - level as f32,
        context: SuperArtContext::DefensiveReversal,
        outcome: SuperArtOutcome::Blocked,
        contact_frame: Some(frame + 10),
        damage: 0.0,
        ko: false,
        punished: true,
        punished_damage: damage,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn events_with(reversals: Vec<ReversalEvent>, supers: Vec<SuperArtEvent>) -> MatchEvents {
    MatchEvents {
        reversals,
        super_arts: supers,
        ..empty_events()
    }
}

/// 狩られた切り返しが無ければ何も出さない。
#[test]
fn nothing_is_reported_without_a_punished_reversal() {
    assert!(detect_reversal_punished(&empty_events(), 1).is_none());
}

/// 通った SA は狩られていない。
#[test]
fn a_super_that_was_not_punished_is_not_reported() {
    let mut events = events_with(vec![], vec![punished_super(100, 3, 0.20)]);
    events.super_arts[0].punished = false;

    assert!(detect_reversal_punished(&events, 1).is_none());
}

/// 一度きりは読み負けと区別が付かないので、事実確認に留める。
#[test]
fn a_single_punished_reversal_stays_an_observation() {
    let events = events_with(vec![reversal(100, 0.25)], vec![]);

    let card = detect_reversal_punished(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation, "一度で診断にしている");
    assert_eq!(card.hp_lost, Some(0.25));
    assert!((card.severity - 0.27).abs() < 1e-6);
}

/// 繰り返していれば、同じ防御回答に偏っている疑いとして診断にする。
#[test]
fn repeated_punished_reversals_become_a_diagnosis() {
    let events = events_with(vec![reversal(100, 0.25), reversal(1000, 0.20)], vec![]);

    let card = detect_reversal_punished(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert!((card.hp_lost.expect("損失がある") - 0.45).abs() < 1e-6);
    assert!((card.severity - 0.49).abs() < 1e-6);
    assert_eq!(card.evidence.len(), 2);
}

/// 同じ切り返しが無敵技と SA の両方で記録されていれば、一つとして数える。
/// 二重に数えると、一度の被弾が繰り返しに化ける。
#[test]
fn one_reversal_recorded_twice_is_counted_once() {
    let events = events_with(
        vec![reversal(100, 0.25)],
        vec![punished_super(160, 3, 0.25)],
    );

    let card = detect_reversal_punished(&events, 1).expect("提示される");

    assert_eq!(card.evidence.len(), 1, "同じ場面を二件に割っている");
    assert_eq!(card.hp_lost, Some(0.25), "同じ被弾を二重に数えている");
    assert_eq!(
        card.kind,
        AdviceKind::Observation,
        "二重計上で繰り返しにしている"
    );
}

/// 離れていれば別の場面。まとめすぎると、二度の失敗が一度に見える。
#[test]
fn a_reversal_and_a_super_far_apart_are_two_situations() {
    let events = events_with(
        vec![reversal(100, 0.25)],
        vec![punished_super(221, 3, 0.20)],
    );

    let card = detect_reversal_punished(&events, 1).expect("提示される");

    assert_eq!(card.evidence.len(), 2, "別の場面をまとめている");
    assert_eq!(card.kind, AdviceKind::Diagnosis);
}

/// ラウンドが違えば、フレームが近くても別の場面。
#[test]
fn the_same_frames_in_another_round_are_a_separate_situation() {
    let mut events = events_with(
        vec![reversal(100, 0.25)],
        vec![punished_super(160, 3, 0.20)],
    );
    events.super_arts[0].round_no = 2;

    let card = detect_reversal_punished(&events, 1).expect("提示される");

    assert_eq!(card.evidence.len(), 2, "ラウンドをまたいでまとめている");
}

/// SA だけで起きている場合は、無敵技の話と混ぜない。ゲージを使う
/// 回答と使わない回答では、直し方が違う。
#[test]
fn a_super_only_situation_gets_its_own_wording() {
    let supers_only = events_with(vec![], vec![punished_super(100, 3, 0.20)]);
    let with_reversal = events_with(vec![reversal(100, 0.25)], vec![]);

    let supers_only = detect_reversal_punished(&supers_only, 1).expect("提示される");
    let with_reversal = detect_reversal_punished(&with_reversal, 1).expect("提示される");

    assert_ne!(
        supers_only.title, with_reversal.title,
        "SA と無敵技を同じ文面で出している"
    );
    assert!(
        supers_only.description.contains("SA3"),
        "使った SA が出ていない: {}",
        supers_only.description
    );
}

/// SA の見出しにはレベルを出す。SA1 と SA3 では消費も意味も違う。
#[test]
fn the_super_level_appears_in_the_clip_label() {
    let events = events_with(vec![], vec![punished_super(100, 1, 0.20)]);

    let card = detect_reversal_punished(&events, 1).expect("提示される");

    assert!(
        card.evidence[0].label.contains("SA1"),
        "レベルが出ていない: {}",
        card.evidence[0].label
    );
}

/// CA はレベルではなく CA として出す。
#[test]
fn a_critical_art_is_named_as_such() {
    let mut events = events_with(vec![], vec![punished_super(100, 3, 0.20)]);
    events.super_arts[0].critical_art = true;

    let card = detect_reversal_punished(&events, 1).expect("提示される");

    assert!(
        card.evidence[0].label.contains("CA"),
        "CA と書いていない: {}",
        card.evidence[0].label
    );
}

/// ガードされたのか空振ったのかは見出しで書き分ける。同じ「狩られた」
/// でも、間合いの話か読みの話かが変わる。
#[test]
fn the_clip_says_whether_it_was_blocked_or_whiffed() {
    let blocked = events_with(vec![reversal(100, 0.25)], vec![]);
    let mut whiffed = blocked.clone();
    whiffed.reversals[0].blocked = false;

    let blocked = detect_reversal_punished(&blocked, 1).expect("提示される");
    let whiffed = detect_reversal_punished(&whiffed, 1).expect("提示される");

    assert!(
        blocked.evidence[0].label.contains("ガード"),
        "{}",
        blocked.evidence[0].label
    );
    assert!(
        whiffed.evidence[0].label.contains("空振り"),
        "{}",
        whiffed.evidence[0].label
    );
}

/// 相手の切り返しは自分の話ではない。
#[test]
fn the_opponents_reversals_are_not_reported() {
    let mut events = events_with(vec![reversal(100, 0.25)], vec![]);
    events.reversals[0].side = 2;

    assert!(detect_reversal_punished(&events, 1).is_none());
}

/// 読み取りが怪しい場面は扱わない。無敵技を撃ったかどうか曖昧なまま
/// 「偏っている」とは言えない。
#[test]
fn an_unconfirmed_reversal_is_not_reported() {
    let mut events = events_with(vec![reversal(100, 0.25)], vec![]);
    events.reversals[0].confidence = EventConfidence::Low;

    assert!(detect_reversal_punished(&events, 1).is_none());
}

/// 怪しい SA も同じく扱わない。
#[test]
fn an_unconfirmed_super_is_not_reported() {
    let mut events = events_with(vec![], vec![punished_super(100, 3, 0.20)]);
    events.super_arts[0].confidence = EventConfidence::Low;

    assert!(detect_reversal_punished(&events, 1).is_none());
}

/// クリップは時間順に並べる。SA と無敵技を別々に集めて繋いだままだと、
/// 見る順序が試合の順序とずれる。
#[test]
fn the_clips_are_ordered_by_time() {
    let events = events_with(
        vec![reversal(2000, 0.25)],
        vec![punished_super(500, 3, 0.20)],
    );

    let card = detect_reversal_punished(&events, 1).expect("提示される");

    assert_eq!(card.evidence[0].frame, 500, "時間順に並んでいない");
    assert_eq!(card.evidence[1].frame, 2000);
}

/// 回数そのものも重みに効く。同じ被ダメでも、繰り返している方が
/// 見直す価値が高い。
#[test]
fn repeating_the_answer_adds_to_the_weight() {
    let once = events_with(vec![reversal(100, 0.40)], vec![]);
    let twice = events_with(vec![reversal(100, 0.20), reversal(1000, 0.20)], vec![]);

    let once = detect_reversal_punished(&once, 1).expect("提示される");
    let twice = detect_reversal_punished(&twice, 1).expect("提示される");

    assert_eq!(once.hp_lost, twice.hp_lost, "損失は同じはず");
    assert!(twice.severity > once.severity, "回数が重みに効いていない");
}

/// 四つの場合それぞれで文面を書き分ける。SA か無敵技か、一度きりか
/// 繰り返しかで、確認すべきことが違う。
#[test]
fn each_of_the_four_cases_has_its_own_wording() {
    let cases = [
        events_with(vec![reversal(100, 0.25)], vec![]),
        events_with(vec![reversal(100, 0.25), reversal(1000, 0.20)], vec![]),
        events_with(vec![], vec![punished_super(100, 3, 0.20)]),
        events_with(
            vec![],
            vec![punished_super(100, 3, 0.20), punished_super(1000, 3, 0.15)],
        ),
    ];

    let cards: Vec<_> = cases
        .iter()
        .map(|events| detect_reversal_punished(events, 1).expect("提示される"))
        .collect();

    for (left, card) in cards.iter().enumerate() {
        assert_usable(card);
        for (right, other) in cards.iter().enumerate().skip(left + 1) {
            assert_ne!(card.title, other.title, "{left} と {right} の見出しが同じ");
            assert_ne!(
                card.description, other.description,
                "{left} と {right} の説明が同じ"
            );
            assert_ne!(
                card.practice, other.practice,
                "{left} と {right} の練習方法が同じ"
            );
        }
    }
}
