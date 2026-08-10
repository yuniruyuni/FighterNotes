//! ラウンドの要約と、レポートの冒頭に出る一文に対するテスト。
//!
//! 冒頭の一文は、レポート全体で最初に読まれる。「改善点なし」と
//! 「読み取れなかったので判らない」を取り違えると、直す点があるのに
//! 無いと伝えることになる。

use super::*;
use crate::match_events::{BurnoutCause, BurnoutPeriod, DamageEvent, RoundInfo};
use match_event_layer::test_support::empty_events;

/// 自分が受けた被弾。
fn taken(round_no: u32, start_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 30,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no,
    }
}

fn round(round_no: u32, start_frame: u32, winner: Option<u8>) -> RoundInfo {
    RoundInfo {
        round_no,
        start_frame,
        end_frame: start_frame + 3_000,
        winner,
        p1_hp_end: 0.4,
        p2_hp_end: 0.8,
    }
}

// ── 被弾の一覧 ───────────────────────────────────────────────────────────

/// 自分が受けた被弾だけを並べる。
#[test]
fn only_the_damage_you_took_is_listed() {
    let mut events = empty_events();
    let mut theirs = taken(1, 200, 0.2);
    theirs.victim = 2;
    events.damage = vec![taken(1, 100, 0.1), theirs];

    let listed = build_damage_taken_events(&events, 1);

    assert_eq!(listed.len(), 1, "相手の被弾を並べている");
    assert_eq!(listed[0].frame, 100);
    assert!((listed[0].hp_drop - 0.1).abs() < 1e-6);
}

/// 被弾の前後の残量も残す。どこまで減ったのかが分かる。
#[test]
fn the_health_before_and_after_are_kept() {
    let mut events = empty_events();
    events.damage = vec![taken(1, 100, 0.1)];

    let listed = build_damage_taken_events(&events, 1);

    assert!((listed[0].own_hp_before - 1.0).abs() < 1e-6);
    assert!((listed[0].own_hp_after - 0.9).abs() < 1e-6);
}

// ── ラウンドの要約 ───────────────────────────────────────────────────────

/// 勝敗は自分から見た結果として持つ。
#[test]
fn the_result_is_recorded_from_your_own_side() {
    let mut events = empty_events();
    events.rounds = vec![round(1, 0, Some(1))];

    assert_eq!(build_round_summaries(&events, 1, 2)[0].won, Some(true));
    assert_eq!(build_round_summaries(&events, 2, 1)[0].won, Some(false));
}

/// 勝敗が読めなければ空のまま。負けにしない。
#[test]
fn an_unread_result_stays_empty() {
    let mut events = empty_events();
    events.rounds = vec![round(1, 0, None)];

    let summaries = build_round_summaries(&events, 1, 2);

    assert_eq!(summaries[0].won, None);
    assert_eq!(
        summaries[0].detection_confidence, "medium",
        "読めていないのに確度を下げていない"
    );
}

/// 勝敗が読めていれば確度は高い。
#[test]
fn a_read_result_is_recorded_with_high_confidence() {
    let mut events = empty_events();
    events.rounds = vec![round(1, 0, Some(1))];

    assert_eq!(
        build_round_summaries(&events, 1, 2)[0].detection_confidence,
        "high"
    );
}

/// 残量も自分から見た向きで持つ。
#[test]
fn the_end_health_follows_your_own_side() {
    let mut events = empty_events();
    events.rounds = vec![round(1, 0, Some(1))];

    let mine = build_round_summaries(&events, 1, 2);
    let theirs = build_round_summaries(&events, 2, 1);

    assert!((mine[0].own_hp_end - 0.4).abs() < 1e-6);
    assert!((theirs[0].own_hp_end - 0.8).abs() < 1e-6);
}

/// 被弾はラウンドごとに数える。またいで数えると、どのラウンドが
/// 荒れたのか分からない。
#[test]
fn the_damage_is_counted_per_round() {
    let mut events = empty_events();
    events.rounds = vec![round(1, 0, Some(2)), round(2, 4_000, Some(1))];
    events.damage = vec![
        taken(1, 100, 0.1),
        taken(1, 500, 0.2),
        taken(2, 4_100, 0.15),
    ];

    let summaries = build_round_summaries(&events, 1, 2);

    assert_eq!(summaries[0].own_hits_taken, 2);
    assert!((summaries[0].own_hp_lost - 0.3).abs() < 1e-6);
    assert_eq!(summaries[1].own_hits_taken, 1);
    assert!((summaries[1].own_hp_lost - 0.15).abs() < 1e-6);
}

