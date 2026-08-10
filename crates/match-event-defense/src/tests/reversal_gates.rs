//! 無敵技を狩られた場面を切り出すまでの条件に対するテスト。
//!
//! メーターに無敵が出るのは無敵技だけではない。投げ抜け、被投げ、起き
//! 上がり、DI のアーマー、テレポート。どれも同じ表示になる。
//!
//! 無敵技だけを拾うには「無敵の直後に自分の攻撃判定が出る」ことを要求
//! する。技でないシステム無敵には攻撃が続かない。ここを緩めると、
//! 投げ抜けた直後の被弾が「無敵技を狩られた」として並ぶ。

use crate::test_support::*;

/// 無敵技を出して、当たらず、後隙を狩られた観測列。
///
/// f100..106 無敵 → f106..112 攻撃判定 → f130 被弾。
fn punished_reversal() -> (Vec<MeterState>, Vec<MeterState>, Vec<DamageEvent>) {
    let length = 180;
    let mut own = vec![MeterState::Free; length];
    own[100..106].fill(MeterState::Invincible);
    own[106..112].fill(MeterState::Active);
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 130,
        pre_freeze_frame: 130,
        end_frame: 145,
        hp_before: 1.0,
        hp_after: 0.76,
        drop: 0.24,
        round_no: 1,
    }];
    (own, vec![MeterState::Free; length], damage)
}

/// 観測列を抽出器へ通す。
fn extract(
    own: Vec<MeterState>,
    opponent: Vec<MeterState>,
    contacts: Vec<ContactEvent>,
    damage: Vec<DamageEvent>,
    segments: Vec<InputSegment>,
    teleports: Vec<TeleportEvent>,
) -> Vec<ReversalEvent> {
    let length = own.len();
    let features: Vec<_> = (0..length)
        .map(|frame| feat(frame as u32, 1.0, 1.0))
        .collect();
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: length as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];

    crate::reversals::extract_reversals(crate::reversals::ReversalInputs {
        features: &features,
        meter_state: &[own, opponent],
        meter_epoch: &[vec![0; length], vec![0; length]],
        contacts: &contacts,
        damage: &damage,
        segments: &[segments, vec![]],
        rounds: &rounds,
        teleports: &teleports,
    })
}

// ── 無敵技だと言えるか ───────────────────────────────────────────────────

/// 無敵の直後に攻撃判定が出ていれば、技を撃っている。
#[test]
fn invincibility_followed_by_an_attack_is_a_move() {
    let (own, opponent, damage) = punished_reversal();

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert_eq!(reversals.len(), 1);
    assert_eq!(reversals[0].frame, 100);
    assert!((reversals[0].drop - 0.24).abs() < 1e-6);
}

/// 攻撃が続かない無敵は、投げ抜けや起き上がりのシステム無敵。技を
/// 撃ったわけではないので、狩られた話にならない。
#[test]
fn invincibility_without_an_attack_is_not_a_move() {
    let (mut own, opponent, damage) = punished_reversal();
    own[106..112].fill(MeterState::Free);

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert!(reversals.is_empty(), "システム無敵を無敵技にしている");
}

/// 攻撃が遠く離れて出るのなら、その無敵から出た技ではない。
#[test]
fn an_attack_long_after_the_invincibility_is_a_separate_move() {
    let (mut own, opponent, damage) = punished_reversal();
    own[106..112].fill(MeterState::Free);
    own[116..122].fill(MeterState::Active);

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert!(reversals.is_empty());
}

/// 飛び道具でも技は技。
#[test]
fn a_projectile_after_the_invincibility_also_counts_as_a_move() {
    let (mut own, opponent, damage) = punished_reversal();
    own[106..112].fill(MeterState::ProjectileActive);

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert_eq!(reversals.len(), 1);
}

/// 短く途切れた無敵は同じ一つの技。別々に数えると、一度の失敗が
/// 二度に見える。
#[test]
fn a_briefly_interrupted_invincibility_is_still_one_move() {
    let (mut own, opponent, damage) = punished_reversal();
    own[102] = MeterState::Free;

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert_eq!(reversals.len(), 1, "一つの技を二つに割っている");
    assert_eq!(reversals[0].frame, 100);
}

/// 大きく離れた無敵は別の技。繋ぐと、二度撃ったことが一度になる。
#[test]
fn two_invincible_runs_far_apart_are_two_moves() {
    let length = 300;
    let mut own = vec![MeterState::Free; length];
    own[100..106].fill(MeterState::Invincible);
    own[106..112].fill(MeterState::Active);
    own[200..206].fill(MeterState::Invincible);
    own[206..212].fill(MeterState::Active);
    let damage = vec![
        DamageEvent {
            victim: 1,
            start_frame: 130,
            pre_freeze_frame: 130,
            end_frame: 145,
            hp_before: 1.0,
            hp_after: 0.76,
            drop: 0.24,
            round_no: 1,
        },
        DamageEvent {
            victim: 1,
            start_frame: 230,
            pre_freeze_frame: 230,
            end_frame: 245,
            hp_before: 0.76,
            hp_after: 0.56,
            drop: 0.20,
            round_no: 1,
        },
    ];

    let reversals = extract(
        own,
        vec![MeterState::Free; length],
        vec![],
        damage,
        vec![],
        vec![],
    );

    assert_eq!(reversals.len(), 2, "別々の技をまとめている");
}

// ── 別のイベントが扱う無敵 ───────────────────────────────────────────────

/// DI のアーマーは無敵技ではない。DI の指摘が扱う。
#[test]
fn a_drive_impact_is_not_a_failed_reversal() {
    let (own, opponent, damage) = punished_reversal();
    let mut di = idle_input(98, 102);
    di.badges = vec!["DI".to_string()];

    let reversals = extract(own, opponent, vec![], damage, vec![di], vec![]);

    assert!(reversals.is_empty(), "DI を無敵技にしている");
}

