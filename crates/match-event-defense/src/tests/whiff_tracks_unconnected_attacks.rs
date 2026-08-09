use crate::test_support::*;
use crate::whiffs::{extract_whiffs, WhiffInputs};

/// P1 が f100..110 に攻撃判定を出す固定の場面を作る。
fn knockdown_free_fixture() -> ([Vec<MeterState>; 2], Vec<FrameFeatures>, Vec<RoundInfo>) {
    use MeterState::*;
    let n = 300usize;
    let mut own = vec![Free; n];
    let opp = vec![Free; n];
    for state in own.iter_mut().take(100).skip(96) {
        *state = Startup;
    }
    for state in own.iter_mut().take(110).skip(100) {
        *state = Active;
    }
    for state in own.iter_mut().take(130).skip(110) {
        *state = Recovery;
    }
    let features: Vec<_> = (0..n as u32).map(|index| feat(index, 1.0, 1.0)).collect();
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 299,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    ([own, opp], features, rounds)
}

/// 抽出に効く周辺イベントだけを差し替えるための束。既定はすべて空。
#[derive(Default)]
struct Around<'a> {
    contacts: &'a [ContactEvent],
    damage: &'a [DamageEvent],
    throw_actions: &'a [ThrowActionEvent],
    drive_impacts: &'a [DriveImpactEvent],
    reversals: &'a [ReversalEvent],
}

fn extract(
    meter_state: &[Vec<MeterState>; 2],
    features: &[FrameFeatures],
    rounds: &[RoundInfo],
    around: Around<'_>,
) -> Vec<WhiffEvent> {
    let n = meter_state[0].len();
    let epochs = [vec![0; n], vec![0; n]];
    extract_whiffs(WhiffInputs {
        features,
        meter_state,
        meter_epoch: &epochs,
        contacts: around.contacts,
        damage: around.damage,
        throw_actions: around.throw_actions,
        drive_impacts: around.drive_impacts,
        reversals: around.reversals,
        rounds,
    })
}

fn around_contacts(contacts: &[ContactEvent]) -> Around<'_> {
    Around {
        contacts,
        ..Around::default()
    }
}

/// 接触しなかった攻撃判定を空振りとして残す。
#[test]
fn an_attack_without_contact_is_a_whiff() {
    let (meter, features, rounds) = knockdown_free_fixture();

    let whiffs = extract(&meter, &features, &rounds, Around::default());

    assert_eq!(whiffs.len(), 1);
    assert_eq!(whiffs[0].side, 1);
    assert_eq!(whiffs[0].frame, 100);
    assert_eq!(whiffs[0].end_frame, 109);
    assert_eq!(whiffs[0].outcome, WhiffOutcome::Unpunished);
    assert_eq!(whiffs[0].drop, 0.0);
}

