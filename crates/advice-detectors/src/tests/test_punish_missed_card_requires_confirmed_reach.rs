use super::support::*;

#[test]
fn test_punish_missed_card_requires_confirmed_reach() {
    let mut ev = empty_events();
    ev.punishes.push(PunishChance {
        frame: 200,
        side: 1,
        advantage: 4,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 196,
        recovery_end_frame: 203,
        source_contact_frame: Some(195),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    });

    assert!(detect_punish_missed(&ev, 1, Some("BLANKA"), None).is_none());
    ev.punishes[0].reachability = PunishReachability::Confirmed;
    let card = detect_punish_missed(&ev, 1, Some("BLANKA"), None)
        .expect("近距離を確認できた候補だけ指摘する");
    assert_eq!(card.id, "punish_missed");
    assert!(card.evidence[0].label.contains("近距離確認"));

    ev.punishes[0].reachability = PunishReachability::OutOfRange;
    assert!(detect_punish_missed(&ev, 1, Some("BLANKA"), None).is_none());
}

/// 距離を確認できない見逃し候補は、単発では出さず、繰り返したときだけ
/// 断定しない観察としてシーンを示す。距離を確認できた見逃しがあるなら
/// 従来の診断が優先される。
#[test]
fn unconfirmed_misses_become_an_observation_only_when_repeated() {
    let unknown_missed = |frame: u32, advantage: u32| PunishChance {
        frame,
        side: 1,
        advantage,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: frame.saturating_sub(4),
        recovery_end_frame: frame + 3,
        source_contact_frame: Some(frame.saturating_sub(5)),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    };

    // 1 回だけでは出さない(先端ガードの可能性が高いまま)。
    let mut ev = empty_events();
    ev.punishes.push(unknown_missed(200, 8));
    assert!(detect_punish_missed(&ev, 1, Some("LUKE"), None).is_none());

    // 2 回で観察に昇格。断定はしない。
    ev.punishes.push(unknown_missed(600, 6));
    let card = detect_punish_missed(&ev, 1, Some("LUKE"), None).expect("観察として提示する");
    assert_eq!(card.id, "punish_missed");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.confidence, EventConfidence::Medium);
    assert_eq!(card.title, "確定反撃の猶予を見逃した可能性");
    assert!(
        (card.severity - 0.04).abs() < 1e-6,
        "重みは件数に比例する: {}",
        card.severity
    );
    assert!(
        card.description
            .contains("攻撃していない場面が 2 回あります"),
        "件数が本文に無い: {}",
        card.description
    );
    assert!(
        card.description.contains("距離までは確認できていない"),
        "断定を避ける文言が無い: {}",
        card.description
    );
    // 回答候補は最小の実測有利(+6F)を物差しにする。
    assert!(
        card.description.contains("有利 6F"),
        "最小猶予で候補を出していない: {}",
        card.description
    );
    assert!(
        card.practice.contains("クリップで距離を確認し"),
        "観察の練習文が無い: {}",
        card.practice
    );
    assert_eq!(card.evidence.len(), 2);
    assert!(card.evidence[0]
        .label
        .contains("確反猶予 +8F を見逃した可能性(距離未確認)"));

    // OutOfRange(届かないと確認済み)は数えない。
    ev.punishes[1].reachability = PunishReachability::OutOfRange;
    assert!(detect_punish_missed(&ev, 1, Some("LUKE"), None).is_none());
    ev.punishes[1].reachability = PunishReachability::Unknown;

    // 距離を確認できた見逃しが 1 つでもあれば、従来の診断だけを出す。
    let mut confirmed = unknown_missed(900, 7);
    confirmed.reachability = PunishReachability::Confirmed;
    ev.punishes.push(confirmed);
    let card = detect_punish_missed(&ev, 1, Some("LUKE"), None).expect("診断を出す");
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert_eq!(card.evidence.len(), 1, "診断の証拠は確認済みの場面だけ");
}

/// ガードした相手の技を、入力表示と実測発生から同定できた場面は技名で
/// 指摘する。同じ技の反復は本文で名指しし、同定できない場面は従来の
/// 実測表現に留める。
#[test]
fn identified_moves_are_named_and_repeats_are_called_out() {
    use crate::match_events::{InputSegment, MeterState};

    let mut ev = empty_events();
    let punish = |frame: u32, contact: u32| PunishChance {
        frame,
        side: 1,
        advantage: 8,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: contact + 2,
        recovery_end_frame: frame + 3,
        source_contact_frame: Some(contact),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    };
    // 同じ技(相手ケンのしゃがみ中K、発生 7)を 2 回ガードして見逃し、
    // 3 回目は入力を観測できず同定できない。
    ev.punishes.push(punish(210, 200));
    ev.punishes.push(punish(410, 400));
    ev.punishes.push(punish(610, 600));
    ev.meter_state[1] = vec![MeterState::Free; 700];
    for contact in [200usize, 400] {
        for frame in contact - 8..contact - 1 {
            ev.meter_state[1][frame] = MeterState::Startup;
        }
        for frame in contact - 1..=contact {
            ev.meter_state[1][frame] = MeterState::Active;
        }
    }
    ev.meter_confidence[1] = vec![1.0; 700];
    let press = |start: u32| InputSegment {
        start_frame: start,
        end_frame: start + 4,
        dir: "D".to_string(),
        badges: vec!["中K".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    ev.segments[1] = vec![press(192), press(392)];

    let card = detect_punish_missed(&ev, 1, Some("LUKE"), Some("KEN")).expect("提示される");
    assert!(
        card.description
            .contains("特に KEN の 2MK は 2 回ガードして、いずれも反撃していません"),
        "反復した技を相手キャラ名で名指ししていない: {}",
        card.description
    );
    assert!(
        card.practice.starts_with("KEN がよく振る技のうち、"),
        "練習文が相手キャラ名で始まっていない: {}",
        card.practice
    );
    assert!(card.evidence[0]
        .label
        .contains("相手の 2MK に確反見逃し +8F"));
    assert!(card.evidence[1].label.contains("相手の 2MK"));
    assert!(
        card.evidence[2].label.contains("近距離確認"),
        "同定できない場面は従来表現に留める: {}",
        card.evidence[2].label
    );

    // 1 回だけなら名指しの反復注記は付けない(クリップには技名が付く)。
    ev.punishes.truncate(1);
    let single = detect_punish_missed(&ev, 1, Some("LUKE"), Some("KEN")).expect("提示される");
    assert!(!single.description.contains("特に KEN の"));
    assert!(!single.description.contains("特に相手の"));
    assert!(single.evidence[0].label.contains("相手の 2MK"));

    // 相手キャラが未指定なら同定せず、練習文も従来表現に留める。
    let unknown = detect_punish_missed(&ev, 1, Some("LUKE"), None).expect("提示される");
    assert!(unknown.evidence[0].label.contains("近距離確認"));
    assert!(
        unknown.practice.starts_with("対戦相手がよく振る技のうち、"),
        "未指定時の練習文が従来表現でない: {}",
        unknown.practice
    );
}
