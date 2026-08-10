//! 「守勢でボタンを押して被弾した」と言えるまでの条件に対するテスト。
//!
//! この指摘は、押したから負けたのだと因果を主張する。だから通す条件が
//! 厳しい。押した入力が直接観測できていること、その入力から技が出たと
//! メーターで裏付けられること、押した時点で守勢だったこと、そして他の
//! 指摘が既に説明している場面でないこと。
//!
//! どれか一つでも緩むと、地上戦の読み合いや対空された飛びが「暴れ」に
//! 化ける。

use super::support::*;
use crate::match_events::{
    ContactEvent, DamageEvent, DriveImpactEvent, DriveImpactOutcome, EventConfidence,
    InputEvidence, InputSegment, JumpDirection, JumpEvent, JumpOutcome, MatchEvents, MeterState,
    MinusPressEvent, MinusPressOutcome, ReversalEvent,
};

/// 押した入力。
fn press(start_frame: u32) -> InputSegment {
    InputSegment {
        start_frame,
        end_frame: start_frame + 5,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }
}

/// 大きく被弾した場面。
fn big_hit(start_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 20,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

/// 直前に削られた場面。これがあると守勢と見なされる。
fn earlier_chip(start_frame: u32) -> DamageEvent {
    DamageEvent {
        drop: 0.04,
        ..big_hit(start_frame, 0.04)
    }
}

/// 入力から技が出て、被弾の瞬間もまだ技中だった、という裏付けを置く。
fn back_the_press_with_a_move(events: &mut MatchEvents) {
    events.meter_state = [vec![MeterState::Free; 2000], vec![MeterState::Free; 2000]];
    events.meter_confidence = [vec![1.0; 2000], vec![1.0; 2000]];
    for frame in 995..=1000 {
        events.meter_state[0][frame] = MeterState::Startup;
    }
}

/// 押して被弾した一場面だけを持つ試合。
fn one_mash() -> MatchEvents {
    let mut events = empty_events();
    events.damage = vec![earlier_chip(880), big_hit(1000, 0.12)];
    events.segments[0] = vec![press(990)];
    events
}

// ── 大きさの門 ───────────────────────────────────────────────────────────

/// 小さい被弾は暴れの話にしない。連係の削りまで拾うと、指摘が
/// 埋もれて読めなくなる。
#[test]
fn a_small_hit_is_not_worth_the_card() {
    let mut events = one_mash();
    events.damage[1].drop = 0.10;
    assert!(
        detect_mashing(&[], &events, 1, 0).is_some(),
        "閾値ちょうどの被弾を落としている"
    );

    events.damage[1].drop = 0.09;
    assert!(
        detect_mashing(&[], &events, 1, 0).is_none(),
        "小さい被弾まで拾っている"
    );
}

// ── 入力の門 ─────────────────────────────────────────────────────────────

/// 被弾から離れた入力は、その被弾の原因ではない。
#[test]
fn a_press_too_long_before_the_hit_is_not_the_cause() {
    let mut events = one_mash();
    events.segments[0] = vec![press(975)];
    assert!(
        detect_mashing(&[], &events, 1, 0).is_some(),
        "窓の内側の入力を落としている"
    );

    events.segments[0] = vec![press(974)];
    assert!(
        detect_mashing(&[], &events, 1, 0).is_none(),
        "窓の外の入力を原因にしている"
    );
}

/// 被弾より後の入力も原因ではない。被弾して硬直中に押したボタンを
/// 原因にすると、時間の向きが逆になる。
#[test]
fn a_press_after_the_hit_is_not_the_cause() {
    let mut events = one_mash();
    events.segments[0] = vec![press(1000)];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// ボタンを含まない入力は暴れではない。移動やガードの方向入力を
/// 「押した」に数えると、守れている場面まで指摘になる。
#[test]
fn a_direction_without_a_button_is_not_a_mash() {
    let mut events = one_mash();
    events.segments[0][0].badges = vec![];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 投げは打撃の暴れとは別の話。既に投げの指摘が扱っている。
#[test]
fn a_throw_input_belongs_to_the_throw_cards() {
    let mut events = one_mash();
    events.segments[0][0].throw = true;

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 補間で埋めた入力は根拠にしない。押したかどうか自体が推測なら、
/// 押したから負けたとは言えない。
#[test]
fn a_repaired_input_is_not_evidence() {
    let mut events = one_mash();
    events.segments[0][0].evidence = InputEvidence {
        observed_frames: 0,
        repaired_frames: 6,
    };

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 入力が複数あれば、被弾に最も近いものを原因にする。遠い方を選ぶと、
/// 見せるクリップが実際の場面からずれる。
#[test]
fn the_nearest_press_is_the_one_reported() {
    let mut events = one_mash();
    events.segments[0] = vec![press(980), press(995)];

    let card = detect_mashing(&[], &events, 1, 0).expect("提示される");

    assert_eq!(card.evidence[0].frame, 995, "遠い入力を原因にしている");
}

// ── 守勢の門 ─────────────────────────────────────────────────────────────

/// 攻めているときに押したボタンは暴れではない。守勢だった裏付けが
/// 無ければ、地上戦の読み合いと区別が付かない。
#[test]
fn a_press_outside_a_defensive_situation_is_not_a_mash() {
    let mut events = one_mash();
    events.damage = vec![big_hit(1000, 0.12)];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 直前に削られていれば守勢。ただし時間が空きすぎていれば別の場面。
#[test]
fn the_earlier_chip_only_counts_while_it_is_recent() {
    let mut events = one_mash();
    events.damage[0] = earlier_chip(740);
    assert!(
        detect_mashing(&[], &events, 1, 0).is_some(),
        "窓の内側の削りを見ていない"
    );

    events.damage[0] = earlier_chip(739);
    assert!(
        detect_mashing(&[], &events, 1, 0).is_none(),
        "古い削りで守勢と見なしている"
    );
}

/// 相手の被弾は自分が守勢だった裏付けにならない。
#[test]
fn the_opponents_damage_does_not_make_you_defensive() {
    let mut events = one_mash();
    events.damage[0].victim = 2;

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// メーターが読めている試合では、ガードした事実そのものを守勢の
/// 裏付けにする。削られていなくても固められてはいる。
#[test]
fn blocking_an_attack_also_counts_as_being_pressured() {
    let mut events = one_mash();
    events.damage = vec![big_hit(1000, 0.12)];
    back_the_press_with_a_move(&mut events);

    assert!(
        detect_mashing(&[], &events, 1, 0).is_none(),
        "ガードも被弾も無いのに守勢と見なしている"
    );

    events.contacts = vec![ContactEvent {
        frame: 900,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    assert!(
        detect_mashing(&[], &events, 1, 0).is_some(),
        "ガードを守勢の裏付けにしていない"
    );
}

/// 飛び道具をガードしただけでは固められていない。距離があるので、
/// 押しても投げられない。
#[test]
fn blocking_a_projectile_is_not_being_pressured() {
    let mut events = one_mash();
    events.damage = vec![big_hit(1000, 0.12)];
    back_the_press_with_a_move(&mut events);
    events.contacts = vec![ContactEvent {
        frame: 900,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: true,
        round_no: 1,
    }];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

// ── 因果の裏付け ─────────────────────────────────────────────────────────

/// 被弾した瞬間に技を出していなければ、押したボタンとは繋がらない。
#[test]
fn a_press_that_produced_no_move_is_not_the_cause() {
    let mut events = one_mash();
    events.meter_state = [vec![MeterState::Free; 2000], vec![MeterState::Free; 2000]];
    events.meter_confidence = [vec![1.0; 2000], vec![1.0; 2000]];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 入力の直後に技が始まっていることまで確かめる。被弾時に技中でも、
/// それが押したボタンから出たとは限らない。
#[test]
fn the_move_has_to_start_near_the_press() {
    let mut events = one_mash();
    events.meter_state = [vec![MeterState::Free; 2000], vec![MeterState::Free; 2000]];
    events.meter_confidence = [vec![1.0; 2000], vec![1.0; 2000]];
    for frame in 997..1001 {
        events.meter_state[0][frame] = MeterState::Active;
    }

    assert!(
        detect_mashing(&[], &events, 1, 0).is_none(),
        "入力と技を繋がずに帰属している"
    );

    events.meter_state[0][997] = MeterState::Startup;

    let card = detect_mashing(&[], &events, 1, 0).expect("入力と技を繋げる");
    assert_eq!(card.confidence, EventConfidence::High);
}

/// メーターの読みが怪しいフレームは根拠にしない。
#[test]
fn an_unreliable_meter_frame_is_not_evidence() {
    let mut events = one_mash();
    events.meter_state = [vec![MeterState::Free; 2000], vec![MeterState::Free; 2000]];
    events.meter_confidence = [vec![1.0; 2000], vec![1.0; 2000]];
    for frame in 997..1001 {
        events.meter_state[0][frame] = MeterState::Active;
    }
    events.meter_state[0][997] = MeterState::Startup;
    events.meter_confidence[0][997] = 0.49;

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// メーターが読めていない試合でも指摘は出すが、確度は下げる。
#[test]
fn without_a_meter_the_card_lowers_its_confidence() {
    let card = detect_mashing(&[], &one_mash(), 1, 0).expect("提示される");

    assert_eq!(
        card.confidence,
        EventConfidence::Medium,
        "裏付けが無いのに確度を下げていない"
    );
}

// ── 読み合いとして成立していた場面 ───────────────────────────────────────

/// 相手の技の硬直に押したのなら、それは暴れではなく差し返し。
#[test]
fn pressing_into_the_opponents_recovery_is_counterplay() {
    let mut events = one_mash();
    back_the_press_with_a_move(&mut events);
    events.meter_state[1][990] = MeterState::Recovery;

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 押した先が飛び道具なら、固めを抜けるための行動。
#[test]
fn throwing_a_projectile_is_not_mashing() {
    let mut events = one_mash();
    back_the_press_with_a_move(&mut events);
    events.meter_state[0][1000] = MeterState::ProjectileActive;

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 相手が無敵技で切り返してきた場面は、こちらの暴れの話ではない。
#[test]
fn losing_to_an_invincible_reversal_is_not_mashing() {
    let mut events = one_mash();
    back_the_press_with_a_move(&mut events);
    events.meter_state[1][990] = MeterState::Invincible;

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

// ── 他の指摘が既に説明している場面 ───────────────────────────────────────

/// 対空された飛びは飛びの指摘が扱う。
#[test]
fn a_stopped_jump_belongs_to_the_jump_card() {
    let mut events = one_mash();
    events.jumps = vec![JumpEvent {
        side: 1,
        frame: 980,
        outcome: JumpOutcome::GotHit,
        input_dir: "UR".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(1000),
        takeoff_confirmed: true,
        air_end: 1030,
        round_no: 1,
    }];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 狩られた無敵技は切り返しの指摘が扱う。
#[test]
fn a_punished_reversal_belongs_to_the_reversal_card() {
    let mut events = one_mash();
    events.reversals = vec![ReversalEvent {
        side: 1,
        frame: 990,
        drop: 0.12,
        blocked: true,
        confidence: EventConfidence::High,
        round_no: 1,
    }];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// 不利フレーム後の暴れは、その専用の指摘が扱う。同じ被弾を二枚の
/// カードで指摘すると、同じ話を二度読まされる。
#[test]
fn a_press_while_minus_belongs_to_its_own_card() {
    let mut events = one_mash();
    events.presses_while_minus = vec![MinusPressEvent {
        side: 1,
        frame: 990,
        minus_frames: 5,
        pressed: "弱".to_string(),
        action_kind: Default::default(),
        outcome: MinusPressOutcome::CounterHit,
        drop: 0.12,
        confidence: EventConfidence::High,
        source_contact_frame: 980,
        round_no: 1,
    }];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

/// ドライブインパクトで返された場面も、その指摘が扱う。
#[test]
fn a_countered_drive_impact_belongs_to_its_own_card() {
    let mut events = one_mash();
    events.drive_impacts = vec![DriveImpactEvent {
        side: 1,
        input_frame: 980,
        active_frame: Some(1000),
        contact_frame: Some(1000),
        outcome: DriveImpactOutcome::Countered,
        damage: 0.12,
        confidence: EventConfidence::High,
        round_no: 1,
    }];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}

// ── カードの中身 ─────────────────────────────────────────────────────────

/// 一度きりは事実確認、繰り返しは診断。文面も書き分ける。
#[test]
fn the_wording_changes_when_it_repeats() {
    let once = one_mash();
    let mut twice = one_mash();
    twice.damage.push(big_hit(1200, 0.12));
    twice.segments[0].push(press(1190));

    let once = detect_mashing(&[], &once, 1, 0).expect("提示される");
    let twice = detect_mashing(&[], &twice, 1, 0).expect("提示される");

    assert_eq!(once.kind, AdviceKind::Observation);
    assert_eq!(twice.kind, AdviceKind::Diagnosis);
    assert_eq!(once.id, twice.id);
    assert_ne!(once.title, twice.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, twice.description,
        "説明を書き分けていない"
    );
    assert_ne!(once.practice, twice.practice, "練習方法を書き分けていない");
}

/// 最も多く押していた入力を出す。何を押す癖なのかが分からないと
/// 直せない。
#[test]
fn the_most_common_input_is_named() {
    let mut events = one_mash();
    events.damage.push(big_hit(1200, 0.12));
    events.damage.push(big_hit(1400, 0.12));
    events.segments[0] = vec![press(990), press(1190), press(1390)];
    events.segments[0][0].badges = vec!["中".to_string()];
    events.segments[0][1].badges = vec!["強".to_string()];
    events.segments[0][2].badges = vec!["強".to_string()];

    let card = detect_mashing(&[], &events, 1, 0).expect("提示される");

    assert!(
        card.description.contains('強'),
        "多かった入力を出していない: {}",
        card.description
    );
}

/// バッジの無い入力にも名前を付ける。空欄のクリップは何も伝えない。
#[test]
fn an_input_without_a_badge_still_gets_a_name() {
    let mut events = one_mash();
    events.segments[0][0].badges = vec![];
    events.segments[0][0].dir = "N".to_string();
    // ボタンを含む扱いにするため、自動入力として記録された場面にする。
    events.segments[0][0].auto = true;
    events.segments[0][0].badges = vec!["AUTO".to_string()];

    let card = detect_mashing(&[], &events, 1, 0).expect("提示される");

    assert!(!card.evidence[0].label.is_empty());
    assert!(
        card.evidence[0].label.contains("AUTO"),
        "入力の名前が出ていない: {}",
        card.evidence[0].label
    );
}

/// クリップは入力から被弾の終わりまで。手前を切ると、何を押したのかが
/// 映らない。
#[test]
fn the_clip_runs_from_the_press_to_the_end_of_the_hit() {
    let card = detect_mashing(&[], &one_mash(), 1, 0).expect("提示される");

    assert_eq!(card.evidence[0].frame, 990);
    assert_eq!(card.evidence[0].end_frame, Some(1020));
}

/// 失った HP の合計が重みになる。
#[test]
fn the_weight_is_the_health_lost() {
    let mut events = one_mash();
    events.damage.push(big_hit(1200, 0.20));
    events.segments[0].push(press(1190));

    let card = detect_mashing(&[], &events, 1, 0).expect("提示される");

    assert!((card.severity - 0.32).abs() < 1e-6);
    assert!((card.hp_lost.expect("損失がある") - 0.32).abs() < 1e-6);
}

/// 入力の記録が無ければ何も出さない。
#[test]
fn without_any_input_record_nothing_is_reported() {
    let mut events = one_mash();
    events.segments[0] = vec![];

    assert!(detect_mashing(&[], &events, 1, 0).is_none());
}
