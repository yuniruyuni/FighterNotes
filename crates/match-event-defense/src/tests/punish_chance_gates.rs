//! 確定反撃の機会を切り出すまでの条件に対するテスト。
//!
//! 相手のメーターに後隙が出ていることは、確反の機会があったことを意味
//! しない。遠くで技を振っただけかもしれないし、自分が攻撃をガードさせた
//! 結果の硬直かもしれない。
//!
//! だから「その後隙はこちらがガードした結果である」ことを確かめる。
//! 確かめないと、地上戦で相手が振った技のすべてが確反の機会になり、
//! 見逃しの数がいくらでも増える。

use crate::test_support::*;

/// 空の（何も起きていない）メーター列。
fn flat(length: usize) -> Vec<MeterState> {
    vec![MeterState::Free; length]
}

/// 相手が技を振り、こちらがガードして、相手に後隙が出ている観測列。
///
/// 相手: f10..20 持続、f20..40 後隙。自分: f10..20 ガード硬直。
fn blocked_then_recovery() -> (Vec<MeterState>, Vec<MeterState>, Vec<ContactEvent>) {
    let length = 120;
    let mut own = flat(length);
    let mut opponent = flat(length);
    for state in opponent.iter_mut().take(20).skip(10) {
        *state = MeterState::Active;
    }
    for state in opponent.iter_mut().take(40).skip(20) {
        *state = MeterState::Recovery;
    }
    for state in own.iter_mut().take(20).skip(10) {
        *state = MeterState::Stun;
    }
    let contacts = vec![ContactEvent {
        frame: 10,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    }];
    (own, opponent, contacts)
}

// ── 機会が成立する条件 ───────────────────────────────────────────────────

/// ガードして相手に後隙が出ていれば、反撃の機会。攻撃しなければ見逃し。
#[test]
fn a_blocked_move_leaving_recovery_is_a_chance() {
    let (own, opponent, contacts) = blocked_then_recovery();

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert_eq!(punishes.len(), 1);
    assert_eq!(punishes[0].outcome, PunishOutcome::Missed);
    assert_eq!(punishes[0].origin, PunishOrigin::BlockedMove);
    assert_eq!(punishes[0].source_contact_frame, Some(10));
}

/// ガードの記録が無い後隙は、遠くで技を振っただけかもしれない。
/// 見逃しとしては数えない。
#[test]
fn recovery_without_a_recorded_block_is_not_a_missed_chance() {
    let (own, opponent, _) = blocked_then_recovery();

    let punishes = extract_synth_punishes(0, own, opponent, vec![]);

    assert!(punishes.is_empty(), "ガードの裏付けなく見逃しを作っている");
}

/// 当たった攻撃は、ガードではない。被弾した後の相手の後隙は、
/// こちらが動けないので確反の機会ではない。
#[test]
fn a_hit_does_not_establish_a_punish_chance() {
    let (own, opponent, mut contacts) = blocked_then_recovery();
    contacts[0].hit = true;

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(punishes.is_empty());
}

/// 相手の持続が長く、ガードから後隙の開始まで間が空いた観測列。
///
/// 相手: f10..45 持続、f45..70 後隙。自分: f10..20 ガード硬直。
fn block_then_a_late_recovery() -> (Vec<MeterState>, Vec<MeterState>) {
    let length = 120;
    let mut own = flat(length);
    let mut opponent = flat(length);
    for state in opponent.iter_mut().take(45).skip(10) {
        *state = MeterState::Active;
    }
    for state in opponent.iter_mut().take(70).skip(45) {
        *state = MeterState::Recovery;
    }
    for state in own.iter_mut().take(20).skip(10) {
        *state = MeterState::Stun;
    }
    (own, opponent)
}

/// ガードが後隙から離れていれば、その後隙の原因ではない。
#[test]
fn a_block_long_before_the_recovery_is_not_its_cause() {
    let (own, opponent) = block_then_a_late_recovery();
    let block = |frame| {
        vec![ContactEvent {
            frame,
            attacker: 2,
            victim: 1,
            hit: false,
            projectile: false,
            round_no: 1,
        }]
    };

    let inside = extract_synth_punishes(0, own.clone(), opponent.clone(), block(25));
    let outside = extract_synth_punishes(0, own, opponent, block(24));

    assert_eq!(inside.len(), 1, "窓の内側のガードを落としている");
    assert!(outside.is_empty(), "離れたガードを起点にしている");
}

