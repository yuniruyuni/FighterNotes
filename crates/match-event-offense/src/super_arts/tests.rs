//! SA/CA の使用を読み取るところに対するテスト。
//!
//! 使ったことはゲージの減りで分かる。ただしゲージの表示は揺れるので、
//! ストックの境目をまたいで十分に減ったときだけ使用と見なす。
//!
//! 難しいのは「いつ撃ったか」で、ゲージが減って見えるのは演出が終わった
//! 後になる。暗転の開始か、メーターに出た発生を遡って起点にする。起点が
//! ずれると、当たったかどうかも文脈もすべて別の場面の話になる。

use super::*;
use match_event_model::test_support::feat;

const LENGTH: usize = 600;

/// ゲージが `before` から `after` へ減る観測列。減りは f200 で起きる。
fn features_dropping(before: f32, after: f32) -> Vec<FrameFeatures> {
    (0..LENGTH)
        .map(|index| {
            let mut feature = feat(index as u32, 1.0, 1.0);
            let value = if index < 200 { before } else { after };
            feature.left_super_value = value;
            feature.left_super_uncertain = false;
            feature.right_super_uncertain = false;
            feature
        })
        .collect()
}

fn rounds() -> Vec<RoundInfo> {
    vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: LENGTH as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 0.5,
    }]
}

/// 何も無い状態から抽出する。
fn extract(features: &[FrameFeatures]) -> Vec<SuperArtEvent> {
    extract_with(features, &[Vec::new(), Vec::new()], &[], &[], &[], &[])
}

fn extract_with(
    features: &[FrameFeatures],
    meter_state: &[Vec<MeterState>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    punishes: &[PunishChance],
    freeze_spans: &[(u32, u32)],
) -> Vec<SuperArtEvent> {
    extract_super_arts(SuperArtInputs {
        features,
        meter_state,
        contacts,
        damage,
        punishes,
        rounds: &rounds(),
        freeze_spans,
    })
}

/// 相手に与えた被弾。
fn dealt(start_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 2,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 60,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

// ── 使ったことをどう読むか ───────────────────────────────────────────────

/// ストックの境目をまたいで減っていれば、使っている。
#[test]
fn a_drop_across_a_stock_boundary_is_a_use() {
    let events = extract(&features_dropping(3.0, 0.0));

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, 3);
    assert_eq!(events[0].side, 1);
}

/// 消えたストックの数がレベルになる。
#[test]
fn the_number_of_stocks_spent_is_the_level() {
    for (before, after, level) in [(3.0, 2.0, 1), (3.0, 1.0, 2), (2.0, 0.0, 2)] {
        let events = extract(&features_dropping(before, after));

        assert_eq!(events.len(), 1, "{before}→{after} を読めていない");
        assert_eq!(events[0].level, level, "{before}→{after}");
    }
}

/// ゲージが増えているのは回復。使用ではない。
#[test]
fn a_rising_gauge_is_not_a_use() {
    assert!(extract(&features_dropping(1.0, 2.0)).is_empty());
}

/// 少ししか減っていなければ、ストックの境目の表示揺れ。
#[test]
fn a_small_drop_is_just_the_display_wobbling() {
    let spent = extract(&features_dropping(2.0, 1.35));
    let wobble = extract(&features_dropping(2.0, 1.36));

    assert_eq!(spent.len(), 1, "閾値ちょうどの消費を落としている");
    assert!(wobble.is_empty(), "表示の揺れを消費にしている");
}

/// 読めなかったフレームは比較に使わない。読めない瞬間を挟むと、
/// そこで減ったように見える。
#[test]
fn an_unreadable_frame_takes_no_part() {
    let mut features = features_dropping(3.0, 3.0);
    for feature in features.iter_mut().take(210).skip(200) {
        feature.left_super_value = 0.0;
        feature.left_super_uncertain = true;
    }

    assert!(
        extract(&features).is_empty(),
        "読めない値で消費を作っている"
    );
}

/// 試合画面でないフレームも比較に使わない。
#[test]
fn a_frame_outside_the_match_takes_no_part() {
    let mut features = features_dropping(3.0, 3.0);
    for feature in features.iter_mut().take(210).skip(200) {
        feature.left_super_value = 0.0;
        feature.is_match_screen = false;
    }

    assert!(extract(&features).is_empty());
}

/// 二人とも別々に見る。
#[test]
fn the_two_sides_are_read_separately() {
    let mut features = features_dropping(3.0, 3.0);
    for feature in features.iter_mut().skip(200) {
        feature.right_super_value = 0.0;
    }
    for feature in features.iter_mut() {
        if feature.frame_index < 200 {
            feature.right_super_value = 3.0;
        }
    }

    let events = extract(&features);

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].side, 2, "側を取り違えている");
}

