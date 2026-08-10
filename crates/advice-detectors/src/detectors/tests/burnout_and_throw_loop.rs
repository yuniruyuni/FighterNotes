//! バーンアウトの収支と、投げを連続で受けた場面に対するテスト。
//!
//! どちらも「起きた回数」だけでは意味が決まらない。バーンアウトは攻めの
//! ために使い切ったのか守りで削られたのかで話が違い、投げは時々通されるのは
//! 正常な読み合いで、連続して初めて守り方の問題になる。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{BurnoutCause, BurnoutPeriod, EventConfidence, MatchEvents, ThrowEvent};
use crate::AdviceKind;

// ── バーンアウト ─────────────────────────────────────────────────────────

fn burnout(start: u32, seconds: u32, cause: BurnoutCause) -> BurnoutPeriod {
    BurnoutPeriod {
        side: 1,
        start_frame: start,
        end_frame: start + seconds * 60,
        hp_lost: 0.10,
        hp_dealt: 0.05,
        cause,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn events_with_burnouts(periods: Vec<BurnoutPeriod>) -> MatchEvents {
    MatchEvents {
        burnouts: periods,
        ..empty_events()
    }
}

/// 一度も入っていなければ何も出さない。
#[test]
fn no_burnout_means_no_card() {
    assert!(detect_burnout(&empty_events(), 1).is_none());
}

/// バーンアウトは良し悪しの判断ではなく収支の記録。回数によらず
/// 統計として出す。
#[test]
fn a_burnout_is_reported_as_a_tally() {
    let events = events_with_burnouts(vec![burnout(100, 5, BurnoutCause::SelfInitiated)]);

    let card = detect_burnout(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Statistic, "判断に踏み込んでいる");
    assert_eq!(card.hp_lost, Some(0.10));
}

/// 相手のバーンアウトは自分の話ではない。
#[test]
fn the_opponents_burnout_is_not_reported() {
    let mut events = events_with_burnouts(vec![burnout(100, 5, BurnoutCause::SelfInitiated)]);
    events.burnouts[0].side = 2;

    assert!(detect_burnout(&events, 1).is_none());
}

/// 突入の理由ごとに数える。攻めで使い切ったのか守りで削られたのかは
/// 直し方が違うので、まとめてしまうと何をすればよいか分からない。
#[test]
fn the_causes_are_counted_separately() {
    let events = events_with_burnouts(vec![
        burnout(100, 3, BurnoutCause::SelfInitiated),
        burnout(1000, 3, BurnoutCause::SelfInitiated),
        burnout(2000, 3, BurnoutCause::ForcedByGuard),
        burnout(3000, 3, BurnoutCause::Mixed),
    ]);

    let card = detect_burnout(&events, 1).expect("提示される");

    assert!(
        card.description.contains("自分のゲージ使用 2 回"),
        "理由ごとに数えていない: {}",
        card.description
    );
    assert!(
        card.description.contains("ガードで削られた場面 1 回"),
        "{}",
        card.description
    );
    assert!(
        card.description.contains("両方 1 回"),
        "{}",
        card.description
    );
    assert!(
        card.description.contains("分類保留 0 回"),
        "分類できた分まで保留に数えている: {}",
        card.description
    );
}

/// 理由を決められなかった期間は保留として残す。無理に振り分けると、
/// 見直す場面を取り違える。
#[test]
fn an_unclassified_burnout_stays_unclassified() {
    let events = events_with_burnouts(vec![burnout(100, 3, BurnoutCause::Unknown)]);

    let card = detect_burnout(&events, 1).expect("提示される");

    assert!(
        card.description.contains("分類保留 1 回"),
        "保留を数えていない: {}",
        card.description
    );
}

/// 長さは秒で出す。1 秒に満たない場面は、時間ではなくラウンドの
/// 終わりまで続いたこととして書く。
#[test]
fn a_burnout_shorter_than_a_second_is_described_by_its_end() {
    let long = events_with_burnouts(vec![burnout(100, 5, BurnoutCause::SelfInitiated)]);
    let brief = events_with_burnouts(vec![BurnoutPeriod {
        end_frame: 130,
        ..burnout(100, 0, BurnoutCause::SelfInitiated)
    }]);

    let long = detect_burnout(&long, 1).expect("提示される");
    let brief = detect_burnout(&brief, 1).expect("提示される");

    assert!(long.description.contains("5 秒"), "{}", long.description);
    assert!(
        brief.description.contains("ラウンド終了まで"),
        "短い期間を秒で書いている: {}",
        brief.description
    );
}

/// 期間の収支は足し合わせる。被ダメだけでなく与ダメも出さないと、
/// 攻めのために使った分が損だけに見える。
#[test]
fn the_exchange_over_all_periods_is_summed() {
    let events = events_with_burnouts(vec![
        burnout(100, 3, BurnoutCause::SelfInitiated),
        burnout(1000, 3, BurnoutCause::SelfInitiated),
    ]);

    let card = detect_burnout(&events, 1).expect("提示される");

    assert!((card.hp_lost.expect("損失がある") - 0.20).abs() < 1e-6);
    assert!(
        card.description.contains("与ダメは 10%"),
        "与ダメを出していない: {}",
        card.description
    );
}

/// 一つでも読み取りが怪しい期間があれば、全体の確度を下げる。
#[test]
fn one_uncertain_period_lowers_the_confidence() {
    let mut events = events_with_burnouts(vec![
        burnout(100, 3, BurnoutCause::SelfInitiated),
        burnout(1000, 3, BurnoutCause::SelfInitiated),
    ]);

    let sure = detect_burnout(&events, 1).expect("提示される");
    events.burnouts[1].confidence = EventConfidence::Medium;
    let unsure = detect_burnout(&events, 1).expect("提示される");

    assert_eq!(sure.confidence, EventConfidence::High);
    assert_eq!(
        unsure.confidence,
        EventConfidence::Medium,
        "怪しい期間を混ぜても確度を下げていない"
    );
}

/// 入った回数そのものも重みに効く。同じ被ダメでも、何度も入って
/// いる方が見直す価値が高い。
#[test]
fn entering_more_often_weighs_more() {
    let once = events_with_burnouts(vec![BurnoutPeriod {
        hp_lost: 0.20,
        ..burnout(100, 3, BurnoutCause::SelfInitiated)
    }]);
    let twice = events_with_burnouts(vec![
        burnout(100, 3, BurnoutCause::SelfInitiated),
        burnout(1000, 3, BurnoutCause::SelfInitiated),
    ]);

    let once = detect_burnout(&once, 1).expect("提示される");
    let twice = detect_burnout(&twice, 1).expect("提示される");

    assert_eq!(once.hp_lost, twice.hp_lost, "被ダメは同じはず");
    assert!(
        twice.severity > once.severity,
        "回数が重みに効いていない: {} / {}",
        twice.severity,
        once.severity
    );
}

// ── 投げの連続 ───────────────────────────────────────────────────────────

fn throw(frame: u32, connected: bool) -> ThrowEvent {
    ThrowEvent {
        thrower: 2,
        frame,
        connected,
        round_no: 1,
    }
}

fn events_with_throws(throws: Vec<ThrowEvent>) -> MatchEvents {
    MatchEvents {
        throws,
        ..empty_events()
    }
}

/// 投げが一度も通っていなければ何も出さない。振られただけでは
/// 守り方の話にならない。
#[test]
fn throws_that_never_connect_are_not_reported() {
    let events = events_with_throws(vec![throw(100, false), throw(400, false)]);

    assert!(detect_throw_loop(&events, 2).is_none());
}

/// 時々投げられるのは正常な読み合い。事実確認に留める。
#[test]
fn being_thrown_now_and_then_stays_an_observation() {
    let events = events_with_throws(vec![throw(100, true), throw(400, false), throw(700, true)]);

    let card = detect_throw_loop(&events, 2).expect("提示される");

    assert_usable(&card);
    assert_eq!(
        card.kind,
        AdviceKind::Observation,
        "読み合いを癖と呼んでいる"
    );
}

/// 連続して投げられていれば、同じ守り方が続いている疑い。
#[test]
fn a_streak_of_throws_becomes_a_diagnosis() {
    let events = events_with_throws(vec![throw(100, true), throw(400, true), throw(700, true)]);

    let card = detect_throw_loop(&events, 2).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis, "連続を拾えていない");
    assert!(
        card.description.contains("最大 3 回連続"),
        "連続数を出していない: {}",
        card.description
    );
}

/// 途中で投げを防げていれば連続は途切れる。防いだ回を無視すると、
/// 守れている場面まで癖に数える。
#[test]
fn escaping_one_throw_breaks_the_streak() {
    let events = events_with_throws(vec![
        throw(100, true),
        throw(400, true),
        throw(700, false),
        throw(1000, true),
    ]);

    let card = detect_throw_loop(&events, 2).expect("提示される");

    assert_eq!(card.kind, AdviceKind::Observation, "抜けた回を無視している");
    assert!(
        card.description.contains("最大連続被投げは 2 回"),
        "途切れを数えていない: {}",
        card.description
    );
}

/// 時間が大きく空いた投げは別の場面。同じ起き攻めが続いたわけでは
/// ないので、連続としては数えない。
#[test]
fn throws_far_apart_are_not_one_streak() {
    let together = events_with_throws(vec![throw(100, true), throw(400, true), throw(1200, true)]);
    let apart = events_with_throws(vec![throw(100, true), throw(400, true), throw(1301, true)]);

    let together = detect_throw_loop(&together, 2).expect("提示される");
    let apart = detect_throw_loop(&apart, 2).expect("提示される");

    assert_eq!(
        together.kind,
        AdviceKind::Diagnosis,
        "続いた投げを別々に数えている"
    );
    assert_eq!(
        apart.kind,
        AdviceKind::Observation,
        "離れた投げを繋いで数えている"
    );
}

/// 自分の投げは相手の攻めではない。
#[test]
fn your_own_throws_are_not_counted_against_you() {
    let mut events = events_with_throws(vec![throw(100, true), throw(400, true), throw(700, true)]);
    for event in &mut events.throws {
        event.thrower = 1;
    }

    assert!(detect_throw_loop(&events, 2).is_none());
}

/// 通らなかった投げも、振られた回数として数に入れる。分母が無いと
/// 「何回中何回通されたか」が言えない。
#[test]
fn the_attempts_that_missed_still_count_as_attempts() {
    let events = events_with_throws(vec![throw(100, true), throw(400, false), throw(700, true)]);

    let card = detect_throw_loop(&events, 2).expect("提示される");

    assert!(
        card.description.contains("3 回中 2 回"),
        "振られた回数を数えていない: {}",
        card.description
    );
}

/// 通された回数が多いほど重く扱う。
#[test]
fn more_throws_landed_weighs_more() {
    let few = events_with_throws(vec![throw(100, true)]);
    let many = events_with_throws(vec![throw(100, true), throw(400, true), throw(700, true)]);

    let few = detect_throw_loop(&few, 2).expect("提示される");
    let many = detect_throw_loop(&many, 2).expect("提示される");

    assert!(
        many.severity > few.severity,
        "通された回数が重みに効いていない"
    );
}

/// 投げの被ダメージは記録に無い。推定で埋めず、空のままにする。
/// 埋めると、他の指摘と並べたときに嘘の数字で順位が付く。
#[test]
fn the_throw_card_does_not_guess_at_the_health_lost() {
    let events = events_with_throws(vec![throw(100, true)]);

    let card = detect_throw_loop(&events, 2).expect("提示される");

    assert_eq!(card.hp_lost, None, "記録に無い被ダメを埋めている");
}

/// 連続かどうかで文面を書き分ける。
#[test]
fn the_throw_wording_changes_with_a_streak() {
    let occasional = events_with_throws(vec![throw(100, true)]);
    let streak = events_with_throws(vec![throw(100, true), throw(400, true), throw(700, true)]);

    let occasional = detect_throw_loop(&occasional, 2).expect("提示される");
    let streak = detect_throw_loop(&streak, 2).expect("提示される");

    assert_eq!(occasional.id, streak.id);
    assert_ne!(occasional.title, streak.title, "見出しを書き分けていない");
    assert_ne!(
        occasional.description, streak.description,
        "説明を書き分けていない"
    );
    assert_ne!(
        occasional.practice, streak.practice,
        "練習方法を書き分けていない"
    );
}

/// 根拠のクリップは通された投げだけ。防いだ投げまで並べると、
/// 何を見直すのか分からなくなる。
#[test]
fn only_the_throws_that_landed_get_a_clip() {
    let events = events_with_throws(vec![throw(100, true), throw(400, false), throw(700, true)]);

    let card = detect_throw_loop(&events, 2).expect("提示される");

    assert_eq!(card.evidence.len(), 2, "防いだ投げまで並べている");
    assert_eq!(card.evidence[0].frame, 100);
    assert_eq!(card.evidence[1].frame, 700);
}
