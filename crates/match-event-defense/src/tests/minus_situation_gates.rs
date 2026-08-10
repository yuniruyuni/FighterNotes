//! 不利フレームの場面を切り出すまでの条件に対するテスト。
//!
//! 「ガードして不利を背負った」と言うには、ガードの接触・実際の硬直・
//! 相手が先に動けたこと・その差の大きさが全部要る。どれかを緩めると、
//! 不利でなかった場面や、そもそもガードしていない場面が分母に入る。
//!
//! 分母が狂うと、その上に乗る「偏り」の判断がまるごと狂う。

use crate::test_support::*;

/// 観測列一式。メーター・接触・入力欄・ラウンド。
type Observations = (
    [Vec<MeterState>; 2],
    Vec<ContactEvent>,
    [Vec<InputSegment>; 2],
    Vec<RoundInfo>,
);

/// ガード接触を一つ置いた観測列。ここへ主題の変更だけを足す。
fn fixture() -> Observations {
    minus_press_fixture()
}

/// 入力欄が読めていたことにする。分母へ入れるための最低条件。
fn observed(segments: &mut [Vec<InputSegment>; 2], index: usize) {
    segments[index] = vec![idle_input(110, 140)];
}

// ── 場面を開く条件 ───────────────────────────────────────────────────────

/// ガードした接触だけが不利の始まり。当たった攻撃は不利ではなく被弾。
#[test]
fn a_hit_does_not_open_a_minus_situation() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    contacts[0].hit = true;

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(extracted.situations.is_empty(), "被弾を不利と数えている");
}

/// 飛び道具をガードしても、距離があるので固められてはいない。
#[test]
fn blocking_a_projectile_does_not_open_a_minus_situation() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    contacts[0].projectile = true;

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(
        extracted.situations.is_empty(),
        "弾ガードを固めと数えている"
    );
}

/// 接触の直後にガード硬直が始まっていなければ、ガードしていない。
/// 硬直を確かめないと、接触の記録だけで不利を作れてしまう。
#[test]
fn a_contact_without_the_stun_that_follows_it_is_not_a_block() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    for state in ms[0].iter_mut().take(120).skip(100) {
        *state = MeterState::Free;
    }

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(
        extracted.situations.is_empty(),
        "硬直の無い接触を固めにしている"
    );
}

/// 硬直の始まりが接触から離れていれば、その接触の結果ではない。
#[test]
fn a_stun_that_starts_much_later_is_not_from_this_contact() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    // 硬直の開始を 5 フレーム遅らせる。
    for state in ms[0].iter_mut().take(105).skip(100) {
        *state = MeterState::Free;
    }

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(
        extracted.situations.is_empty(),
        "離れた硬直を結び付けている"
    );
}

/// 相手より先に動けるなら不利ではない。向きを取り違えると、有利な
/// 場面まで「不利からの暴れ」に数える。
#[test]
fn becoming_actionable_first_is_not_a_minus_situation() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    // 相手の後隙を伸ばし、こちらが先に動けるようにする。
    for state in ms[1].iter_mut().take(140).skip(105) {
        *state = MeterState::MotionRecovery;
    }

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(
        extracted.situations.is_empty(),
        "有利な場面を不利にしている"
    );
}

// ── 不利幅 ───────────────────────────────────────────────────────────────

/// 不利が大きすぎる場面は扱わない。20 フレーム不利なら押すも押さないも
/// 無いので、回答の偏りを語る意味がない。
#[test]
fn a_minus_too_large_to_answer_is_left_out() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    // 硬直明けを遅らせて不利 16F にする。
    for state in ms[0].iter_mut().take(131).skip(120) {
        *state = MeterState::Stun;
    }
    for state in ms[0].iter_mut().take(135).skip(131) {
        *state = MeterState::Startup;
    }

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(
        extracted.situations.is_empty(),
        "答えようのない不利を数えている"
    );
}

