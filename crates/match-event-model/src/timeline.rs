use super::*;

/// 両者のメーターが同時に長時間停止しているスパン（演出フリーズ）を列挙する。
///
/// SA 暗転・投げ演出中はゲームが止まり、メーターの 1 エントリが
/// FREEZE_MIN_DWELL 以上の video frame にまたがる。両側のそのような
/// スパンの重なりを取り、近接するものを連結して返す（video frame, 昇順）。
pub fn both_freeze_spans(left: &MeterTimeline, right: &MeterTimeline) -> Vec<(u32, u32)> {
    let long_spans = |tl: &MeterTimeline| -> Vec<(u32, u32)> {
        let mut v: Vec<(u32, u32)> = tl
            .segments
            .iter()
            .flat_map(|seg| seg.entries.iter())
            .filter(|e| {
                e.video_frame_first >= 0
                    && e.video_frame_last - e.video_frame_first + 1 >= FREEZE_MIN_DWELL
            })
            .map(|e| (e.video_frame_first as u32, e.video_frame_last as u32))
            .collect();
        v.sort_unstable();
        v
    };
    let ls = long_spans(left);
    let rs = long_spans(right);
    // 両側の重なりを取る
    let mut both: Vec<(u32, u32)> = Vec::new();
    for &(a1, b1) in &ls {
        for &(a2, b2) in &rs {
            let a = a1.max(a2);
            let b = b1.min(b2);
            if a <= b {
                both.push((a, b));
            }
        }
    }
    both.sort_unstable();
    // 近接スパンの連結（SA 暗転 → 演出本体のような分割を 1 つに）
    let mut merged: Vec<(u32, u32)> = Vec::new();
    for (a, b) in both {
        match merged.last_mut() {
            Some((_, pb)) if a <= *pb + FREEZE_MERGE_GAP => *pb = (*pb).max(b),
            _ => merged.push((a, b)),
        }
    }
    merged
}

/// タイムラインを「video frame → game frame」の写像に展開する（未カバーは -1）。
///
/// ヒットストップ・演出フリーズ中はメーターが停止し、1 つの game frame が
/// 複数の video frame にまたがる。フレーム有利の計算は video frame の
/// 引き算ではなくこの写像で game frame を数える必要がある
/// （フィードバック③: 停止が挟まると実 +2 が +12 と過大表示された）。
pub fn gf_per_frame(tl: &MeterTimeline, n: usize) -> Vec<i64> {
    let mut out = vec![-1i64; n];
    for seg in &tl.segments {
        for e in &seg.entries {
            if e.video_frame_first < 0 {
                continue;
            }
            let a = e.video_frame_first.max(0) as usize;
            let b = ((e.video_frame_last.max(0) as usize) + 1).min(n);
            for slot in &mut out[a.min(b)..b] {
                *slot = e.game_frame;
            }
        }
    }
    out
}

/// ビデオフレームごとのメータートラッカー区間ID（未観測は -1）。
/// フレームメーターのリセットをまたぐ状態同士を因果付けないために使う。
pub fn epoch_per_frame(tl: &MeterTimeline, n: usize) -> Vec<i32> {
    let mut out = vec![-1i32; n];
    for segment in &tl.segments {
        for entry in &segment.entries {
            if entry.video_frame_first < 0 {
                continue;
            }
            let start = entry.video_frame_first as usize;
            let end = ((entry.video_frame_last.max(0) as usize) + 1).min(n);
            for slot in &mut out[start.min(end)..end] {
                *slot = segment.segment_id;
            }
        }
    }
    out
}

pub fn continuous_epoch(epochs: &[i32], start: usize, end: usize) -> Option<i32> {
    if epochs.is_empty() || start >= epochs.len() || end >= epochs.len() || start > end {
        return None;
    }
    let epoch = epochs[start];
    (epoch >= 0 && epochs[start..=end].iter().all(|value| *value == epoch)).then_some(epoch)
}