/// テレポートも無敵技ではない。テレポートの指摘が扱う。
#[test]
fn a_teleport_is_not_a_failed_reversal() {
    let (own, opponent, damage) = punished_reversal();
    let teleport = TeleportEvent {
        attacker: 1,
        defender: 2,
        input_frame: 98,
        inv_start_frame: 100,
        inv_end_frame: 106,
        followup_attack_frame: Some(108),
        followup_contact_frame: None,
        airborne: false,
        defender_actionable: true,
        context: TeleportContext::NakedAttack,
        response: None,
        outcome: ThreatOutcome::Whiffed,
        damage: 0.0,
        dp_reachability: DpReachability::Unknown,
        round_no: 1,
        confidence: 1.0,
    };

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![teleport]);

    assert!(reversals.is_empty(), "テレポートを無敵技にしている");
}

/// 相手のテレポートは、自分の無敵技を打ち消す理由にならない。
#[test]
fn the_opponents_teleport_does_not_excuse_your_reversal() {
    let (own, opponent, damage) = punished_reversal();
    let teleport = TeleportEvent {
        attacker: 2,
        defender: 1,
        input_frame: 98,
        inv_start_frame: 100,
        inv_end_frame: 106,
        followup_attack_frame: Some(108),
        followup_contact_frame: None,
        airborne: false,
        defender_actionable: true,
        context: TeleportContext::NakedAttack,
        response: None,
        outcome: ThreatOutcome::Whiffed,
        damage: 0.0,
        dp_reachability: DpReachability::Unknown,
        round_no: 1,
        confidence: 1.0,
    };

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![teleport]);

    assert_eq!(reversals.len(), 1);
}

// ── 結果 ─────────────────────────────────────────────────────────────────

/// 当たっていれば無敵技は通っている。指摘しない。
#[test]
fn a_reversal_that_landed_is_not_reported() {
    let (own, opponent, damage) = punished_reversal();
    let contacts = vec![ContactEvent {
        frame: 108,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }];

    let reversals = extract(own, opponent, contacts, damage, vec![], vec![]);

    assert!(reversals.is_empty());
}

/// ガードされたのなら、届いてはいる。空振りと書き分ける。
#[test]
fn a_reversal_that_was_blocked_is_marked_as_blocked() {
    let (own, opponent, damage) = punished_reversal();
    let contacts = vec![ContactEvent {
        frame: 108,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    let reversals = extract(own, opponent, contacts, damage, vec![], vec![]);

    assert_eq!(reversals.len(), 1);
    assert!(reversals[0].blocked, "ガードされたのに空振りにしている");
}

/// 何にも触れていなければ空振り。
#[test]
fn a_reversal_that_touched_nothing_is_a_whiff() {
    let (own, opponent, damage) = punished_reversal();

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert!(!reversals[0].blocked);
}

/// 逃げ切った空振りは指摘しない。リスクが表に出ていない。
#[test]
fn a_whiffed_reversal_that_cost_nothing_is_not_reported() {
    let (own, opponent, _) = punished_reversal();

    let reversals = extract(own, opponent, vec![], vec![], vec![], vec![]);

    assert!(reversals.is_empty());
}

/// 被弾が遠ければ、その無敵技を狩られた結果ではない。
#[test]
fn damage_long_after_the_reversal_is_not_its_punishment() {
    let (own, opponent, mut damage) = punished_reversal();
    damage[0].start_frame = 172;

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert!(reversals.is_empty(), "無関係な被弾を狩られた結果にしている");
}

/// 相手が受けた被弾は自分が狩られた証拠ではない。
#[test]
fn damage_the_opponent_took_is_not_your_punishment() {
    let (own, opponent, mut damage) = punished_reversal();
    damage[0].victim = 2;

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert!(reversals.is_empty());
}

// ── 確度 ─────────────────────────────────────────────────────────────────

/// 入力表示で無敵技が確認できていれば、確度は高い。
#[test]
fn an_explicit_input_makes_it_certain() {
    let (own, opponent, damage) = punished_reversal();
    let mut dp = idle_input(98, 102);
    dp.badges = vec!["DP".to_string()];

    let reversals = extract(own, opponent, vec![], damage, vec![dp], vec![]);

    assert_eq!(reversals[0].confidence, EventConfidence::High);
}

/// 直前にガード硬直があれば、切り返しとして撃ったことが読み取れる。
#[test]
fn coming_out_of_blockstun_also_makes_it_certain() {
    let (mut own, opponent, damage) = punished_reversal();
    own[90..100].fill(MeterState::Stun);

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert_eq!(reversals[0].confidence, EventConfidence::High);
}

/// 直前に攻撃を受けていた記録があっても、切り返しとして読める。
#[test]
fn a_recent_contact_against_you_also_makes_it_certain() {
    let (own, opponent, damage) = punished_reversal();
    let contacts = vec![ContactEvent {
        frame: 60,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    }];

    let reversals = extract(own, opponent, contacts, damage, vec![], vec![]);

    assert_eq!(reversals[0].confidence, EventConfidence::High);
}

/// 守勢の裏付けも入力表示も無ければ、確度を下げる。何もない場面で
/// 撃った技かもしれない。
#[test]
fn a_reversal_out_of_nowhere_is_less_certain() {
    let (own, opponent, damage) = punished_reversal();

    let reversals = extract(own, opponent, vec![], damage, vec![], vec![]);

    assert_eq!(
        reversals[0].confidence,
        EventConfidence::Medium,
        "裏付けが無いのに確度を下げていない"
    );
}