/// 自分が攻撃を当てた／ガードさせた結果の硬直は、確反の機会ではない。
/// 自分の連係の途中を「相手の後隙」と読むことになる。
#[test]
fn recovery_your_own_attack_caused_is_not_a_chance() {
    let (own, opponent, mut contacts) = blocked_then_recovery();
    contacts.push(ContactEvent {
        frame: 18,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    });

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(
        punishes.is_empty(),
        "自分が作った硬直を確反の機会にしている"
    );
}

/// 自分の攻撃がずっと前なら、その硬直の原因ではない。
#[test]
fn your_own_attack_long_before_does_not_explain_the_recovery() {
    let (own, opponent) = block_then_a_late_recovery();
    let contacts = vec![
        ContactEvent {
            frame: 25,
            attacker: 2,
            victim: 1,
            hit: false,
            projectile: false,
            round_no: 1,
        },
        ContactEvent {
            frame: 14,
            attacker: 1,
            victim: 2,
            hit: false,
            projectile: false,
            round_no: 1,
        },
    ];

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert_eq!(punishes.len(), 1, "古い攻撃で機会を潰している");
}

/// 有利が小さすぎれば反撃は入らない。時間の足りない場面を見逃しに
/// 数えると、直しようのない指摘が並ぶ。
#[test]
fn a_recovery_too_short_to_punish_is_not_a_chance() {
    let length = 120;
    let mut own = flat(length);
    let mut opponent = flat(length);
    for state in opponent.iter_mut().take(20).skip(10) {
        *state = MeterState::Active;
    }
    // 後隙は 3 フレームだけ。
    for state in opponent.iter_mut().take(23).skip(20) {
        *state = MeterState::Recovery;
    }
    for state in own.iter_mut().take(20).skip(10) {
        *state = MeterState::Stun;
    }
    let contacts = vec![ContactEvent {
        frame: 10,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(
        punishes.is_empty(),
        "反撃の間に合わない後隙を機会にしている"
    );
}

/// まだ硬直が明けていなければ、機会は始まらない。硬直の明けた最初の
/// フレームが起点になる。
#[test]
fn the_chance_starts_when_your_own_stun_ends() {
    let (mut own, opponent, contacts) = blocked_then_recovery();
    // ガード硬直を後隙の途中まで伸ばす。
    for state in own.iter_mut().take(30).skip(10) {
        *state = MeterState::Stun;
    }

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert_eq!(punishes.len(), 1);
    assert_eq!(punishes[0].frame, 30, "硬直中から機会が始まっている");
    assert_eq!(punishes[0].advantage, 10);
}

/// 硬直が後隙の終わりまで続いていれば、機会そのものが無い。
#[test]
fn a_stun_lasting_past_the_recovery_leaves_no_chance() {
    let (mut own, opponent, contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(45).skip(10) {
        *state = MeterState::Stun;
    }

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(punishes.is_empty());
}

/// ジャンプ中の機会は地上の確反ではない。飛びの話は飛びの指摘が扱う。
#[test]
fn a_chance_during_your_own_jump_belongs_to_the_jump_cards() {
    let (own, opponent, contacts) = blocked_then_recovery();
    let length = own.len();
    let features: Vec<_> = (0..length).map(|i| feat(i as u32, 1.0, 1.0)).collect();
    let epochs = [vec![0; length], vec![0; length]];
    let game_frames = [
        (0..length as i64).collect::<Vec<_>>(),
        (0..length as i64).collect::<Vec<_>>(),
    ];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: length as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let mut jump = idle_input(15, 20);
    jump.dir = "UR".to_string();

    let punishes = crate::punishes::extract_punishes(crate::punishes::PunishInputs {
        features: &features,
        meter_state: &[own, opponent],
        meter_epoch: &epochs,
        meter_game_frame: &game_frames,
        contacts: &contacts,
        damage: &[],
        segments: &[vec![jump], vec![]],
        rounds: &rounds,
    });

    assert!(punishes.is_empty(), "ジャンプ中の機会を地上確反にしている");
}

// ── 反撃を出した結果 ─────────────────────────────────────────────────────

/// 反撃を当てていれば成功。位置も確認できたことになる。
#[test]
fn landing_the_punish_confirms_both_the_timing_and_the_range() {
    let (mut own, opponent, mut contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }
    contacts.push(ContactEvent {
        frame: 25,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    });

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert_eq!(punishes.len(), 1);
    assert_eq!(punishes[0].outcome, PunishOutcome::Success);
    assert_eq!(punishes[0].reachability, PunishReachability::Confirmed);
    assert_eq!(punishes[0].attack_start_frame, Some(20));
    assert_eq!(punishes[0].attack_active_frame, Some(24));
}

/// ガードされたのなら届いている。距離の話ではないので、この機会は
/// 扱わない。
#[test]
fn a_punish_that_was_blocked_is_not_a_range_problem() {
    let (mut own, opponent, mut contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }
    contacts.push(ContactEvent {
        frame: 25,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    });

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(punishes.is_empty(), "ガードされた反撃を空振りにしている");
}

/// 攻撃判定が出て、どこにも触れなければ空振り。
#[test]
fn an_attack_that_touched_nothing_is_a_whiff() {
    let (mut own, opponent, contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert_eq!(punishes.len(), 1);
    assert_eq!(punishes[0].outcome, PunishOutcome::WhiffFail);
    assert_eq!(
        punishes[0].reachability,
        PunishReachability::Unknown,
        "距離を確かめずに確定させている"
    );
}

/// 攻撃判定が後隙のうちに出ていなければ、距離の問題ではなく単に
/// 間に合っていない。
#[test]
fn an_attack_whose_active_frames_came_too_late_is_not_a_range_failure() {
    let (mut own, opponent, contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(41).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(45).skip(41) {
        *state = MeterState::Active;
    }

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(punishes.is_empty(), "間に合わなかった技を空振りにしている");
}

/// 弾は生成の時刻しか読めない。着弾を確かめられない弾を「届かなかった」
/// とは断定しない。
#[test]
fn a_projectile_is_not_declared_a_failed_punish() {
    let (mut own, opponent, contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::ProjectileActive;
    }

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(punishes.is_empty());
}

/// 自分から撃った無敵技は、相手の後隙への反撃ではない。
#[test]
fn your_own_invincible_move_is_not_a_punish() {
    let (mut own, opponent, contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(25).skip(21) {
        *state = MeterState::Invincible;
    }

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(punishes.is_empty());
}

// ── 接触の記録が立たなかったガード ───────────────────────────────────────
//
// ごく短い接触ではヒットストップの条件を満たさず、接触の記録が作れない
// ことがある。そのとき、相手の攻撃判定と自分のガード硬直が同時に出て
// いれば、ガードしていたと読める。ただし補助的な証拠なので、反撃を
// 出さなかった見逃しの側には使わない。

/// 接触の記録が無くても、メーターの重なりでガードを読み取れる。
#[test]
fn a_block_can_be_read_from_the_meters_when_no_contact_was_recorded() {
    let (mut own, opponent, _) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }

    let punishes = extract_synth_punishes(0, own, opponent, vec![]);

    assert_eq!(punishes.len(), 1, "メーターのガードを読めていない");
    assert_eq!(punishes[0].outcome, PunishOutcome::WhiffFail);
    assert_eq!(punishes[0].origin, PunishOrigin::BlockedMove);
}

/// メーターの重なりだけでは、反撃を出さなかった見逃しは作らない。
/// 誤検出のとき、行動の裏付けが何も無くなる。
#[test]
fn the_meter_reading_alone_does_not_create_a_missed_chance() {
    let (own, opponent, _) = blocked_then_recovery();

    let punishes = extract_synth_punishes(0, own, opponent, vec![]);

    assert!(punishes.is_empty(), "裏付けの薄い見逃しを作っている");
}

/// 硬直の近くで HP が減っていれば、それはガードではなくコンボの途中。
#[test]
fn a_stun_with_damage_nearby_is_a_combo_not_a_block() {
    let (mut own, opponent, _) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }
    let length = own.len();
    let features: Vec<_> = (0..length).map(|i| feat(i as u32, 1.0, 1.0)).collect();
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: length as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 15,
        pre_freeze_frame: 15,
        end_frame: 19,
        hp_before: 1.0,
        hp_after: 0.9,
        drop: 0.1,
        round_no: 1,
    }];

    let punishes = crate::punishes::extract_punishes(crate::punishes::PunishInputs {
        features: &features,
        meter_state: &[own, opponent],
        meter_epoch: &[vec![0; length], vec![0; length]],
        meter_game_frame: &[
            (0..length as i64).collect::<Vec<_>>(),
            (0..length as i64).collect::<Vec<_>>(),
        ],
        contacts: &[],
        damage: &damage,
        segments: &[vec![], vec![]],
        rounds: &rounds,
    });

    assert!(punishes.is_empty(), "コンボ中の硬直をガードと読んでいる");
}

// ── 反撃の後始末 ─────────────────────────────────────────────────────────

/// 後隙が終わった直後に触れていれば、届いてはいる。空振りではない。
#[test]
fn a_contact_just_after_the_recovery_still_means_it_reached() {
    let (mut own, opponent, mut contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }
    contacts.push(ContactEvent {
        frame: 55,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    });

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert!(punishes.is_empty(), "届いた反撃を空振りにしている");
}

/// ずっと後の接触は、その反撃の結果ではない。
#[test]
fn a_contact_long_after_the_recovery_is_a_separate_exchange() {
    let (mut own, opponent, mut contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }
    contacts.push(ContactEvent {
        frame: 61,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    });

    let punishes = extract_synth_punishes(0, own, opponent, contacts);

    assert_eq!(punishes.len(), 1, "無関係な接触で機会を潰している");
    assert_eq!(punishes[0].outcome, PunishOutcome::WhiffFail);
}

/// 空振りの後に被弾していれば、その分を記録する。
#[test]
fn health_lost_after_a_whiffed_punish_is_recorded() {
    let (mut own, opponent, contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }
    let length = own.len();
    let features: Vec<_> = (0..length).map(|i| feat(i as u32, 1.0, 1.0)).collect();
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: length as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 50,
        pre_freeze_frame: 50,
        end_frame: 70,
        hp_before: 1.0,
        hp_after: 0.82,
        drop: 0.18,
        round_no: 1,
    }];

    let punishes = crate::punishes::extract_punishes(crate::punishes::PunishInputs {
        features: &features,
        meter_state: &[own, opponent],
        meter_epoch: &[vec![0; length], vec![0; length]],
        meter_game_frame: &[
            (0..length as i64).collect::<Vec<_>>(),
            (0..length as i64).collect::<Vec<_>>(),
        ],
        contacts: &contacts,
        damage: &damage,
        segments: &[vec![], vec![]],
        rounds: &rounds,
    });

    assert_eq!(punishes.len(), 1);
    assert!((punishes[0].punished_drop - 0.18).abs() < 1e-6);
}

