//! 守り側の抽出を直接呼ぶテスト補助。
//!
//! 観測列の組み立て自体は `match-event-model` の test-support が持つ。
//! ここにあるのは、その観測列をこの crate の抽出器へ通すところだけ。

pub use match_event_model::test_support::*;

use crate::minus_press::{extract_minus_events, MinusEvents};

pub fn extract_synth_punishes(
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
    crate::punishes::extract_punishes(crate::punishes::PunishInputs {
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

pub fn extract_minus(
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
    crate::minus_press::extract_presses_while_minus(
        meter_state,
        &epochs,
        &game_frames,
        contacts,
        damage,
        segments,
        rounds,
    )
}

pub fn extract_minus_all(
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
