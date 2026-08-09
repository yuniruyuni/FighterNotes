use crate::test_support::*;

/// 有利側の入力欄が読めていない機会は分母に入れない。
/// 欠測を「攻めなかった」と数えると、放棄率が実際より高く出る。
#[test]
fn advantage_situation_needs_the_observed_input_of_the_advantaged_side() {
    let (ms, contacts, segs, rounds) = minus_press_fixture();

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(extracted.advantages.is_empty());
}

/// ガードさせた側が有利のうちに攻撃を始めなかった場面。
/// 続けて相手の攻撃を受ける側へ回った場合だけ TurnLost として記録する。
#[test]
fn advantage_situation_reports_an_abandoned_turn_as_lost() {
    let (ms, mut contacts, mut segs, rounds) = minus_press_fixture();
    // 有利側（P2）は入力欄が読めているが、ボタンを押していない。
    segs[1] = vec![idle_input(110, 130)];
    // ガードした側（P1）が硬直明けに攻撃を通し、攻守が入れ替わる。
    contacts.push(ContactEvent {
        frame: 126,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    });
    let damage = vec![DamageEvent {
        victim: 2,
        start_frame: 126,
        pre_freeze_frame: 126,
        end_frame: 140,
        hp_before: 1.0,
        hp_after: 0.88,
        drop: 0.12,
        round_no: 1,
    }];

    let extracted = extract_minus_all(&ms, &contacts, &damage, &segs, &rounds);

    assert_eq!(extracted.advantages.len(), 1);
    let advantage = &extracted.advantages[0];
    assert_eq!(advantage.side, 2);
    assert_eq!(advantage.frame, 115);
    assert_eq!(advantage.plus_frames, 5);
    assert_eq!(advantage.action_frame, None);
    assert_eq!(advantage.follow_up, None);
    assert_eq!(advantage.outcome, AdvantageOutcome::TurnLost);
    assert!((advantage.drop - 0.12).abs() < 1e-6);
}

/// 有利のうちに発生が始まっていれば、攻めを継続したものとして扱う。
#[test]
fn advantage_situation_reports_continued_pressure() {
    let (mut ms, contacts, mut segs, rounds) = minus_press_fixture();
    // 有利側（P2）が硬直明け（f115）から次の技を出す。
    for state in ms[1].iter_mut().take(119).skip(115) {
        *state = MeterState::Startup;
    }
    segs[1] = vec![minus_press(114)];

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.advantages.len(), 1);
    let advantage = &extracted.advantages[0];
    assert_eq!(advantage.side, 2);
    assert_eq!(advantage.plus_frames, 5);
    assert_eq!(advantage.action_frame, Some(115));
    assert_eq!(advantage.follow_up, Some(PressureFollowUp::Strike));
    assert_eq!(advantage.outcome, AdvantageOutcome::Continued);
}

/// 攻めず、相手も攻めてこなかった場合は仕切り直しとして区別する。
/// 被弾を伴わない放棄を損失として数えない。
#[test]
fn advantage_situation_without_a_counter_attack_is_a_reset() {
    let (ms, contacts, mut segs, rounds) = minus_press_fixture();
    segs[1] = vec![idle_input(110, 130)];

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.advantages.len(), 1);
    let advantage = &extracted.advantages[0];
    assert_eq!(advantage.outcome, AdvantageOutcome::Reset);
    assert_eq!(advantage.drop, 0.0);
}