/// 上限ちょうどまでは扱う。
#[test]
fn a_minus_at_the_upper_limit_is_still_counted() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    for state in ms[0].iter_mut().take(130).skip(120) {
        *state = MeterState::Stun;
    }
    for state in ms[0].iter_mut().take(134).skip(130) {
        *state = MeterState::Startup;
    }

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1, "上限ちょうどを落としている");
    assert_eq!(extracted.situations[0].minus_frames, 15);
}

// ── 分母に入れる条件 ─────────────────────────────────────────────────────

/// 入力欄が読めていない機会は分母に入れない。欠測を「何もしなかった」と
/// 数えると、ガード継続の回数が水増しされる。
#[test]
fn a_moment_without_a_readable_input_display_is_not_in_the_denominator() {
    let (ms, contacts, segs, rounds) = fixture();

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(extracted.situations.is_empty());
}

/// 補間で埋めた入力も読めたことにしない。
#[test]
fn a_repaired_input_display_does_not_count_as_readable() {
    let (ms, contacts, mut segs, rounds) = fixture();
    let mut idle = idle_input(110, 140);
    idle.evidence = InputEvidence {
        observed_frames: 0,
        repaired_frames: 30,
    };
    segs[0] = vec![idle];

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(
        extracted.situations.is_empty(),
        "補間を読めた扱いにしている"
    );
}

/// 離れた時刻の入力欄は、その瞬間が読めていた証拠にならない。
#[test]
fn an_input_display_from_another_moment_does_not_cover_this_one() {
    let (ms, contacts, mut segs, rounds) = fixture();
    segs[0] = vec![idle_input(200, 240)];

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(
        extracted.situations.is_empty(),
        "離れた時刻で読めたことにしている"
    );
}

// ── 回答として数えない行動 ───────────────────────────────────────────────

/// 硬直明けに技が出ていなければ、打撃も投げも選んでいない。分母には
/// 残すが、特定の回答としては数えない。
#[test]
fn doing_nothing_stays_in_the_denominator_without_being_an_answer() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    for state in ms[0].iter_mut().take(130).skip(120) {
        *state = MeterState::Free;
    }

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1, "分母から外している");
    assert_eq!(extracted.situations[0].fastest_action, None);
    assert!(
        extracted.presses.is_empty(),
        "何もしていないのに押したことにしている"
    );
}

/// 技が出るのが遅ければ「最速」ではない。硬直明けから離れた技は、
/// 別の判断で出したもの。
#[test]
fn a_move_that_starts_late_is_not_the_fastest_answer() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    // 発生の開始を 2 フレーム遅らせる。
    for state in ms[0].iter_mut().take(122).skip(120) {
        *state = MeterState::Free;
    }

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1);
    assert_eq!(
        extracted.situations[0].fastest_action, None,
        "遅れて出た技を最速と数えている"
    );
}

/// 無敵技は打撃や投げの回答ではない。切り返しの指摘が扱う。
#[test]
fn an_invincible_move_is_not_a_strike_or_a_throw() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    ms[0][121] = MeterState::Invincible;

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1, "分母から外している");
    assert_eq!(extracted.situations[0].fastest_action, None);
    assert!(extracted.presses.is_empty());
}

/// 相手の硬直に合わせて押したのなら、暴れではなく差し返し。
#[test]
fn pressing_into_the_opponents_recovery_is_not_a_mash() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    ms[1][120] = MeterState::Recovery;

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1);
    assert_eq!(extracted.situations[0].fastest_action, None);
    assert!(extracted.presses.is_empty());
}

/// 入力を結び付けられなければ、何を押したのかが決まらない。分母には
/// 残す。
#[test]
fn a_move_without_a_linked_input_stays_unclassified() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1);
    assert_eq!(extracted.situations[0].fastest_action, None);
    assert!(extracted.presses.is_empty());
}

/// ガードした接触より前の入力は、その場面の回答ではない。
#[test]
fn an_input_from_before_the_block_is_not_the_answer() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(95));

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1);
    assert_eq!(
        extracted.situations[0].fastest_action, None,
        "ガード前の入力を回答にしている"
    );
}

// ── 有利側 ───────────────────────────────────────────────────────────────

