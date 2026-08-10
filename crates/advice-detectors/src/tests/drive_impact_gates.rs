//! 通常技の実行中に相手の DI を受けた場面に対するテスト。
//!
//! DI に取られたこと一般ではなく、「技を出している最中に取られた」場面
//! だけを扱う。技を置いた距離とタイミングの話であって、DI への反応が
//! 遅いという話ではないため。
//!
//! それを言うには、入力表示・技の発生・相手 DI のヒット・HP の低下が
//! 全部揃っている必要がある。どれかを緩めると、DI を見てからガードした
//! 場面まで「技を置いていた」ことになる。

use super::support::*;
use crate::match_events::{
    DamageEvent, DriveImpactEvent, DriveImpactOutcome, EventConfidence, InputEvidence,
    InputSegment, MatchEvents, MeterState,
};

/// 通常技の入力。
fn button(start_frame: u32, badge: &str, dir: &str) -> InputSegment {
    InputSegment {
        start_frame,
        end_frame: start_frame + 4,
        dir: dir.to_string(),
        badges: vec![badge.to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }
}

/// 相手が当てた DI。
fn opponent_di(input_frame: u32, contact_frame: u32, damage: f32) -> DriveImpactEvent {
    DriveImpactEvent {
        side: 2,
        input_frame,
        active_frame: Some(contact_frame - 3),
        contact_frame: Some(contact_frame),
        outcome: DriveImpactOutcome::Hit,
        damage,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn taken_hit(start_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 120,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

/// 技を出している最中に DI を受けた、という裏付けをメーターに置く。
fn back_the_button_with_a_move(events: &mut MatchEvents, press: u32, contact: u32) {
    let length = (contact + 200) as usize;
    events.meter_state = [
        vec![MeterState::Free; length],
        vec![MeterState::Free; length],
    ];
    events.meter_confidence = [vec![1.0; length], vec![1.0; length]];
    for frame in press..contact {
        events.meter_state[0][frame as usize] = MeterState::Startup;
    }
    events.meter_state[0][contact as usize] = MeterState::Recovery;
}

/// 技を置いたところへ DI が刺さった一場面。
fn one_catch() -> MatchEvents {
    let mut events = empty_events();
    events.damage = vec![taken_hit(1000, 0.24)];
    events.segments[0] = vec![button(990, "強K", "N")];
    events.drive_impacts = vec![opponent_di(970, 1000, 0.24)];
    back_the_button_with_a_move(&mut events, 990, 1000);
    events
}

// ── 相手の DI 側の条件 ───────────────────────────────────────────────────

/// DI が当たっていなければ話にならない。ガードされた DI は別の場面。
#[test]
fn a_di_that_did_not_land_is_not_reported() {
    let mut events = one_catch();
    events.drive_impacts[0].outcome = DriveImpactOutcome::Blocked;

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 自分が撃った DI は自分が取られた話ではない。
#[test]
fn your_own_drive_impact_is_not_this_card() {
    let mut events = one_catch();
    events.drive_impacts[0].side = 1;

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 読み取りが怪しい DI は使わない。
#[test]
fn an_unconfirmed_drive_impact_is_not_used() {
    let mut events = one_catch();
    events.drive_impacts[0].confidence = EventConfidence::Medium;

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 接触の時刻が取れていない DI も使えない。技と DI の前後関係が
/// 決まらないため。
#[test]
fn a_drive_impact_without_a_contact_frame_cannot_be_placed() {
    let mut events = one_catch();
    events.drive_impacts[0].contact_frame = None;

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// HP が減っていない DI は、取られた場面ではない。
#[test]
fn a_drive_impact_that_cost_nothing_is_not_reported() {
    let mut events = one_catch();
    events.drive_impacts[0].damage = 0.0;

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

// ── 被弾との結び付け ─────────────────────────────────────────────────────

/// 接触から離れた被弾は、その DI の結果ではない。
#[test]
fn damage_far_from_the_contact_is_not_its_result() {
    let mut inside = one_catch();
    inside.damage = vec![taken_hit(1080, 0.24)];
    assert!(
        detect_committed_button_vs_di(&inside, 1, 0).is_some(),
        "窓の内側の被弾を落としている"
    );

    let mut outside = one_catch();
    outside.damage = vec![taken_hit(1081, 0.24)];
    assert!(
        detect_committed_button_vs_di(&outside, 1, 0).is_none(),
        "窓の外の被弾を結び付けている"
    );
}

/// 被弾が複数あれば、接触に最も近いものを選ぶ。
#[test]
fn the_hit_closest_to_the_contact_is_the_one_used() {
    let mut events = one_catch();
    events.damage = vec![taken_hit(1002, 0.24), taken_hit(1060, 0.10)];

    let card = detect_committed_button_vs_di(&events, 1, 0).expect("提示される");

    assert_eq!(card.hp_lost, Some(0.24), "遠い被弾を結び付けている");
}

/// 相手が受けた被弾は自分の損失ではない。
#[test]
fn damage_the_opponent_took_is_not_yours() {
    let mut events = one_catch();
    events.damage[0].victim = 2;

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// ラウンドが違えば別の場面。
#[test]
fn damage_in_another_round_is_not_connected() {
    let mut events = one_catch();
    events.damage[0].round_no = 2;

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

// ── 入力側の条件 ─────────────────────────────────────────────────────────

/// 技を置いていなければ、この指摘の対象ではない。DI を見てから
/// ガードしようとして間に合わなかった場面がこれに当たる。
#[test]
fn without_a_button_there_is_nothing_to_review() {
    let mut events = one_catch();
    events.segments[0] = vec![];

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 補間で埋めた入力は根拠にしない。
#[test]
fn a_repaired_input_is_not_evidence() {
    let mut events = one_catch();
    events.segments[0][0].evidence = InputEvidence {
        observed_frames: 0,
        repaired_frames: 5,
    };

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// DI の入力より前に押した技は、この DI に取られた技ではない。
#[test]
fn a_button_pressed_before_the_di_input_is_not_the_one_caught() {
    let mut events = one_catch();
    events.segments[0] = vec![button(969, "強K", "N")];
    back_the_button_with_a_move(&mut events, 969, 1000);

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 投げは通常技ではない。DI は投げに勝つので、別の読み合いになる。
#[test]
fn a_throw_is_not_a_committed_normal() {
    let mut events = one_catch();
    events.segments[0][0].throw = true;

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 自分の DI 入力も通常技ではない。DI 同士のぶつかり合いは別の話。
#[test]
fn your_own_di_input_is_not_a_committed_normal() {
    let mut events = one_catch();
    events.segments[0][0].badges = vec!["DI".to_string()];

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 同時押しは通常技として扱わない。何の技か決まらないため。
#[test]
fn a_multi_button_input_is_not_a_single_normal() {
    let mut events = one_catch();
    events.segments[0][0].badges = vec!["中P".to_string(), "中K".to_string()];

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 入力が複数あれば、接触に最も近いものを取られた技とする。
#[test]
fn the_latest_button_before_the_contact_is_the_one_caught() {
    let mut events = one_catch();
    events.segments[0] = vec![button(975, "弱P", "N"), button(990, "強K", "N")];

    let card = detect_committed_button_vs_di(&events, 1, 0).expect("提示される");

    assert!(
        card.evidence[0].label.contains("強K"),
        "遠い入力を取られた技にしている: {}",
        card.evidence[0].label
    );
}

/// ボタンを押したまま方向だけ離すと入力が分かれて記録される。
/// 押した瞬間まで戻さないと、クリップが技の出始めより後から始まる。
#[test]
fn a_held_button_is_traced_back_to_when_it_was_pressed() {
    let mut events = one_catch();
    events.segments[0] = vec![button(990, "強K", "DR"), button(995, "強K", "N")];
    events.segments[0][0].end_frame = 994;

    let card = detect_committed_button_vs_di(&events, 1, 0).expect("提示される");

    assert_eq!(
        card.evidence[0].frame, 990,
        "ボタンを押した瞬間まで戻していない"
    );
    assert!(
        card.evidence[0].label.contains('↘'),
        "押した瞬間の方向を出していない: {}",
        card.evidence[0].label
    );
}

/// 間の空いた入力は同じ押しの続きではない。繋ぐと、別の技の入力まで
/// 遡ってクリップが長くなる。
#[test]
fn a_gap_means_it_was_a_separate_press() {
    let mut events = one_catch();
    events.segments[0] = vec![button(980, "強K", "DR"), button(995, "強K", "N")];
    events.segments[0][0].end_frame = 984;

    let card = detect_committed_button_vs_di(&events, 1, 0).expect("提示される");

    assert_eq!(card.evidence[0].frame, 995, "離れた入力まで遡っている");
}

// ── 技が出ていた裏付け ───────────────────────────────────────────────────

/// メーターが読めていなければ、技を出していたとは言えない。
#[test]
fn without_a_meter_the_move_cannot_be_confirmed() {
    let mut events = one_catch();
    events.meter_state = [vec![], vec![]];
    events.meter_confidence = [vec![], vec![]];

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// 入力の直後に技が始まっていなければ、その入力から出た技ではない。
#[test]
fn the_move_has_to_start_near_the_input() {
    let mut events = one_catch();
    for frame in 990..1000 {
        events.meter_state[0][frame] = MeterState::Active;
    }

    assert!(
        detect_committed_button_vs_di(&events, 1, 0).is_none(),
        "入力と技を繋がずに帰属している"
    );
}

/// DI を受けた瞬間に技中でなければ、技を置いていたとは言えない。
/// 技が終わった後で取られたのなら、それは反応の話。
#[test]
fn the_move_has_to_still_be_running_at_the_contact() {
    let mut events = one_catch();
    for frame in 995..1001 {
        events.meter_state[0][frame] = MeterState::Free;
    }

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

/// メーターの読みが怪しいフレームは根拠にしない。
#[test]
fn an_unreliable_meter_frame_is_not_evidence() {
    let mut events = one_catch();
    for frame in 990..=1000 {
        events.meter_confidence[0][frame] = 0.49;
    }

    assert!(detect_committed_button_vs_di(&events, 1, 0).is_none());
}

// ── カードの中身 ─────────────────────────────────────────────────────────

/// 一度きりは事実確認。繰り返して初めて診断にする。
#[test]
fn the_wording_changes_when_it_repeats() {
    let once = one_catch();
    let mut twice = one_catch();
    twice.damage.push(taken_hit(2000, 0.20));
    twice.segments[0].push(button(1990, "強K", "N"));
    twice.drive_impacts.push(opponent_di(1970, 2000, 0.20));
    let length = 2400;
    twice.meter_state = [
        vec![MeterState::Free; length],
        vec![MeterState::Free; length],
    ];
    twice.meter_confidence = [vec![1.0; length], vec![1.0; length]];
    for frame in [990..1000, 1990..2000] {
        for index in frame {
            twice.meter_state[0][index] = MeterState::Startup;
        }
    }
    twice.meter_state[0][1000] = MeterState::Recovery;
    twice.meter_state[0][2000] = MeterState::Recovery;

    let once = detect_committed_button_vs_di(&once, 1, 0).expect("提示される");
    let twice = detect_committed_button_vs_di(&twice, 1, 0).expect("提示される");

    assert_eq!(once.kind, AdviceKind::Observation);
    assert_eq!(twice.kind, AdviceKind::Diagnosis);
    assert_eq!(once.id, twice.id);
    assert_ne!(once.title, twice.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, twice.description,
        "説明を書き分けていない"
    );
    assert!((twice.hp_lost.expect("損失がある") - 0.44).abs() < 1e-6);
}

/// どちらの場合も、断定はしない。技が先だったのか DI が先だったのかは
/// この時系列からは決められない。
#[test]
fn the_card_never_claims_who_moved_first() {
    let card = detect_committed_button_vs_di(&one_catch(), 1, 0).expect("提示される");

    assert!(
        card.description.contains("断定"),
        "決められないことを決めている: {}",
        card.description
    );
}

/// クリップは入力から被弾の終わりまで。
#[test]
fn the_clip_runs_from_the_input_to_the_end_of_the_hit() {
    let card = detect_committed_button_vs_di(&one_catch(), 1, 0).expect("提示される");

    assert_eq!(card.evidence[0].frame, 990);
    assert_eq!(card.evidence[0].end_frame, Some(1120));
    assert!(
        card.evidence[0].label.contains("強K"),
        "取られた技を出していない: {}",
        card.evidence[0].label
    );
}

/// 方向を伴わない入力は、技名だけで出す。余計な矢印は付けない。
#[test]
fn a_neutral_input_is_labelled_without_an_arrow() {
    let card = detect_committed_button_vs_di(&one_catch(), 1, 0).expect("提示される");

    assert!(
        !card.evidence[0].label.contains('+'),
        "方向の無い入力に矢印を付けている: {}",
        card.evidence[0].label
    );
}