// ── いつ撃ったか ─────────────────────────────────────────────────────────

/// 暗転が見つかっていれば、その開始を撃った時刻にする。ゲージが減って
/// 見えるのは演出の後なので、そのままでは遅すぎる。
#[test]
fn a_freeze_span_anchors_the_moment_it_was_thrown() {
    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &[],
        &[],
        &[],
        &[(150, 195)],
    );

    assert_eq!(events[0].frame, 150, "暗転の開始を起点にしていない");
    assert_eq!(events[0].confidence, EventConfidence::High);
}

/// 離れた暗転は別の場面。
#[test]
fn a_freeze_span_far_from_the_drop_is_not_this_super() {
    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &[],
        &[],
        &[],
        &[(100, 120)],
    );

    assert_ne!(events[0].frame, 100, "無関係な暗転を起点にしている");
}

/// 暗転が無ければ、メーターに出た発生まで遡る。
#[test]
fn without_a_freeze_the_meter_start_anchors_it() {
    let mut states = vec![MeterState::Free; LENGTH];
    states[180..190].fill(MeterState::Startup);

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[states, vec![MeterState::Free; LENGTH]],
        &[],
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].frame, 180, "発生まで遡っていない");
    assert_eq!(events[0].confidence, EventConfidence::High);
}

/// 無敵の始まりの方を優先する。無敵から始まる SA では、そちらが技の
/// 本当の始まり。
#[test]
fn invincibility_is_preferred_over_startup_as_the_anchor() {
    let mut states = vec![MeterState::Free; LENGTH];
    states[170..180].fill(MeterState::Invincible);
    states[180..190].fill(MeterState::Startup);

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[states, vec![MeterState::Free; LENGTH]],
        &[],
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].frame, 170);
}

/// 何も見つからなければ、ゲージが減った時刻を起点にする。ただし
/// 遡れていないので確度は下げる。
#[test]
fn without_any_evidence_the_drop_frame_is_used_with_lower_confidence() {
    let events = extract(&features_dropping(3.0, 0.0));

    assert_eq!(events[0].frame, 200);
    assert_eq!(events[0].gauge_drop_frame, 200);
    assert_eq!(
        events[0].confidence,
        EventConfidence::Medium,
        "遡れていないのに確度を下げていない"
    );
}

// ── 撃った結果 ───────────────────────────────────────────────────────────

/// 当たっていればヒット。
#[test]
fn a_contact_that_hit_makes_it_a_hit() {
    let contacts = vec![ContactEvent {
        frame: 210,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &contacts,
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].outcome, SuperArtOutcome::Hit);
    assert_eq!(events[0].contact_frame, Some(210));
}

/// 触れたが当たっていなければガードされている。
#[test]
fn a_contact_that_did_not_hit_means_it_was_blocked() {
    let contacts = vec![ContactEvent {
        frame: 210,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &contacts,
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].outcome, SuperArtOutcome::Blocked);
}