/// 与えた分も数える。攻めが通っていたのかが分かる。
#[test]
fn the_damage_you_dealt_is_counted_too() {
    let mut events = empty_events();
    events.rounds = vec![round(1, 0, Some(1))];
    let mut theirs = taken(1, 200, 0.25);
    theirs.victim = 2;
    events.damage = vec![taken(1, 100, 0.1), theirs];

    let summaries = build_round_summaries(&events, 1, 2);

    assert!((summaries[0].opp_hp_lost - 0.25).abs() < 1e-6);
    assert!((summaries[0].own_hp_lost - 0.1).abs() < 1e-6);
}

/// 開幕の被弾は、そのラウンドの頭から数えた時間で決まる。
#[test]
fn the_early_hit_flag_is_measured_from_the_rounds_own_start() {
    let mut events = empty_events();
    events.rounds = vec![round(2, 4_000, Some(2))];

    events.damage = vec![taken(2, 4_179, 0.1)];
    assert!(build_round_summaries(&events, 1, 2)[0].early_hit);

    events.damage = vec![taken(2, 4_180, 0.1)];
    assert!(
        !build_round_summaries(&events, 1, 2)[0].early_hit,
        "窓の外の被弾を開幕にしている"
    );
}

/// バーンアウトの回数もラウンドごとに数える。
#[test]
fn the_burnouts_are_counted_per_round_and_per_side() {
    let mut events = empty_events();
    events.rounds = vec![round(1, 0, Some(2))];
    let burnout = |side, round_no| BurnoutPeriod {
        side,
        start_frame: 100,
        end_frame: 400,
        hp_lost: 0.1,
        hp_dealt: 0.0,
        cause: BurnoutCause::SelfInitiated,
        confidence: EventConfidence::High,
        round_no,
    };
    events.burnouts = vec![burnout(1, 1), burnout(2, 1), burnout(1, 2)];

    assert_eq!(build_round_summaries(&events, 1, 2)[0].own_burnouts, 1);
}

// ── 冒頭の一文 ───────────────────────────────────────────────────────────

fn card(id: &str, kind: AdviceKind) -> AdviceCard {
    AdviceCard {
        id: id.to_string(),
        kind,
        confidence: EventConfidence::High,
        title: format!("{id} の見出し"),
        severity: 0.1,
        hp_lost: None,
        description: format!("{id} の説明"),
        practice: format!("{id} の練習"),
        evidence: Vec::new(),
    }
}

fn suppressed(id: &str) -> SuppressedAdviceCard {
    SuppressedAdviceCard {
        id: id.to_string(),
        title: id.to_string(),
        missing_requirements: vec![EvidenceRequirement::OwnInput],
    }
}

fn everything_available() -> AnalysisCoverage {
    let available = EvidenceAvailability::Available;
    AnalysisCoverage {
        availability: Some(AnalysisAvailability {
            own_hp: available,
            opponent_hp: available,
            own_drive: available,
            opponent_drive: available,
            own_super: available,
            opponent_super: available,
            own_input: available,
            opponent_input: available,
            own_meter: available,
            opponent_meter: available,
            contacts: available,
            punishes: available,
            spatial: available,
            own_attack_info: available,
            opponent_attack_info: available,
        }),
        ..AnalysisCoverage::default()
    }
}

/// 診断があれば、それを最優先として名指しする。
#[test]
fn a_diagnosis_is_named_as_the_thing_to_fix_first() {
    let cards = vec![card("mashing", AdviceKind::Diagnosis)];

    let (_, _, summary) = build_compatibility_summary(&cards, &[], 3, 5, &everything_available());

    assert!(summary.contains("優先改善"), "{summary}");
    assert!(summary.contains("mashing の見出し"), "{summary}");
}

