pub(super) use super::super::*;
pub(super) use crate::input_history::BadgeMark;

pub(crate) fn feat(i: u32, l: f32, r: f32) -> FrameFeatures {
    FrameFeatures {
        frame_index: i,
        fps: 60.0,
        own_hp: l,
        opponent_hp: r,
        is_match_screen: true,
        own_meter_state: None,
        opponent_meter_state: None,
        left_hp_score: 0.1,
        right_hp_score: 0.1,
        left_drive_ratio: 1.0,
        right_drive_ratio: 1.0,
        left_burnout: false,
        right_burnout: false,
        left_drive_uncertain: false,
        right_drive_uncertain: false,
        left_super_value: 0.0,
        right_super_value: 0.0,
        left_super_uncertain: true,
        right_super_uncertain: true,
        left_ca_ready: false,
        right_ca_ready: false,
        left_hp_raw: l,
        right_hp_raw: r,
        left_hp_raw_quality: 0.0,
        right_hp_raw_quality: 0.0,
    }
}

pub(super) fn tracked(
    count: u32,
    dir: InputDir,
    badges: Vec<BadgeMark>,
    auto: bool,
    throw: bool,
) -> TrackedInput {
    TrackedInput {
        count: Some(count),
        dir,
        badges,
        auto,
        throw,
        repaired: false,
        uncertain: false,
    }
}

/// 2 ラウンドの合成試合: R1 は P2 が KO 負け、R2 は P1 が KO 負け
pub(super) fn synth_two_rounds() -> Vec<FrameFeatures> {
    let mut fs = Vec::new();
    let mut i = 0u32;
    // R1: 全快 100f → P2 が 3 回被弾して 0 に
    for _ in 0..100 {
        fs.push(feat(i, 1.0, 1.0));
        i += 1;
    }
    for k in 0..30 {
        fs.push(feat(i, 1.0, 1.0 - 0.01 * k as f32));
        i += 1;
    } // -0.3
    for _ in 0..100 {
        fs.push(feat(i, 1.0, 0.7));
        i += 1;
    }
    for k in 0..30 {
        fs.push(feat(i, 1.0, 0.7 - 0.015 * k as f32));
        i += 1;
    } // -0.45
    for _ in 0..100 {
        fs.push(feat(i, 1.0, 0.25));
        i += 1;
    }
    for k in 0..25 {
        fs.push(feat(i, 1.0, (0.25 - 0.01 * k as f32).max(0.0)));
        i += 1;
    }
    for _ in 0..80 {
        fs.push(feat(i, 1.0, 0.0));
        i += 1;
    } // KO
      // R2: 全快 100f → P1 が一気に 0 に
    for _ in 0..100 {
        fs.push(feat(i, 1.0, 1.0));
        i += 1;
    }
    for k in 0..50 {
        fs.push(feat(i, (1.0 - 0.02 * k as f32).max(0.0), 1.0));
        i += 1;
    }
    for _ in 0..80 {
        fs.push(feat(i, 0.0, 1.0));
        i += 1;
    }
    fs
}

/// P2 視点の3ラウンド合成試合。勝者は P2, P1, P2 の順。
pub(super) fn synth_three_rounds_for_p2() -> Vec<FrameFeatures> {
    let mut features = Vec::new();
    let mut frame = 0u32;

    for winner in [2u8, 1, 2] {
        for _ in 0..30 {
            features.push(feat(frame, 1.0, 1.0));
            frame += 1;
        }
        for step in 1..=20 {
            let loser_hp = (1.0 - 0.06 * step as f32).max(0.0);
            let (p2_hp, p1_hp) = if winner == 2 {
                (1.0, loser_hp)
            } else {
                (loser_hp, 1.0)
            };
            features.push(feat(frame, p2_hp, p1_hp));
            frame += 1;
        }
        for _ in 0..20 {
            let (p2_hp, p1_hp) = if winner == 2 { (1.0, 0.0) } else { (0.0, 1.0) };
            features.push(feat(frame, p2_hp, p1_hp));
            frame += 1;
        }
    }

    features
}

pub(super) fn synth_timeline(entries: Vec<(i64, &str, i64, i64)>) -> MeterTimeline {
    MeterTimeline {
        side: "test".to_string(),
        segments: vec![meter_tracker::TimelineSegment {
            segment_id: 0,
            entries: entries
                .into_iter()
                .map(|(gf, st, a, b)| meter_tracker::TimelineEntry {
                    game_frame: gf,
                    state: st.to_string(),
                    video_frame_first: a,
                    video_frame_last: b,
                    confidence: 1.0,
                })
                .collect(),
        }],
    }
}

pub(super) fn synth_segmented_timeline(
    segment_id: i32,
    entries: Vec<(i64, &str, i64, i64)>,
) -> MeterTimeline {
    MeterTimeline {
        side: "test".to_string(),
        segments: vec![meter_tracker::TimelineSegment {
            segment_id,
            entries: entries
                .into_iter()
                .map(|(gf, st, a, b)| meter_tracker::TimelineEntry {
                    game_frame: gf,
                    state: st.to_string(),
                    video_frame_first: a,
                    video_frame_last: b,
                    confidence: 1.0,
                })
                .collect(),
        }],
    }
}

