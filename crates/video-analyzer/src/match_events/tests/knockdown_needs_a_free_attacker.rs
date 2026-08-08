use super::super::knockdowns::{extract_knockdowns, KnockdownInputs};
use super::support::*;

const FRAMES: usize = 400;

/// P2 が f100 で被弾して f100..200 まで `Stun`、f200 に起き上がる。
/// P1 は f110 以降ずっと自由に動ける、という基本形を作る。
fn knockdown_fixture() -> ([Vec<MeterState>; 2], Vec<FrameFeatures>, Vec<RoundInfo>) {
    use MeterState::*;
    let mut attacker = vec![Free; FRAMES];
    let mut down = vec![Free; FRAMES];
    for state in attacker.iter_mut().take(110).skip(100) {
        *state = Active;
    }
    for state in down.iter_mut().take(200).skip(100) {
        *state = Stun;
    }
    let features: Vec<_> = (0..FRAMES as u32)
        .map(|index| feat(index, 1.0, 1.0))
        .collect();
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: FRAMES as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    ([attacker, down], features, rounds)
}

fn knockdown_hit() -> Vec<ContactEvent> {
    vec![ContactEvent {
        frame: 102,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    }]
}

fn extract(
    meter: &[Vec<MeterState>; 2],
    features: &[FrameFeatures],
    rounds: &[RoundInfo],
    contacts: &[ContactEvent],
) -> Vec<KnockdownEvent> {
    let epochs = [vec![0; FRAMES], vec![0; FRAMES]];
    extract_knockdowns(KnockdownInputs {
        features,
        meter_state: meter,
        meter_epoch: &epochs,
        contacts,
        rounds,
    })
}

/// 起き上がりのフレームに攻撃判定が乗っていれば持続当て。
#[test]
fn an_active_frame_on_wakeup_is_a_meaty() {
    let (mut meter, features, rounds) = knockdown_fixture();
    for state in meter[0].iter_mut().take(206).skip(198) {
        *state = MeterState::Active;
    }

    let downs = extract(&meter, &features, &rounds, &knockdown_hit());

    assert_eq!(downs.len(), 1);
    assert_eq!(downs[0].side, 2);
    assert_eq!(downs[0].attacker, 1);
    assert_eq!(downs[0].wakeup_frame, 200);
    assert_eq!(downs[0].okizeme, OkizemeOutcome::Meaty);
}

/// 重ねられなくても、起き上がり直後に攻め始めていれば継続として扱う。
#[test]
fn starting_an_attack_after_wakeup_counts_as_pressure() {
    let (mut meter, features, rounds) = knockdown_fixture();
    for state in meter[0].iter_mut().take(212).skip(205) {
        *state = MeterState::Startup;
    }

    let downs = extract(&meter, &features, &rounds, &knockdown_hit());

    assert_eq!(downs.len(), 1);
    assert_eq!(downs[0].okizeme, OkizemeOutcome::Pressured);
}

/// 何も始めなければ仕切り直し。強い無敵技への警戒として正当な選択でもある
/// ため、ここでは事実として残すだけにする。
#[test]
fn doing_nothing_is_recorded_as_neutral() {
    let (meter, features, rounds) = knockdown_fixture();

    let downs = extract(&meter, &features, &rounds, &knockdown_hit());

    assert_eq!(downs.len(), 1);
    assert_eq!(downs[0].okizeme, OkizemeOutcome::Neutral);
    assert!(downs[0].setup_frames >= 20);
}

/// 攻撃側が拘束され続けている長い `Stun` は連続ガード・連続ヒットであって
/// ダウンではない。長さだけで判定すると、この2つを取り違える。
#[test]
fn a_long_stun_without_a_free_attacker_is_not_a_knockdown() {
    let (mut meter, features, rounds) = knockdown_fixture();
    // 攻撃側が stun 区間じゅう攻撃し続けている＝固めている。
    for state in meter[0].iter_mut().take(200).skip(100) {
        *state = MeterState::Active;
    }

    assert!(extract(&meter, &features, &rounds, &knockdown_hit()).is_empty());
}

/// 短い硬直はダウンとして扱わない。
#[test]
fn a_short_stun_is_not_a_knockdown() {
    let (mut meter, features, rounds) = knockdown_fixture();
    for state in meter[1].iter_mut().take(200).skip(130) {
        *state = MeterState::Free;
    }

    assert!(extract(&meter, &features, &rounds, &knockdown_hit()).is_empty());
}

/// 原因のヒット接触を確認できなければ、誰が取ったダウンか断定しない。
#[test]
fn a_knockdown_without_a_causing_hit_is_not_attributed() {
    let (meter, features, rounds) = knockdown_fixture();

    assert!(extract(&meter, &features, &rounds, &[]).is_empty());

    let blocked = vec![ContactEvent {
        frame: 102,
        attacker: 1,
        victim: 2,
        hit: false,
        projectile: false,
        round_no: 1,
    }];
    assert!(extract(&meter, &features, &rounds, &blocked).is_empty());
}