/// 有利が小さければ、攻めを継続する機会として数えない。
#[test]
fn a_small_advantage_is_not_a_pressure_chance() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140)];
    // 硬直明けを早めて有利 2F にする。
    for state in ms[0].iter_mut().take(120).skip(117) {
        *state = MeterState::Free;
    }

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(
        extracted.advantages.is_empty(),
        "攻めが間に合わない有利まで数えている"
    );
}

/// 有利側の入力欄が読めていない機会も分母に入れない。
#[test]
fn the_advantaged_sides_input_display_has_to_be_readable_too() {
    let (ms, contacts, mut segs, rounds) = fixture();
    // 守備側だけが読めている。
    segs[0] = vec![idle_input(110, 140)];

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert!(extracted.advantages.is_empty());
    assert_eq!(extracted.situations.len(), 1, "守備側まで落としている");
}

/// 有利のうちに技を始めていれば、攻めを継続している。
#[test]
fn starting_a_move_within_the_advantage_is_continuing_the_pressure() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140), minus_press(115)];
    ms[1][116] = MeterState::Startup;

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.advantages.len(), 1);
    let advantage = &extracted.advantages[0];
    assert_eq!(advantage.outcome, AdvantageOutcome::Continued);
    assert_eq!(advantage.action_frame, Some(116));
    assert_eq!(advantage.follow_up, Some(PressureFollowUp::Strike));
}

/// 投げで攻めを継続した場合は、打撃と区別して記録する。
#[test]
fn continuing_with_a_throw_is_recorded_as_a_throw() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    let mut throw = minus_press(115);
    throw.throw = true;
    segs[1] = vec![idle_input(110, 140), throw];
    ms[1][116] = MeterState::Startup;

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(
        extracted.advantages[0].follow_up,
        Some(PressureFollowUp::Throw)
    );
}

/// 相手が動けるようになった後で始めた技は、有利を使ったことにならない。
#[test]
fn a_move_started_after_the_advantage_ran_out_is_not_continuing() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140), minus_press(122)];
    ms[1][123] = MeterState::Startup;

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.advantages.len(), 1);
    assert_eq!(
        extracted.advantages[0].action_frame, None,
        "有利の外で始めた技を継続にしている"
    );
}

/// 攻めずに終わり、相手も攻めてこなければ仕切り直し。
#[test]
fn not_attacking_without_being_attacked_is_a_reset() {
    let (ms, contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140)];

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.advantages.len(), 1);
    assert_eq!(extracted.advantages[0].outcome, AdvantageOutcome::Reset);
    assert_eq!(extracted.advantages[0].drop, 0.0);
}

/// 攻守が入れ替わったかは、決まった時間の内側だけで見る。ずっと後の
/// 攻撃までターンを返した結果に数えると、無関係な被弾が付く。
#[test]
fn the_turn_only_counts_as_lost_inside_the_result_window() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140)];
    contacts.push(ContactEvent {
        frame: 161,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    });

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(
        extracted.advantages[0].outcome,
        AdvantageOutcome::Reset,
        "窓の外の攻撃でターンを返したことにしている"
    );
}

/// 自分が当てた攻撃は、ターンを返した証拠にならない。
#[test]
fn your_own_hit_does_not_mean_you_lost_the_turn() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140)];
    contacts.push(ContactEvent {
        frame: 130,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    });

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.advantages[0].outcome, AdvantageOutcome::Reset);
}

// ── ゲーム内の時間で不利を測る ───────────────────────────────────────────
//
// 動画のフレーム番号とゲーム内のフレーム番号は一致しない。ヒットストップ中は
// メーターが止まるので、動画では 20 フレーム経っていてもゲーム内では 12 しか
// 進んでいない、ということが起きる。不利幅はゲーム内の時間で決まる。

/// 動画のフレームに対して、指定の位置から一定量だけ止まったゲームフレーム。
fn game_frames_stalled_at(length: usize, stall_from: usize, stall: i64) -> Vec<i64> {
    (0..length as i64)
        .map(|frame| {
            if frame as usize >= stall_from {
                frame - stall
            } else {
                frame
            }
        })
        .collect()
}