/// 反撃に使った入力を記録する。何で取ろうとしたのかが分からないと、
/// 代わりの技を選べない。
#[test]
fn the_button_used_for_the_punish_is_recorded() {
    let (mut own, opponent, contacts) = blocked_then_recovery();
    for state in own.iter_mut().take(24).skip(20) {
        *state = MeterState::Startup;
    }
    for state in own.iter_mut().take(28).skip(24) {
        *state = MeterState::Active;
    }
    let length = own.len();
    let features: Vec<_> = (0..length).map(|i| feat(i as u32, 1.0, 1.0)).collect();
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: length as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    let mut press = idle_input(19, 23);
    press.badges = vec!["強P".to_string()];

    let punishes = crate::punishes::extract_punishes(crate::punishes::PunishInputs {
        features: &features,
        meter_state: &[own, opponent],
        meter_epoch: &[vec![0; length], vec![0; length]],
        meter_game_frame: &[
            (0..length as i64).collect::<Vec<_>>(),
            (0..length as i64).collect::<Vec<_>>(),
        ],
        contacts: &contacts,
        damage: &[],
        segments: &[vec![press], vec![]],
        rounds: &rounds,
    });

    assert_eq!(punishes.len(), 1);
    assert_eq!(punishes[0].pressed, "強P");
}