/// 1 gf = 1 エントリの現実的なランを合成する（gf は 1 ずつ進む）。
/// 有利フレームの game frame 計上を正しく通すために使う。
pub(super) fn synth_run(gf0: i64, st: &str, a: i64, b: i64) -> Vec<(i64, &str, i64, i64)> {
    (0..=(b - a)).map(|k| (gf0 + k, st, a + k, a + k)).collect()
}

pub(super) fn extract_synth_punishes(
    base_frame: u32,
    p1: Vec<MeterState>,
    p2: Vec<MeterState>,
    contacts: Vec<ContactEvent>,
) -> Vec<PunishChance> {
    let n = p1.len();
    assert_eq!(p2.len(), n);
    let features: Vec<_> = (0..n)
        .map(|index| feat(base_frame + index as u32, 1.0, 1.0))
        .collect();
    let game_frames: Vec<i64> = (0..n as i64).collect();
    let epochs = [vec![0; n], vec![0; n]];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: base_frame,
        end_frame: base_frame + n as u32 - 1,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    super::punishes::extract_punishes(super::punishes::PunishInputs {
        features: &features,
        meter_state: &[p1, p2],
        meter_epoch: &epochs,
        meter_game_frame: &[game_frames.clone(), game_frames],
        contacts: &contacts,
        damage: &[],
        segments: &[vec![], vec![]],
        rounds: &rounds,
    })
}

pub(super) fn up_inputs(n: usize, ranges: &[(usize, usize)]) -> Vec<TrackedInput> {
    let mut inputs: Vec<_> = (0..n)
        .map(|k| tracked((k + 1) as u32, InputDir::Neutral, vec![], false, false))
        .collect();
    for &(start, end) in ranges {
        for (frame, input) in inputs.iter_mut().enumerate().take(end + 1).skip(start) {
            *input = tracked(
                (frame - start + 1) as u32,
                InputDir::UpRight,
                vec![],
                false,
                false,
            );
        }
    }
    inputs
}

pub(super) type MinusPressFixture = (
    [Vec<MeterState>; 2],
    Vec<ContactEvent>,
    [Vec<InputSegment>; 2],
    Vec<RoundInfo>,
);

pub(super) fn minus_press_fixture() -> MinusPressFixture {
    use MeterState::*;
    let n = 300usize;
    let mut own = vec![Free; n]; // P1（自分）
    let mut opp = vec![Free; n]; // P2（相手）
                                 // f100 ガード接触: 自分は f100..120 ガード硬直（20F）
    for s in own.iter_mut().take(120).skip(100) {
        *s = Stun;
    }
    // 硬直明け最速で実際に技が発生する。
    for s in own.iter_mut().take(124).skip(120) {
        *s = Startup;
    }
    for s in own.iter_mut().take(130).skip(124) {
        *s = Active;
    }
    // 相手は f95..100 発生 → f100..105 持続 → f105..115 通常後隙
    for s in opp.iter_mut().take(100).skip(95) {
        *s = Startup;
    }
    for s in opp.iter_mut().take(105).skip(100) {
        *s = Active;
    }
    for s in opp.iter_mut().take(115).skip(105) {
        *s = MotionRecovery;
    }
    // → t_own=120, t_opp=115, 不利 5F
    let contacts = vec![ContactEvent {
        frame: 100,
        attacker: 2,
        victim: 1,
        hit: false,
        projectile: false,
        round_no: 1,
    }];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 299,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    ([own, opp], contacts, [vec![], vec![]], rounds)
}

/// ボタンを含まない、直接観測できた入力区間。
/// 「入力欄は読めていたが攻撃はしていない」機会を作るために使う。
pub(super) fn idle_input(start: u32, end: u32) -> InputSegment {
    InputSegment {
        start_frame: start,
        end_frame: end,
        dir: "N".to_string(),
        badges: vec![],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }
}

pub(super) fn minus_press(f: u32) -> InputSegment {
    InputSegment {
        start_frame: f,
        end_frame: f + 4,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    }
}

pub(super) fn extract_minus(
    meter_state: &[Vec<MeterState>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    segments: &[Vec<InputSegment>; 2],
    rounds: &[RoundInfo],
) -> Vec<MinusPressEvent> {
    let n = meter_state[0].len();
    let epochs = [vec![0; n], vec![0; n]];
    let game_frames = [
        (0..n as i64).collect::<Vec<_>>(),
        (0..n as i64).collect::<Vec<_>>(),
    ];
    extract_presses_while_minus(
        meter_state,
        &epochs,
        &game_frames,
        contacts,
        damage,
        segments,
        rounds,
    )
}

pub(super) fn extract_minus_all(
    meter_state: &[Vec<MeterState>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    segments: &[Vec<InputSegment>; 2],
    rounds: &[RoundInfo],
) -> MinusEvents {
    let n = meter_state[0].len();
    let epochs = [vec![0; n], vec![0; n]];
    let game_frames = [
        (0..n as i64).collect::<Vec<_>>(),
        (0..n as i64).collect::<Vec<_>>(),
    ];
    extract_minus_events(
        meter_state,
        &epochs,
        &game_frames,
        contacts,
        damage,
        segments,
        rounds,
    )
}