/// 移動系（ジャンプ・ダッシュ等）の表示があるフレームの真偽列。
///
/// SF6 のフレームメーターはジャンプの空中フレームをシアン（motion）で、
/// 予備動作や一部キャラのジャンプを緑（counter）で表示する（検証済み試合:
/// ブランカ通常ジャンプ = counter 連鎖 29gf / ダルシム側 = motion 38gf+）。
/// 上入力ベースのジャンプ検出の確認証拠として使う。
pub fn movementish_per_frame(tl: &MeterTimeline, n: usize) -> Vec<bool> {
    let mut out = vec![false; n];
    for seg in &tl.segments {
        for e in &seg.entries {
            if !matches!(e.state.as_str(), "counter" | "motion_recovery") {
                continue;
            }
            if e.video_frame_first < 0 {
                continue;
            }
            let a = e.video_frame_first.max(0) as usize;
            let b = ((e.video_frame_last.max(0) as usize) + 1).min(n);
            for slot in &mut out[a.min(b)..b] {
                *slot = true;
            }
        }
    }
    out
}

/// A long `counter` run can be either jump movement or ordinary move startup.
/// When it is directly sandwiched between two Active states in one meter epoch,
/// it belongs to a grounded attack chain and cannot independently confirm takeoff.
pub fn movement_run_is_ground_attack_chain(
    states: &[MeterState],
    epochs: &[i32],
    start: usize,
    end: usize,
) -> bool {
    if states.get(start) != Some(&MeterState::Startup) {
        return false;
    }
    let Some(before) = start.checked_sub(1) else {
        return false;
    };
    if states.get(before) != Some(&MeterState::Active) {
        return false;
    }
    let search_end = end.saturating_add(3).min(states.len().saturating_sub(1));
    let Some(after) = (end.saturating_add(1)..=search_end)
        .find(|&index| states.get(index) == Some(&MeterState::Active))
    else {
        return false;
    };
    continuous_epoch(epochs, before, after).is_some()
}

/// タイムラインをフレームごとの粗い状態列に展開する。
pub fn state_per_frame(tl: &MeterTimeline, n: usize) -> Vec<MeterState> {
    let mut out = vec![MeterState::Free; n];
    for seg in &tl.segments {
        for e in &seg.entries {
            let st = meter_state_from_label(&e.state);
            if st == MeterState::Free {
                continue;
            }
            let a = e.video_frame_first.max(0) as usize;
            let b = ((e.video_frame_last.max(0) as usize) + 1).min(n);
            for slot in &mut out[a.min(b)..b] {
                *slot = st;
            }
        }
    }
    out
}

fn meter_state_from_label(label: &str) -> MeterState {
    match label {
        "counter" => MeterState::Startup,
        "inv_full" | "inv_strike" | "inv_proj" => MeterState::Invincible,
        "active" => MeterState::Active,
        "projectile_active" => MeterState::ProjectileActive,
        "parry" => MeterState::Parry,
        "motion_recovery" => MeterState::MotionRecovery,
        "punish_counter" => MeterState::Recovery,
        "stun" => MeterState::Stun,
        _ => MeterState::Free,
    }
}

/// 非 Free 状態と対になる読取信頼度を動画フレームへ展開する。
/// 重複時は state_per_frame と同じ走査順で上書きし、別状態の確度を混ぜない。
pub fn confidence_per_frame(tl: &MeterTimeline, n: usize) -> Vec<f32> {
    let mut out = vec![0.0_f32; n];
    for segment in &tl.segments {
        for entry in &segment.entries {
            if meter_state_from_label(&entry.state) == MeterState::Free {
                continue;
            }
            let a = entry.video_frame_first.max(0) as usize;
            let b = ((entry.video_frame_last.max(0) as usize) + 1).min(n);
            for slot in &mut out[a.min(b)..b] {
                *slot = entry.confidence as f32;
            }
        }
    }
    out
}

/// フレーム番号が属するラウンドを返す。
pub fn round_of(rounds: &[RoundInfo], frame: u32) -> Option<u32> {
    rounds
        .iter()
        .find(|r| frame >= r.start_frame && frame <= r.end_frame)
        .map(|r| r.round_no)
}

/// features 内で frame_index == frame となる位置（見つからなければ近似）。
pub fn idx_of(features: &[FrameFeatures], frame: u32) -> usize {
    // frame_index は通常 0..n の連番。二分探索で頑健に
    features
        .binary_search_by_key(&frame, |f| f.frame_index)
        .unwrap_or_else(|i| i.min(features.len().saturating_sub(1)))
}