/// 不利幅はゲーム内の時間で測る。動画の差で測ると、演出で止まった分まで
/// 不利に数える。
#[test]
fn the_minus_is_measured_in_game_frames() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    let length = ms[0].len();
    let epochs = [vec![0; length], vec![0; length]];
    // 自分側のメーターだけが f116 以降 3 フレーム止まっていた。
    let game_frames = [
        game_frames_stalled_at(length, 116, 3),
        (0..length as i64).collect::<Vec<_>>(),
    ];

    let extracted = extract_minus_with(&ms, &epochs, &game_frames, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1);
    assert_eq!(
        extracted.situations[0].minus_frames, 2,
        "動画の差をそのまま不利幅にしている"
    );
}

/// ゲーム内の時間が読めていなければ、動画の差で代用する。読めない
/// フレームで場面ごと落とすと、記録が虫食いになる。
#[test]
fn an_unreadable_game_frame_falls_back_to_the_video_difference() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    let length = ms[0].len();
    let epochs = [vec![0; length], vec![0; length]];
    let mut game_frames = [
        (0..length as i64).collect::<Vec<_>>(),
        (0..length as i64).collect::<Vec<_>>(),
    ];
    game_frames[0][120] = -1;

    let extracted = extract_minus_with(&ms, &epochs, &game_frames, &contacts, &[], &segs, &rounds);

    assert_eq!(
        extracted.situations.len(),
        1,
        "読めない時刻で場面を捨てている"
    );
    assert_eq!(extracted.situations[0].minus_frames, 5);
}

/// ゲーム内の時間が逆転していれば信用しない。読み違いをそのまま
/// 使うと、不利幅が桁違いになる。
#[test]
fn a_game_frame_ordering_that_cannot_be_true_falls_back() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    let length = ms[0].len();
    let epochs = [vec![0; length], vec![0; length]];
    let mut game_frames = [
        (0..length as i64).collect::<Vec<_>>(),
        (0..length as i64).collect::<Vec<_>>(),
    ];
    // 自分が動けるようになった時刻の方が、相手より前になっている。
    game_frames[0][120] = 100;

    let extracted = extract_minus_with(&ms, &epochs, &game_frames, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.situations.len(), 1);
    assert_eq!(
        extracted.situations[0].minus_frames, 5,
        "逆転した時刻を使っている"
    );
}

// ── メーターの読みが途切れた場面 ─────────────────────────────────────────

/// 接触の時点でメーターを読めていなければ、そこから何も測れない。
#[test]
fn a_contact_on_an_unreadable_meter_opens_nothing() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    let length = ms[0].len();
    let mut epochs = [vec![0; length], vec![0; length]];
    epochs[0][100] = -1;

    let extracted = extract_minus_with(
        &ms,
        &epochs,
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );

    assert!(extracted.situations.is_empty());
}

/// 二人のメーターが別々の区間に属していれば、時間を比べられない。
#[test]
fn two_meters_from_different_epochs_cannot_be_compared() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    let length = ms[0].len();
    let epochs = [vec![0; length], vec![1; length]];

    let extracted = extract_minus_with(
        &ms,
        &epochs,
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );

    assert!(extracted.situations.is_empty());
}

/// 結果を見る窓の途中で読みが途切れていれば、結果の確度を下げる。
/// 途切れた先で何が起きたかは分からない。
#[test]
fn a_break_inside_the_result_window_lowers_the_confidence() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    let length = ms[0].len();

    let whole = extract_minus_with(
        &ms,
        &[vec![0; length], vec![0; length]],
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );
    assert_eq!(whole.presses.len(), 1);
    assert_eq!(whole.presses[0].confidence, EventConfidence::High);

    let mut broken = [vec![0; length], vec![0; length]];
    for epoch in broken[0].iter_mut().skip(140) {
        *epoch = 1;
    }
    let partial = extract_minus_with(
        &ms,
        &broken,
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );

    assert_eq!(partial.presses.len(), 1, "途切れで場面ごと捨てている");
    assert_eq!(
        partial.presses[0].confidence,
        EventConfidence::Medium,
        "途切れたのに確度を下げていない"
    );
}