/// 接触の記録が無くても、相手の HP が減っていれば当たっている。
#[test]
fn health_taken_off_the_opponent_also_means_a_hit() {
    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &[],
        &[dealt(205, 0.35)],
        &[],
        &[],
    );

    assert_eq!(events[0].outcome, SuperArtOutcome::Hit);
    assert!((events[0].damage - 0.35).abs() < 1e-6);
}

/// 起点から離れた被弾は、その SA の結果ではない。
#[test]
fn health_taken_off_much_later_is_not_this_supers_damage() {
    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &[],
        &[dealt(231, 0.35)],
        &[],
        &[],
    );

    assert_eq!(events[0].damage, 0.0, "離れた被弾を結び付けている");
}

/// 撃った時刻を遡れているのに何も起きていなければ、設置型のように
/// すぐには触れない技。空振りとは断定しない。
#[test]
fn a_traced_super_that_touched_nothing_is_not_called_a_whiff() {
    let mut states = vec![MeterState::Free; LENGTH];
    states[180..190].fill(MeterState::Startup);

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[states, vec![MeterState::Free; LENGTH]],
        &[],
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].outcome, SuperArtOutcome::NoImmediateContact);
}

/// 撃った時刻も遡れていなければ、結果は分からない。
#[test]
fn an_untraced_super_leaves_the_outcome_unconfirmed() {
    let events = extract(&features_dropping(3.0, 0.0));

    assert_eq!(events[0].outcome, SuperArtOutcome::Unconfirmed);
}

// ── 後隙を狩られたか ─────────────────────────────────────────────────────

/// 当たった SA は狩られていない。
#[test]
fn a_super_that_hit_is_never_marked_as_punished() {
    let contacts = vec![ContactEvent {
        frame: 210,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let taken = DamageEvent {
        victim: 1,
        start_frame: 230,
        pre_freeze_frame: 230,
        end_frame: 260,
        hp_before: 1.0,
        hp_after: 0.8,
        drop: 0.2,
        round_no: 1,
    };

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &contacts,
        &[taken],
        &[],
        &[],
    );

    assert!(!events[0].punished);
    assert_eq!(events[0].punished_damage, 0.0);
}

/// ガードされた後に被弾していれば、後隙を狩られている。
#[test]
fn health_lost_after_a_blocked_super_is_the_punishment() {
    let contacts = vec![ContactEvent {
        frame: 210,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];
    let taken = DamageEvent {
        victim: 1,
        start_frame: 240,
        pre_freeze_frame: 240,
        end_frame: 270,
        hp_before: 1.0,
        hp_after: 0.75,
        drop: 0.25,
        round_no: 1,
    };

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &contacts,
        &[taken],
        &[],
        &[],
    );

    assert!(events[0].punished);
    assert!((events[0].punished_damage - 0.25).abs() < 1e-6);
}

/// ずっと後の被弾は、その後隙を狩られた結果ではない。
#[test]
fn health_lost_long_after_is_not_the_punishment() {
    let contacts = vec![ContactEvent {
        frame: 210,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];
    let taken = DamageEvent {
        victim: 1,
        start_frame: 301,
        pre_freeze_frame: 301,
        end_frame: 330,
        hp_before: 1.0,
        hp_after: 0.75,
        drop: 0.25,
        round_no: 1,
    };

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &contacts,
        &[taken],
        &[],
        &[],
    );

    assert!(!events[0].punished);
}

// ── どういう文脈で撃ったか ───────────────────────────────────────────────

/// 確定反撃として撃ったのなら、その文脈で記録する。
#[test]
fn a_super_thrown_as_a_punish_is_recorded_as_one() {
    let punishes = vec![PunishChance {
        frame: 195,
        side: 1,
        advantage: 20,
        outcome: PunishOutcome::Success,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 180,
        recovery_end_frame: 200,
        source_contact_frame: Some(180),
        attack_start_frame: Some(196),
        attack_active_frame: Some(202),
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    }];

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &[],
        &[],
        &punishes,
        &[],
    );

    assert_eq!(events[0].context, SuperArtContext::Punish);
}