/// 診断が無く事実確認だけなら、断定はせず確認を促す。
#[test]
fn without_a_diagnosis_the_summary_only_asks_for_a_look() {
    let cards = vec![card("big_hits", AdviceKind::Observation)];

    let (_, _, summary) = build_compatibility_summary(&cards, &[], 3, 5, &everything_available());

    assert!(!summary.contains("優先改善"), "断定している: {summary}");
    assert!(summary.contains("要確認"), "{summary}");
}

/// 証拠不足で黙ったカードがあれば、「改善点なし」とは言わない。
/// 言えば、直す点があるのに無いと伝えることになる。
#[test]
fn suppressed_cards_prevent_claiming_there_is_nothing_to_fix() {
    let (_, _, summary) =
        build_compatibility_summary(&[], &[suppressed("mashing")], 3, 5, &everything_available());

    assert!(
        summary.contains("改善点なしとは判定していません"),
        "{summary}"
    );
    assert!(summary.contains('1'), "件数を出していない: {summary}");
}

/// 読み取り自体が足りていなければ、やはり「改善点なし」とは言わない。
#[test]
fn poor_coverage_also_prevents_claiming_there_is_nothing_to_fix() {
    let mut coverage = everything_available();
    if let Some(availability) = coverage.availability.as_mut() {
        availability.own_input = EvidenceAvailability::Unavailable;
    }

    let (_, _, summary) = build_compatibility_summary(&[], &[], 3, 5, &coverage);

    assert!(
        summary.contains("改善点なしとは判定していません"),
        "{summary}"
    );
}

/// 全部読めていて、それでも何も出なければ、初めて「無かった」と言える。
#[test]
fn only_full_coverage_supports_saying_nothing_was_found() {
    let (_, _, summary) = build_compatibility_summary(&[], &[], 3, 5, &everything_available());

    assert!(summary.contains("検出されませんでした"), "{summary}");
    assert!(
        !summary.contains("改善点なしとは判定していません"),
        "{summary}"
    );
}

/// カードが出ていても、黙った候補があることは添える。
#[test]
fn suppressed_cards_are_mentioned_alongside_the_ones_that_appeared() {
    let cards = vec![card("mashing", AdviceKind::Diagnosis)];

    let (_, _, summary) = build_compatibility_summary(
        &cards,
        &[suppressed("anti_air")],
        3,
        5,
        &everything_available(),
    );

    assert!(summary.contains("優先改善"), "{summary}");
    assert!(
        summary.contains("証拠不足"),
        "黙った候補を伏せている: {summary}"
    );
}

/// 何を数えたのかを冒頭に置く。ラウンド数と被弾件数。
#[test]
fn the_summary_says_what_was_counted() {
    let (_, _, summary) = build_compatibility_summary(&[], &[], 3, 7, &everything_available());

    assert!(summary.contains("3ラウンド"), "{summary}");
    assert!(summary.contains("被弾 7 件"), "{summary}");
}

/// HP が読めていなければ、被弾件数は出さない。読めていない数を
/// 出すと、その数を信じられてしまう。
#[test]
fn the_damage_count_is_withheld_when_the_health_was_not_read() {
    let mut coverage = everything_available();
    if let Some(availability) = coverage.availability.as_mut() {
        availability.own_hp = EvidenceAvailability::Unavailable;
    }

    let (_, _, summary) = build_compatibility_summary(&[], &[], 3, 7, &coverage);

    assert!(
        !summary.contains("被弾 7 件"),
        "読めていない数を出している: {summary}"
    );
    assert!(summary.contains("確認不能"), "{summary}");
}

/// 弱点と練習項目は、出たカードから作る。
#[test]
fn the_weaknesses_and_practice_items_come_from_the_cards() {
    let mut with_clips = card("mashing", AdviceKind::Diagnosis);
    with_clips.evidence = vec![
        EvidenceClip {
            frame: 100,
            end_frame: None,
            label: "1".to_string(),
        },
        EvidenceClip {
            frame: 200,
            end_frame: None,
            label: "2".to_string(),
        },
    ];

    let (weaknesses, practice, _) =
        build_compatibility_summary(&[with_clips], &[], 3, 5, &everything_available());

    assert_eq!(weaknesses.len(), 1);
    assert_eq!(weaknesses[0].category, "mashing");
    assert_eq!(weaknesses[0].frequency, 2, "クリップの数を出していない");
    assert_eq!(practice, vec!["mashing の練習".to_string()]);
}