/// 相手側の読みが途切れた場合も同じ。
#[test]
fn a_break_on_the_opponents_meter_lowers_it_too() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    let length = ms[0].len();
    let mut broken = [vec![0; length], vec![0; length]];
    for epoch in broken[1].iter_mut().skip(140) {
        *epoch = 1;
    }

    let extracted = extract_minus_with(
        &ms,
        &broken,
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );

    assert_eq!(extracted.presses[0].confidence, EventConfidence::Medium);
}

/// 動画と同じ長さのゲームフレーム。止まりも欠測も無い。
fn default_game_frames(length: usize) -> [Vec<i64>; 2] {
    [(0..length as i64).collect(), (0..length as i64).collect()]
}

// ── 押した結果 ───────────────────────────────────────────────────────────

/// 押した技が潰されて被弾していれば、狩られたことになる。
#[test]
fn getting_hit_while_the_move_runs_is_a_counter_hit() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    contacts.push(ContactEvent {
        frame: 126,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    });
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 126,
        pre_freeze_frame: 126,
        end_frame: 150,
        hp_before: 1.0,
        hp_after: 0.88,
        drop: 0.12,
        round_no: 1,
    }];

    let extracted = extract_minus_all(&ms, &contacts, &damage, &segs, &rounds);

    assert_eq!(extracted.presses.len(), 1);
    assert_eq!(extracted.presses[0].outcome, MinusPressOutcome::CounterHit);
    assert!((extracted.presses[0].drop - 0.12).abs() < 1e-6);
}

/// 技が終わった後の被弾は、その技が潰された結果ではない。
#[test]
fn getting_hit_after_the_move_ended_is_not_a_counter_hit() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    contacts.push(ContactEvent {
        frame: 145,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    });

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.presses.len(), 1);
    assert_eq!(
        extracted.presses[0].outcome,
        MinusPressOutcome::GotAway,
        "技の外の被弾を潰されたことにしている"
    );
}

/// 結果を見る窓の外の被弾も数えない。
#[test]
fn a_hit_outside_the_result_window_does_not_decide_the_outcome() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    contacts.push(ContactEvent {
        frame: 151,
        attacker: 2,
        victim: 1,
        hit: true,
        projectile: false,
        round_no: 1,
    });

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.presses[0].outcome, MinusPressOutcome::GotAway);
}

/// 押した技が当たっていれば、押し勝っている。
#[test]
fn landing_the_move_is_winning_the_exchange() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    contacts.push(ContactEvent {
        frame: 126,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    });

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.presses[0].outcome, MinusPressOutcome::Won);
}

/// 接触の記録が無くても、技の最中に HP が減っていれば潰されている。
/// ただし接触で確かめたときより確度は落ちる。
#[test]
fn health_lost_during_the_move_is_a_counter_hit_with_less_confidence() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));
    let damage = vec![DamageEvent {
        victim: 1,
        start_frame: 126,
        pre_freeze_frame: 126,
        end_frame: 150,
        hp_before: 1.0,
        hp_after: 0.88,
        drop: 0.12,
        round_no: 1,
    }];

    let extracted = extract_minus_all(&ms, &contacts, &damage, &segs, &rounds);

    assert_eq!(extracted.presses[0].outcome, MinusPressOutcome::CounterHit);
    assert_eq!(extracted.presses[0].confidence, EventConfidence::Medium);
}

/// 何も起きなければ、押して逃げ切っている。
#[test]
fn nothing_happening_means_the_press_got_away() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    segs[0].push(minus_press(120));

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.presses[0].outcome, MinusPressOutcome::GotAway);
    assert_eq!(extracted.presses[0].drop, 0.0);
}

// ── 押した入力の名前 ─────────────────────────────────────────────────────