/// 直前に自分の攻撃が当たっていれば、コンボの締め。
#[test]
fn a_super_after_your_own_hit_is_a_combo_ender() {
    let contacts = vec![ContactEvent {
        frame: 180,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &contacts,
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].context, SuperArtContext::Combo);
}

/// 直前に自分が攻撃を受けていれば、切り返し。
#[test]
fn a_super_thrown_while_under_attack_is_a_reversal() {
    let contacts = vec![ContactEvent {
        frame: 180,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[Vec::new(), Vec::new()],
        &contacts,
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].context, SuperArtContext::DefensiveReversal);
}

/// メーターにガード硬直が残っていても切り返しと読む。
#[test]
fn blockstun_on_the_meter_also_makes_it_a_reversal() {
    let mut states = vec![MeterState::Free; LENGTH];
    states[160..180].fill(MeterState::Stun);

    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[states, vec![MeterState::Free; LENGTH]],
        &[],
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].context, SuperArtContext::DefensiveReversal);
}

/// どれにも当たらず、メーターが読めていれば地上戦の中。
#[test]
fn a_super_out_of_neutral_is_recorded_as_neutral() {
    let events = extract_with(
        &features_dropping(3.0, 0.0),
        &[
            vec![MeterState::Free; LENGTH],
            vec![MeterState::Free; LENGTH],
        ],
        &[],
        &[],
        &[],
        &[],
    );

    assert_eq!(events[0].context, SuperArtContext::Neutral);
}

/// メーターが読めていなければ、文脈は分からない。
#[test]
fn without_a_meter_the_context_stays_unknown() {
    let events = extract(&features_dropping(3.0, 0.0));

    assert_eq!(events[0].context, SuperArtContext::Unknown);
}

// ── CA かどうか ──────────────────────────────────────────────────────────

/// 3 本使っていて、CA の表示も出ていれば CA。
#[test]
fn three_stocks_with_the_label_is_a_critical_art() {
    let mut features = features_dropping(3.0, 0.0);
    for feature in features.iter_mut() {
        feature.left_ca_ready = true;
    }

    assert!(extract(&features)[0].critical_art);
}

/// 表示が無ければ SA3。同じ 3 本でも別の技。
#[test]
fn three_stocks_without_the_label_is_a_level_three_super() {
    assert!(!extract(&features_dropping(3.0, 0.0))[0].critical_art);
}

/// 3 本使っていなければ、表示が出ていても CA ではない。
#[test]
fn fewer_than_three_stocks_is_never_a_critical_art() {
    let mut features = features_dropping(2.0, 0.0);
    for feature in features.iter_mut() {
        feature.left_ca_ready = true;
    }

    assert!(!extract(&features)[0].critical_art);
}

// ── 決着 ─────────────────────────────────────────────────────────────────