// ── ゲーム内の時間で有利を測る ───────────────────────────────────────────

/// 有利フレームはゲーム内の時間で数える。演出でメーターが止まっている
/// 間も動画のフレームは進むので、動画の差では過大になる。
#[test]
fn the_advantage_is_counted_in_game_frames() {
    let (own, opponent, contacts) = blocked_then_recovery();
    let length = own.len();
    let features: Vec<_> = (0..length).map(|i| feat(i as u32, 1.0, 1.0)).collect();
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: length as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    // 相手のメーターが f25 以降 8 フレーム止まっていた。
    let stalled: Vec<i64> = (0..length as i64)
        .map(|frame| if frame >= 25 { frame - 8 } else { frame })
        .collect();

    let punishes = crate::punishes::extract_punishes(crate::punishes::PunishInputs {
        features: &features,
        meter_state: &[own, opponent],
        meter_epoch: &[vec![0; length], vec![0; length]],
        meter_game_frame: &[(0..length as i64).collect::<Vec<_>>(), stalled],
        contacts: &contacts,
        damage: &[],
        segments: &[vec![], vec![]],
        rounds: &rounds,
    });

    assert_eq!(punishes.len(), 1);
    assert_eq!(
        punishes[0].advantage, 12,
        "動画の差をそのまま有利フレームにしている"
    );
}
