//! カードごとに要る証拠に対するテスト。
//!
//! 読み取れなかった情報から作ったカードは、当てずっぽうと変わらない。
//! だからカードごとに、何が読めていなければ出さないのかを決めてある。
//!
//! この表が緩むと、入力欄が一度も読めていない動画から「入力の癖」が
//! 出る。厳しすぎると、読めている情報から言えることまで黙る。

use super::*;

/// 何もかも読めている状態。
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

/// 一種類だけ読めていない状態。
fn without(requirement: EvidenceRequirement) -> AnalysisCoverage {
    let mut coverage = everything_available();
    let missing = EvidenceAvailability::Unavailable;
    if let Some(availability) = coverage.availability.as_mut() {
        match requirement {
            EvidenceRequirement::OwnInput => availability.own_input = missing,
            EvidenceRequirement::OpponentInput => availability.opponent_input = missing,
            EvidenceRequirement::OwnHp => availability.own_hp = missing,
            EvidenceRequirement::OpponentHp => availability.opponent_hp = missing,
            EvidenceRequirement::FrameMeter => availability.own_meter = missing,
            EvidenceRequirement::Contacts => availability.contacts = missing,
            EvidenceRequirement::Punishes => availability.punishes = missing,
            EvidenceRequirement::Spatial => availability.spatial = missing,
            EvidenceRequirement::OwnDrive => availability.own_drive = missing,
            EvidenceRequirement::OwnSuper => availability.own_super = missing,
            EvidenceRequirement::OwnAttackInfo => availability.own_attack_info = missing,
            EvidenceRequirement::OpponentDrive => availability.opponent_drive = missing,
            EvidenceRequirement::OpponentSuper => availability.opponent_super = missing,
            EvidenceRequirement::OpponentAttackInfo => availability.opponent_attack_info = missing,
        }
    }
    coverage
}

fn card(id: &str) -> AdviceCard {
    AdviceCard {
        id: id.to_string(),
        kind: AdviceKind::Diagnosis,
        confidence: EventConfidence::High,
        title: id.to_string(),
        severity: 0.0,
        hp_lost: None,
        description: String::new(),
        practice: String::new(),
        evidence: Vec::new(),
    }
}

/// カードとその必要な証拠の対応。ここに書いた組み合わせが仕様。
fn required_evidence() -> Vec<(&'static str, Vec<EvidenceRequirement>)> {
    use EvidenceRequirement::*;
    vec![
        ("anti_air", vec![OpponentInput, OwnHp, OpponentHp]),
        ("own_jumps", vec![OwnInput, OwnHp, OpponentHp]),
        ("throw_loop", vec![OpponentInput, OwnHp]),
        (
            "committed_button_vs_di",
            vec![OwnInput, OpponentInput, FrameMeter, OwnHp],
        ),
        ("mashing", vec![OwnInput, FrameMeter, OwnHp]),
        ("press_while_minus", vec![OwnInput, FrameMeter, OwnHp]),
        ("throw_while_minus", vec![OwnInput, FrameMeter, OwnHp]),
        ("advantage_abandoned", vec![OwnInput, FrameMeter, OwnHp]),
        (
            "throw_interrupted_by_invincible",
            vec![OwnInput, FrameMeter, OwnHp],
        ),
        ("throw_whiff_punished", vec![OwnInput, FrameMeter, OwnHp]),
        ("guard_break", vec![OwnInput, Contacts, OwnHp]),
        ("whiff_punished", vec![FrameMeter, Contacts, OwnHp]),
        ("reversal_punished", vec![Punishes, OwnHp]),
        ("low_conversion", vec![Punishes, OpponentHp]),
        ("punish_fail", vec![Punishes, Spatial, OwnHp]),
        (
            "teleport_defense",
            vec![OpponentInput, FrameMeter, Spatial, OwnHp],
        ),
        ("punish_missed", vec![Punishes, Spatial, OwnHp]),
        ("layered_defense", vec![OpponentInput, Contacts, OwnHp]),
        ("burnout", vec![OwnDrive, OwnHp, OpponentHp]),
        (
            "low_scaling_super",
            vec![OwnSuper, Contacts, OwnAttackInfo, OpponentHp],
        ),
        ("early_hits", vec![OwnHp]),
        ("big_hits", vec![OwnHp]),
        ("lead_loss", vec![OwnHp, OpponentHp]),
    ]
}

/// 何もかも読めていれば、どのカードも黙らない。
#[test]
fn nothing_is_suppressed_when_everything_was_read() {
    let coverage = everything_available();

    for (id, _) in required_evidence() {
        assert_eq!(
            card_missing_requirements(&card(id), &coverage),
            Vec::new(),
            "{id} が読めている情報を要求している"
        );
    }
}

/// 必要な情報が一つでも欠ければ、そのカードは黙る。
#[test]
fn every_listed_requirement_actually_suppresses_its_card() {
    for (id, requirements) in required_evidence() {
        for requirement in requirements {
            let missing = card_missing_requirements(&card(id), &without(requirement));

            assert!(
                missing.contains(&requirement),
                "{id} が {requirement:?} 無しでも出ている"
            );
        }
    }
}