/// 当てて、そのラウンドを取っていれば決着。
#[test]
fn a_super_that_hit_and_ended_the_round_is_a_finish() {
    let contacts = vec![ContactEvent {
        frame: 210,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let mut rounds = rounds();
    rounds[0].winner = Some(1);
    rounds[0].end_frame = 260;

    let events = extract_super_arts(SuperArtInputs {
        features: &features_dropping(3.0, 0.0),
        meter_state: &[Vec::new(), Vec::new()],
        contacts: &contacts,
        damage: &[],
        punishes: &[],
        rounds: &rounds,
        freeze_spans: &[],
    });

    assert!(events[0].ko);
}

/// ラウンドを取っていても、当たっていなければ決着ではない。
#[test]
fn a_super_that_did_not_hit_is_not_a_finish() {
    let mut rounds = rounds();
    rounds[0].winner = Some(1);
    rounds[0].end_frame = 260;

    let events = extract_super_arts(SuperArtInputs {
        features: &features_dropping(3.0, 0.0),
        meter_state: &[Vec::new(), Vec::new()],
        contacts: &[],
        damage: &[],
        punishes: &[],
        rounds: &rounds,
        freeze_spans: &[],
    });

    assert!(!events[0].ko);
}

/// 相手が取ったラウンドなら決着ではない。
#[test]
fn a_round_the_opponent_won_is_not_your_finish() {
    let contacts = vec![ContactEvent {
        frame: 210,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let mut rounds = rounds();
    rounds[0].winner = Some(2);
    rounds[0].end_frame = 260;

    let events = extract_super_arts(SuperArtInputs {
        features: &features_dropping(3.0, 0.0),
        meter_state: &[Vec::new(), Vec::new()],
        contacts: &contacts,
        damage: &[],
        punishes: &[],
        rounds: &rounds,
        freeze_spans: &[],
    });

    assert!(!events[0].ko);
}

// ── 既存の観測列からの読み取り ───────────────────────────────────────────

#[test]
fn gauge_drop_and_meter_action_create_a_level_two_hit() {
    let features = features_with_spend(2.7, 0.7);
    let mut states = [vec![MeterState::Free; 100], vec![MeterState::Free; 100]];
    states[0][45..50].fill(MeterState::Invincible);
    let contacts = vec![ContactEvent {
        frame: 70,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let damage = vec![DamageEvent {
        victim: 2,
        start_frame: 70,
        pre_freeze_frame: 45,
        end_frame: 80,
        hp_before: 1.0,
        hp_after: 0.8,
        drop: 0.2,
        round_no: 1,
    }];
    let events = extract_super_arts(SuperArtInputs {
        features: &features,
        meter_state: &states,
        contacts: &contacts,
        damage: &damage,
        punishes: &[],
        rounds: &legacy_rounds(),
        freeze_spans: &[],
    });

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, 2);
    assert_eq!(events[0].frame, 45);
    assert_eq!(events[0].outcome, SuperArtOutcome::Hit);
    assert_eq!(events[0].damage, 0.2);
    assert_eq!(events[0].confidence, EventConfidence::High);
    assert!(!events[0].critical_art);
}

#[test]
fn ca_without_immediate_contact_is_not_called_a_whiff() {
    let mut features = features_with_spend(3.0, 0.2);
    for feature in &mut features[..50] {
        feature.left_ca_ready = true;
    }
    let states = [vec![MeterState::Free; 300], vec![MeterState::Free; 300]];
    let later_unrelated_contact = [ContactEvent {
        frame: 250,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let events = extract_super_arts(SuperArtInputs {
        features: &features,
        meter_state: &states,
        contacts: &later_unrelated_contact,
        damage: &[],
        punishes: &[],
        rounds: &legacy_rounds(),
        freeze_spans: &[(42, 60)],
    });

    assert_eq!(events.len(), 1);
    assert_eq!(events[0].level, 3);
    assert!(events[0].critical_art);
    assert_eq!(events[0].frame, 42);
    assert_eq!(events[0].outcome, SuperArtOutcome::NoImmediateContact);
}

#[test]
fn stock_boundary_jitter_does_not_create_a_super_event() {
    let features = features_with_spend(3.0, 2.94);
    let states = [vec![MeterState::Free; 300], vec![MeterState::Free; 300]];
    let events = extract_super_arts(SuperArtInputs {
        features: &features,
        meter_state: &states,
        contacts: &[],
        damage: &[],
        punishes: &[],
        rounds: &legacy_rounds(),
        freeze_spans: &[(42, 60)],
    });
    assert!(events.is_empty());
}

fn features_with_spend(before: f32, after: f32) -> Vec<FrameFeatures> {
    (0..300)
        .map(|index| {
            let mut feature = feat(index, 1.0, 1.0);
            feature.is_match_screen = true;
            feature.left_super_value = if index < 50 { before } else { after };
            feature.left_super_uncertain = false;
            feature
        })
        .collect()
}

fn legacy_rounds() -> Vec<RoundInfo> {
    vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 299,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }]
}
