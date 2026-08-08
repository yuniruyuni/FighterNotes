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

/// ダウンと認める stun の長さは境界そのもの。ちょうど閾値なら数え、
/// 1フレーム短ければ数えない。
#[test]
fn the_stun_length_threshold_decides_at_its_edge() {
    use MeterState::*;
    let (mut meter, features, rounds) = knockdown_fixture();
    // stun をちょうど KNOCKDOWN_MIN_STUN 分にする。
    for state in meter[1].iter_mut().take(200).skip(100) {
        *state = Free;
    }
    for state in meter[1].iter_mut().take(100 + KNOCKDOWN_MIN_STUN).skip(100) {
        *state = Stun;
    }
    assert_eq!(
        extract(&meter, &features, &rounds, &knockdown_hit()).len(),
        1
    );

    // 1フレーム短いと足りない。
    meter[1][100 + KNOCKDOWN_MIN_STUN - 1] = Free;
    assert!(extract(&meter, &features, &rounds, &knockdown_hit()).is_empty());
}

/// 準備時間も境界で決める。攻撃側が自由な時間がちょうど閾値ならダウンとして
/// 扱い、1フレーム足りなければ固めと区別できない。
#[test]
fn the_setup_length_threshold_decides_at_its_edge() {
    use MeterState::*;
    let (mut meter, features, rounds) = knockdown_fixture();
    // 攻撃側は stun 中ずっと Active（固め）にしておき、末尾だけ自由にする。
    for state in meter[0].iter_mut().take(200).skip(100) {
        *state = Active;
    }
    for state in meter[0]
        .iter_mut()
        .take(200)
        .skip(200 - KNOCKDOWN_MIN_SETUP)
    {
        *state = Free;
    }
    assert_eq!(
        extract(&meter, &features, &rounds, &knockdown_hit()).len(),
        1
    );

    meter[0][200 - KNOCKDOWN_MIN_SETUP] = Active;
    assert!(extract(&meter, &features, &rounds, &knockdown_hit()).is_empty());
}

/// 原因のヒットを探す猶予も境界で決める。stun 開始より猶予ぶん前までは
/// そのダウンの原因として認める。
#[test]
fn the_cause_lookback_decides_at_its_edge() {
    let (meter, features, rounds) = knockdown_fixture();
    let hit = |frame: u32| {
        vec![ContactEvent {
            frame,
            attacker: 1,
            victim: 2,
            hit: true,
            projectile: false,
            round_no: 1,
        }]
    };

    assert_eq!(
        extract(
            &meter,
            &features,
            &rounds,
            &hit(100 - KNOCKDOWN_CAUSE_GRACE)
        )
        .len(),
        1
    );
    assert!(extract(
        &meter,
        &features,
        &rounds,
        &hit(100 - KNOCKDOWN_CAUSE_GRACE - 1)
    )
    .is_empty());
}

/// 起き上がり直後の攻めと認める窓も境界で決める。窓の外で始めた攻撃は
/// その起き上がりへの攻めではない。
#[test]
fn the_pressure_window_decides_at_its_edge() {
    use MeterState::*;
    let (mut meter, features, rounds) = knockdown_fixture();
    let edge = (200 + OKIZEME_PRESSURE_WINDOW) as usize;
    meter[0][edge] = Startup;
    assert_eq!(
        extract(&meter, &features, &rounds, &knockdown_hit())[0].okizeme,
        OkizemeOutcome::Pressured
    );

    meter[0][edge] = Free;
    meter[0][edge + 1] = Startup;
    assert_eq!(
        extract(&meter, &features, &rounds, &knockdown_hit())[0].okizeme,
        OkizemeOutcome::Neutral
    );
}

/// meter epoch が 0 以外でも同じように扱い、起き上がりで epoch が変われば
/// 結果まで確定したとは言わない。
#[test]
fn the_epoch_decides_the_confidence() {
    use MeterState::*;
    let (mut meter, features, rounds) = knockdown_fixture();
    for state in meter[0].iter_mut().take(206).skip(198) {
        *state = Active;
    }
    let n = meter[0].len();

    let same = extract_knockdowns(KnockdownInputs {
        features: &features,
        meter_state: &meter,
        meter_epoch: &[vec![7; n], vec![7; n]],
        contacts: &knockdown_hit(),
        rounds: &rounds,
    });
    assert_eq!(same.len(), 1);
    assert_eq!(same[0].confidence, EventConfidence::High);
    assert_eq!(same[0].okizeme, OkizemeOutcome::Meaty);

    let mut attacker_epoch = vec![0; n];
    for value in attacker_epoch.iter_mut().skip(200) {
        *value = 1;
    }
    let drifted = extract_knockdowns(KnockdownInputs {
        features: &features,
        meter_state: &meter,
        meter_epoch: &[attacker_epoch, vec![0; n]],
        contacts: &knockdown_hit(),
        rounds: &rounds,
    });
    assert_eq!(drifted.len(), 1);
    assert_eq!(drifted[0].confidence, EventConfidence::Medium);
}