/// 投げは投げとして記録する。打撃と混ぜると、回答の内訳が壊れる。
#[test]
fn a_throw_is_labelled_as_a_throw() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    let mut throw = minus_press(120);
    throw.throw = true;
    segs[0].push(throw);

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.presses[0].action_kind, DefensiveActionKind::Throw);
    assert_eq!(extracted.presses[0].pressed, "投げ");
}

/// バッジのある入力は、そのバッジをそのまま名前にする。
#[test]
fn a_badged_input_keeps_its_badges() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    let mut press = minus_press(120);
    press.badges = vec!["中P".to_string(), "中K".to_string()];
    segs[0].push(press);

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.presses[0].pressed, "中P+中K");
}

/// 自動入力にも名前を付ける。空欄では何を押したのか伝わらない。
#[test]
fn an_automatic_input_still_gets_a_name() {
    let (ms, contacts, mut segs, rounds) = fixture();
    observed(&mut segs, 0);
    let mut press = minus_press(120);
    press.badges = vec![];
    press.auto = true;
    segs[0].push(press);

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.presses.len(), 1);
    assert_eq!(extracted.presses[0].pressed, "AUTO");
}

// ── 有利側の確度 ─────────────────────────────────────────────────────────

/// 攻めなかった結果を見る窓の途中で読みが途切れていれば、有利側の
/// 記録も確度を下げる。その先で攻め返されたかどうかが分からない。
#[test]
fn a_break_inside_the_advantage_window_lowers_its_confidence() {
    let (ms, contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140)];
    let length = ms[0].len();

    let whole = extract_minus_with(
        &ms,
        &[vec![0; length], vec![0; length]],
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );
    assert_eq!(whole.advantages.len(), 1);
    assert_eq!(whole.advantages[0].confidence, EventConfidence::High);

    let mut broken = [vec![0; length], vec![0; length]];
    for epoch in broken[1].iter_mut().skip(150) {
        *epoch = 1;
    }
    let partial = extract_minus_with(
        &ms,
        &broken,
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );

    assert_eq!(partial.advantages.len(), 1, "途切れで場面ごと捨てている");
    assert_eq!(
        partial.advantages[0].confidence,
        EventConfidence::Medium,
        "途切れたのに確度を下げていない"
    );
}

/// 守備側の読みが途切れた場合も、有利側の確度を下げる。攻め返された
/// かどうかは両者の表示が要る。
#[test]
fn a_break_on_the_defending_side_lowers_the_advantage_confidence_too() {
    let (ms, contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140)];
    let length = ms[0].len();
    let mut broken = [vec![0; length], vec![0; length]];
    for epoch in broken[0].iter_mut().skip(150) {
        *epoch = 1;
    }

    let extracted = extract_minus_with(
        &ms,
        &broken,
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );

    assert_eq!(extracted.advantages[0].confidence, EventConfidence::Medium);
}

/// 攻めを継続した場合は、結果を待つ必要がない。窓の先が途切れていても
/// 確度は下げない。
#[test]
fn continuing_the_pressure_does_not_need_the_result_window() {
    let (mut ms, contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140), minus_press(115)];
    ms[1][116] = MeterState::Startup;
    let length = ms[0].len();
    let mut broken = [vec![0; length], vec![0; length]];
    for epoch in broken[1].iter_mut().skip(150) {
        *epoch = 1;
    }

    let extracted = extract_minus_with(
        &ms,
        &broken,
        &default_game_frames(length),
        &contacts,
        &[],
        &segs,
        &rounds,
    );

    assert_eq!(extracted.advantages[0].outcome, AdvantageOutcome::Continued);
    assert_eq!(extracted.advantages[0].confidence, EventConfidence::High);
}

/// 同じ機会を二度記録しない。接触が二つ重なっても、有利は一つ。
#[test]
fn one_advantage_is_recorded_once_per_moment() {
    let (ms, mut contacts, mut segs, rounds) = fixture();
    segs[1] = vec![idle_input(110, 140)];
    contacts.push(ContactEvent {
        frame: 101,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    });

    let extracted = extract_minus_all(&ms, &contacts, &[], &segs, &rounds);

    assert_eq!(extracted.advantages.len(), 1, "同じ有利を二度数えている");
}