/// ガードさせた技は届いているので空振りではない。
#[test]
fn a_blocked_attack_is_not_a_whiff() {
    let (meter, features, rounds) = knockdown_free_fixture();
    let contacts = vec![ContactEvent {
        frame: 103,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    assert!(extract(&meter, &features, &rounds, around_contacts(&contacts)).is_empty());
}

/// 硬直中に被弾したら狩られたものとして結果と HP を残す。
#[test]
fn a_punished_whiff_records_the_lost_hp() {
    let (meter, features, rounds) = knockdown_free_fixture();
    let contacts = vec![ContactEvent {
        frame: 118,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 118,
        pre_freeze_frame: 118,
        end_frame: 140,
        hp_before: 1.0,
        hp_after: 0.75,
        drop: 0.25,
        round_no: 1,
    }];

    let whiffs = extract(
        &meter,
        &features,
        &rounds,
        Around {
            contacts: &contacts,
            damage: &damage,
            ..Around::default()
        },
    );

    assert_eq!(whiffs.len(), 1);
    assert_eq!(whiffs[0].outcome, WhiffOutcome::Punished);
    assert_eq!(whiffs[0].punished_frame, Some(118));
    assert!((whiffs[0].drop - 0.25).abs() < 1e-6);
}

/// 結果窓を過ぎてからの被弾は、その空振りの硬直を狩られた結果ではない。
#[test]
fn damage_after_the_result_window_is_not_attributed() {
    let (meter, features, rounds) = knockdown_free_fixture();
    let contacts = vec![ContactEvent {
        frame: 109 + WHIFF_PUNISH_WINDOW + 1,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    }];

    let whiffs = extract(&meter, &features, &rounds, around_contacts(&contacts));

    assert_eq!(whiffs.len(), 1);
    assert_eq!(whiffs[0].outcome, WhiffOutcome::Unpunished);
}

/// 投げ・Drive Impact・無敵技は専用カードが結果を扱う。
/// ここで数えると同じ被弾が複数のカードへ出る。
#[test]
fn actions_tracked_by_their_own_events_are_excluded() {
    let (meter, features, rounds) = knockdown_free_fixture();

    let throw = vec![ThrowActionEvent {
        thrower: 1,
        input_frame: 96,
        startup_frame: Some(96),
        active_frame: Some(100),
        outcome: ThrowOutcome::ExecutedWhiff,
        damage: 0.0,
        approach: Default::default(),
        confidence: EventConfidence::High,
        round_no: 1,
    }];
    assert!(extract(
        &meter,
        &features,
        &rounds,
        Around {
            throw_actions: &throw,
            ..Around::default()
        },
    )
    .is_empty());

    let impact = vec![DriveImpactEvent {
        side: 1,
        input_frame: 96,
        active_frame: Some(100),
        contact_frame: None,
        outcome: DriveImpactOutcome::Whiffed,
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }];
    assert!(extract(
        &meter,
        &features,
        &rounds,
        Around {
            drive_impacts: &impact,
            ..Around::default()
        },
    )
    .is_empty());

    let reversal = vec![ReversalEvent {
        side: 1,
        frame: 100,
        drop: 0.2,
        blocked: false,
        confidence: EventConfidence::High,
        round_no: 1,
    }];
    assert!(extract(
        &meter,
        &features,
        &rounds,
        Around {
            reversals: &reversal,
            ..Around::default()
        },
    )
    .is_empty());
}

/// 弾を撃つ行動は距離を取って出すのが正常なので空振りとして数えない。
#[test]
fn projectiles_are_not_counted_as_whiffs() {
    use MeterState::*;
    let (mut meter, features, rounds) = knockdown_free_fixture();
    for state in meter[0].iter_mut().take(110).skip(100) {
        *state = ProjectileActive;
    }

    assert!(extract(&meter, &features, &rounds, Around::default()).is_empty());
}

/// 相手の空振りも同じ形で残す。差し返し率の分母になる。
#[test]
fn the_opponents_whiff_is_recorded_for_the_same_reason() {
    use MeterState::*;
    let (mut meter, features, rounds) = knockdown_free_fixture();
    meter.swap(0, 1);
    let contacts = vec![ContactEvent {
        frame: 115,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    assert_eq!(meter[1][105], Active);

    let whiffs = extract(&meter, &features, &rounds, around_contacts(&contacts));

    assert_eq!(whiffs.len(), 1);
    assert_eq!(whiffs[0].side, 2);
    assert_eq!(whiffs[0].outcome, WhiffOutcome::Punished);
}

/// 接触判定の猶予は境界そのもの。猶予いっぱいの接触は「届いた」側で、
/// 1フレーム外れたら空振りになる。ここがずれると空振りの数が変わる。
#[test]
fn the_contact_grace_window_decides_at_its_edge() {
    let (meter, features, rounds) = knockdown_free_fixture();

    // 攻撃判定の開始より猶予ぶん前の接触までは、その技が届いたものとして扱う。
    let at_edge = vec![ContactEvent {
        frame: 100 - WHIFF_CONTACT_GRACE,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    assert!(extract(&meter, &features, &rounds, around_contacts(&at_edge)).is_empty());

    let outside = vec![ContactEvent {
        frame: 100 - WHIFF_CONTACT_GRACE - 1,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    assert_eq!(
        extract(&meter, &features, &rounds, around_contacts(&outside)).len(),
        1
    );
}

/// 攻撃判定の終了フレームちょうどの被弾は、その技の硬直を狩られた結果では
/// ない。まだ攻撃判定が出ている最中だからである。
#[test]
fn damage_on_the_last_active_frame_is_not_a_punish() {
    let (meter, features, rounds) = knockdown_free_fixture();

    let on_edge = vec![ContactEvent {
        frame: 109,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let whiffs = extract(&meter, &features, &rounds, around_contacts(&on_edge));
    assert_eq!(whiffs.len(), 1);
    assert_eq!(whiffs[0].outcome, WhiffOutcome::Unpunished);

    let after = vec![ContactEvent {
        frame: 110,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let whiffs = extract(&meter, &features, &rounds, around_contacts(&after));
    assert_eq!(whiffs[0].outcome, WhiffOutcome::Punished);
}

/// meter epoch が 0 以外でも同じように扱う。epoch の符号を取り違えると、
/// リセット後の区間がまるごと落ちる。
#[test]
fn a_non_zero_epoch_is_still_analysed() {
    let (meter, features, rounds) = knockdown_free_fixture();
    let n = meter[0].len();
    let epochs = [vec![3; n], vec![3; n]];

    let whiffs = extract_whiffs(WhiffInputs {
        features: &features,
        meter_state: &meter,
        meter_epoch: &epochs,
        contacts: &[],
        damage: &[],
        throw_actions: &[],
        drive_impacts: &[],
        reversals: &[],
        rounds: &rounds,
    });

    assert_eq!(whiffs.len(), 1);
    assert_eq!(whiffs[0].confidence, EventConfidence::High);
}

/// epoch が途中で変わったら、結果まで確定したとは言えない。
#[test]
fn an_epoch_reset_inside_the_result_window_lowers_confidence() {
    let (meter, features, rounds) = knockdown_free_fixture();
    let n = meter[0].len();
    let mut own_epoch = vec![0; n];
    for value in own_epoch.iter_mut().skip(120) {
        *value = 1;
    }

    let whiffs = extract_whiffs(WhiffInputs {
        features: &features,
        meter_state: &meter,
        meter_epoch: &[own_epoch, vec![0; n]],
        contacts: &[],
        damage: &[],
        throw_actions: &[],
        drive_impacts: &[],
        reversals: &[],
        rounds: &rounds,
    });

    assert_eq!(whiffs.len(), 1);
    assert_eq!(whiffs[0].confidence, EventConfidence::Medium);
}

/// 除外は「同じ側の」「重なる」行動にだけ効く。相手側の投げや、離れた
/// フレームの DI で自分の空振りが消えてはならない。
#[test]
fn the_exclusion_only_matches_the_same_side_and_overlap() {
    let (meter, features, rounds) = knockdown_free_fixture();

    let other_side = vec![ThrowActionEvent {
        thrower: 2,
        input_frame: 96,
        startup_frame: Some(96),
        active_frame: Some(100),
        outcome: ThrowOutcome::ExecutedWhiff,
        damage: 0.0,
        approach: Default::default(),
        confidence: EventConfidence::High,
        round_no: 1,
    }];
    assert_eq!(
        extract(
            &meter,
            &features,
            &rounds,
            Around {
                throw_actions: &other_side,
                ..Around::default()
            },
        )
        .len(),
        1
    );

    let far_away = vec![DriveImpactEvent {
        side: 1,
        input_frame: 200,
        active_frame: Some(220),
        contact_frame: None,
        outcome: DriveImpactOutcome::Whiffed,
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }];
    assert_eq!(
        extract(
            &meter,
            &features,
            &rounds,
            Around {
                drive_impacts: &far_away,
                ..Around::default()
            },
        )
        .len(),
        1
    );
}

/// 攻撃判定が動画の最終フレームまで続く場合。走査の終端条件を誤ると
/// 範囲外参照になる。結果窓が取れないので確度は落とす。
#[test]
fn an_active_run_reaching_the_last_frame_is_handled() {
    use MeterState::*;
    let (mut meter, features, rounds) = knockdown_free_fixture();
    for state in meter[0].iter_mut().skip(100) {
        *state = Active;
    }

    let whiffs = extract(&meter, &features, &rounds, Around::default());

    assert_eq!(whiffs.len(), 1);
    assert_eq!(whiffs[0].frame, 100);
    assert_eq!(whiffs[0].end_frame, meter[0].len() as u32 - 1);
}

/// epoch 系列が meter より短い壊れた入力でも、既定値で「不明」として
/// 扱い、有効な epoch と取り違えない。
#[test]
fn a_short_epoch_series_is_treated_as_unknown() {
    let (meter, features, rounds) = knockdown_free_fixture();

    let whiffs = extract_whiffs(WhiffInputs {
        features: &features,
        meter_state: &meter,
        // 攻撃判定の開始位置まで届かない長さにする。
        meter_epoch: &[vec![0; 50], vec![0; 50]],
        contacts: &[],
        damage: &[],
        throw_actions: &[],
        drive_impacts: &[],
        reversals: &[],
        rounds: &rounds,
    });

    assert!(whiffs.is_empty());
}

/// 接触猶予は攻撃判定の終了側にも効く。開始側だけ検査していると、
/// 遅れて表示された接触を空振りと誤認する。
#[test]
fn the_contact_grace_also_covers_the_trailing_edge() {
    let (meter, features, rounds) = knockdown_free_fixture();

    let at_edge = vec![ContactEvent {
        frame: 109 + WHIFF_CONTACT_GRACE,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    assert!(extract(&meter, &features, &rounds, around_contacts(&at_edge)).is_empty());

    let outside = vec![ContactEvent {
        frame: 109 + WHIFF_CONTACT_GRACE + 1,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    assert_eq!(
        extract(&meter, &features, &rounds, around_contacts(&outside)).len(),
        1
    );
}

/// 被弾量は、狩られた接触に対応する被弾だけから取る。接触より前の被弾や
/// 結果窓を越えた被弾を混ぜると、その空振りの代償を過大に見せる。
#[test]
fn only_damage_tied_to_the_punishing_contact_is_counted() {
    let (meter, features, rounds) = knockdown_free_fixture();
    let contacts = vec![ContactEvent {
        frame: 120,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    }];
    let damage = |start_frame: u32, drop: f32| DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 10,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    };
    let events = vec![
        // 接触より猶予ぶん手前までは同じ被弾として認める。
        damage(120 - WHIFF_CONTACT_GRACE, 0.2),
        // それより前は別の被弾。
        damage(120 - WHIFF_CONTACT_GRACE - 1, 0.9),
        // 結果窓を越えた被弾も別。
        damage(109 + WHIFF_PUNISH_WINDOW + 1, 0.9),
    ];

    let whiffs = extract(
        &meter,
        &features,
        &rounds,
        Around {
            contacts: &contacts,
            damage: &events,
            ..Around::default()
        },
    );

    assert_eq!(whiffs.len(), 1);
    assert!((whiffs[0].drop - 0.2).abs() < 1e-6);
}

/// 除外判定の重なりも端で決める。猶予ぶん手前で始まった投げは同じ行動
/// だが、そこから外れたものは別行動なので空振りを消してはならない。
#[test]
fn the_exclusion_overlap_is_decided_at_its_edge() {
    let (meter, features, rounds) = knockdown_free_fixture();
    let throw_at = |active: u32| {
        vec![ThrowActionEvent {
            thrower: 1,
            input_frame: active.saturating_sub(4),
            startup_frame: Some(active.saturating_sub(4)),
            active_frame: Some(active),
            outcome: ThrowOutcome::ExecutedWhiff,
            damage: 0.0,
            approach: Default::default(),
            confidence: EventConfidence::High,
            round_no: 1,
        }]
    };

    let inside = throw_at(100 - WHIFF_CONTACT_GRACE);
    assert!(extract(
        &meter,
        &features,
        &rounds,
        Around {
            throw_actions: &inside,
            ..Around::default()
        },
    )
    .is_empty());

    let outside = throw_at(100 - WHIFF_CONTACT_GRACE - 1);
    assert_eq!(
        extract(
            &meter,
            &features,
            &rounds,
            Around {
                throw_actions: &outside,
                ..Around::default()
            },
        )
        .len(),
        1
    );
}