/// 表に書いていない情報は、そのカードを黙らせない。厳しすぎると、
/// 読めている情報から言えることまで出なくなる。
#[test]
fn an_unrelated_gap_does_not_suppress_a_card() {
    use EvidenceRequirement::*;
    let every_requirement = [
        OwnInput,
        OpponentInput,
        OwnHp,
        OpponentHp,
        FrameMeter,
        Contacts,
        Punishes,
        Spatial,
        OwnDrive,
        OwnSuper,
        OwnAttackInfo,
        OpponentDrive,
        OpponentSuper,
        OpponentAttackInfo,
    ];

    for (id, requirements) in required_evidence() {
        for requirement in every_requirement {
            if requirements.contains(&requirement) {
                continue;
            }
            let missing = card_missing_requirements(&card(id), &without(requirement));

            assert!(
                !missing.contains(&requirement),
                "{id} が要らないはずの {requirement:?} を要求している"
            );
        }
    }
}

/// 知らないカードは何も要求しない。新しいカードを足したときに、
/// 黙って全部消えるより出た方がよい。
#[test]
fn a_card_without_an_entry_requires_nothing() {
    assert_eq!(
        card_missing_requirements(&card("something_new"), &without(EvidenceRequirement::OwnHp)),
        Vec::new()
    );
}

/// 二つ以上欠けていれば、両方を挙げる。何を直せば出るのかが分かる。
#[test]
fn every_missing_requirement_is_listed() {
    let mut coverage = everything_available();
    if let Some(availability) = coverage.availability.as_mut() {
        availability.own_input = EvidenceAvailability::Unavailable;
        availability.own_hp = EvidenceAvailability::Unavailable;
    }

    let missing = card_missing_requirements(&card("mashing"), &coverage);

    assert!(missing.contains(&EvidenceRequirement::OwnInput));
    assert!(missing.contains(&EvidenceRequirement::OwnHp));
}

// ── 並べる順序 ───────────────────────────────────────────────────────────

fn sorted(cards: Vec<AdviceCard>) -> Vec<String> {
    let mut cards = cards;
    sort_cards(&mut cards);
    cards.into_iter().map(|card| card.id).collect()
}

/// 診断が先、次に事実確認、最後に統計。読む人が最初に見るのは、
/// 直せることであってほしい。
#[test]
fn diagnoses_come_before_observations_and_statistics() {
    let mut statistic = card("statistic");
    statistic.kind = AdviceKind::Statistic;
    let mut observation = card("observation");
    observation.kind = AdviceKind::Observation;

    let order = sorted(vec![statistic, observation, card("diagnosis")]);

    assert_eq!(order, vec!["diagnosis", "observation", "statistic"]);
}

/// 同じ種類なら、確からしい方が先。
#[test]
fn a_more_certain_card_comes_first() {
    let mut unsure = card("unsure");
    unsure.confidence = EventConfidence::Medium;

    assert_eq!(sorted(vec![unsure, card("sure")]), vec!["sure", "unsure"]);
}

/// 確度も同じなら、重い方が先。
#[test]
fn a_heavier_card_comes_first() {
    let mut light = card("light");
    light.severity = 0.1;
    let mut heavy = card("heavy");
    heavy.severity = 0.5;

    assert_eq!(sorted(vec![light, heavy]), vec!["heavy", "light"]);
}

/// 全部同じなら id 順。並びが動画ごとに変わらないようにする。
#[test]
fn otherwise_the_order_is_stable_by_id() {
    assert_eq!(sorted(vec![card("b"), card("a")]), vec!["a", "b"]);
}

// ── 古い形式の観測列 ─────────────────────────────────────────────────────
//
// 読み取り状況をまとめた欄が無い、古い形式の記録もある。その場合は
// フレーム数の割合から同じ判断をする。ここを間違えると、古い記録から
// 出るカードだけが違う基準になる。

/// 読み取り状況の欄を持たない観測列。フレーム数だけで判断する。
fn legacy_coverage(observed: u32, total: u32) -> AnalysisCoverage {
    AnalysisCoverage {
        availability: None,
        detector_match_frames: total,
        own_hp_reliable_frames: observed,
        opponent_hp_reliable_frames: observed,
        own_drive_reliable_frames: observed,
        own_super_reliable_frames: observed,
        own_input_observed_frames: observed,
        opponent_input_observed_frames: observed,
        own_meter_mapped_frames: observed,
        opponent_meter_mapped_frames: observed,
        spatial_candidate_frames: total,
        spatial_sampled_frames: observed,
        spatial_usable_frames: observed,
        attack_damage_events: total,
        attack_damage_linked: observed,
        ..AnalysisCoverage::default()
    }
}

/// 十分な割合が読めていれば、古い記録でもカードは出る。
#[test]
fn a_legacy_record_with_enough_coverage_lets_the_cards_through() {
    let coverage = legacy_coverage(60, 100);

    for (id, _) in required_evidence() {
        assert_eq!(
            card_missing_requirements(&card(id), &coverage),
            Vec::new(),
            "{id} が十分な割合を足りないと言っている"
        );
    }
}

/// 割合が足りなければ黙る。境目はちょうどで分かれる。
#[test]
fn the_legacy_coverage_threshold_has_an_exact_edge() {
    assert_eq!(
        card_missing_requirements(&card("early_hits"), &legacy_coverage(60, 100)),
        Vec::new(),
        "ちょうどの割合を足りないと言っている"
    );
    assert_eq!(
        card_missing_requirements(&card("early_hits"), &legacy_coverage(59, 100)),
        vec![EvidenceRequirement::OwnHp],
        "足りない割合を通している"
    );
}

/// 母数が 0 なら何も読めていない。割り算の前に止める。
#[test]
fn a_legacy_record_with_no_frames_at_all_suppresses_everything() {
    let missing = card_missing_requirements(&card("early_hits"), &legacy_coverage(0, 0));

    assert_eq!(missing, vec![EvidenceRequirement::OwnHp]);
}

/// SA ゲージは他より緩い基準で見る。演出中は読めないので、同じ基準では
/// 常に足りなくなる。
#[test]
fn the_super_gauge_is_held_to_a_looser_standard() {
    let mut coverage = legacy_coverage(60, 100);
    coverage.own_super_reliable_frames = 20;
    assert_eq!(
        card_missing_requirements(&card("low_scaling_super"), &coverage),
        Vec::new(),
        "SA ゲージに厳しい基準を当てている"
    );

    coverage.own_super_reliable_frames = 19;
    assert_eq!(
        card_missing_requirements(&card("low_scaling_super"), &coverage),
        vec![EvidenceRequirement::OwnSuper]
    );
}

/// 空間解析の分母は試合全体ではなく、意味のある区間だけ。全体を分母に
/// すると、必要な区間が読めていても足りないことになる。
#[test]
fn the_spatial_coverage_is_measured_against_its_own_candidates() {
    let mut coverage = legacy_coverage(60, 100);
    coverage.spatial_candidate_frames = 10;
    coverage.spatial_sampled_frames = 6;
    coverage.spatial_usable_frames = 2;

    assert_eq!(
        card_missing_requirements(&card("punish_missed"), &coverage),
        Vec::new(),
        "空間解析の分母を取り違えている"
    );
}

/// 空間解析は、区間を見に行けたことと、実際に使えたことの両方が要る。
#[test]
fn the_spatial_evidence_needs_both_sampling_and_usable_frames() {
    let mut unsampled = legacy_coverage(60, 100);
    unsampled.spatial_sampled_frames = 5;
    assert_eq!(
        card_missing_requirements(&card("punish_missed"), &unsampled),
        vec![EvidenceRequirement::Spatial],
        "見に行けていない区間を通している"
    );

    let mut unusable = legacy_coverage(60, 100);
    unusable.spatial_usable_frames = 19;
    assert_eq!(
        card_missing_requirements(&card("punish_missed"), &unusable),
        vec![EvidenceRequirement::Spatial],
        "使えなかった観測を通している"
    );
}

/// メーターは二人とも読めていなければ使えない。片側だけでは、
/// 有利不利が決まらない。
#[test]
fn the_frame_meter_needs_both_sides() {
    let mut coverage = legacy_coverage(60, 100);
    coverage.opponent_meter_mapped_frames = 10;

    assert!(
        card_missing_requirements(&card("mashing"), &coverage)
            .contains(&EvidenceRequirement::FrameMeter),
        "片側だけのメーターを通している"
    );
}

/// 接触はメーターと両者の HP から作る。どれかが欠ければ接触も使えない。
#[test]
fn contacts_depend_on_the_meter_and_both_health_bars() {
    let mut coverage = legacy_coverage(60, 100);
    coverage.opponent_hp_reliable_frames = 10;

    let missing = card_missing_requirements(&card("whiff_punished"), &coverage);

    assert!(
        missing.contains(&EvidenceRequirement::Contacts),
        "相手の HP 無しで接触を作っている"
    );
}

/// 確定反撃は接触と両者の入力から作る。
#[test]
fn punishes_depend_on_contacts_and_both_inputs() {
    let mut coverage = legacy_coverage(60, 100);
    coverage.opponent_input_observed_frames = 10;

    let missing = card_missing_requirements(&card("reversal_punished"), &coverage);

    assert!(
        missing.contains(&EvidenceRequirement::Punishes),
        "相手の入力無しで確定反撃を作っている"
    );
}

/// 攻撃表示は、被弾に結び付いた件数で見る。フレーム数ではない。
#[test]
fn the_attack_readings_are_measured_against_the_damage_events() {
    let mut coverage = legacy_coverage(60, 100);
    coverage.attack_damage_events = 10;
    coverage.attack_damage_linked = 6;
    assert_eq!(
        card_missing_requirements(&card("low_scaling_super"), &coverage),
        Vec::new()
    );

    coverage.attack_damage_linked = 5;
    assert_eq!(
        card_missing_requirements(&card("low_scaling_super"), &coverage),
        vec![EvidenceRequirement::OwnAttackInfo]
    );
}
