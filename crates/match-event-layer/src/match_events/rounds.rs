//! HP 全快持続によるラウンド分割
//!
//! match_events.rs からの機械的分割（挙動不変）。

use super::*;
use crate::round_start::FightMarker;

/// タイムアップや KO 演出でゼロ HP を読めない場合に、残 HP 差から勝者を
/// 推定する最小差。HP バー約 14px 相当で、通常の読み取り揺れより十分大きい。
const WINNER_HP_MARGIN: f32 = 0.02;

/// 両者全快の持続区間からラウンド境界を推定する。
pub(crate) fn detect_rounds_from_hp(
    features: &[FrameFeatures],
    hp: &[Vec<f32>; 2],
) -> Vec<RoundInfo> {
    let n = features.len();
    if n == 0 {
        return vec![];
    }

    // 全快 run（FULL_MIN_RUN 以上持続）の開始位置を収集
    let full = |i: usize| features[i].is_match_screen && hp[0][i] >= FULL_HP && hp[1][i] >= FULL_HP;
    let mut onsets: Vec<usize> = Vec::new();
    let mut run_start = None;
    for i in 0..n {
        if full(i) {
            run_start.get_or_insert(i);
        } else if let Some(start) = run_start.take() {
            if i - start >= FULL_MIN_RUN {
                onsets.push(start);
            }
        }
    }
    if let Some(start) = run_start {
        if n - start >= FULL_MIN_RUN {
            onsets.push(start);
        }
    }
    if onsets.is_empty() {
        return vec![];
    }

    // 遮蔽等による全快 run の分断をマージ:
    // 前の onset からこの onset までの間に実ダメージ（min HP < MERGE_MIN_HP）が
    // 無ければ同一ラウンドの開始前区間なので捨てる
    let mut merged: Vec<usize> = vec![onsets[0]];
    for &o in &onsets[1..] {
        let prev = *merged.last().unwrap();
        let min_hp = hp[0]
            .iter()
            .zip(&hp[1])
            .take(o)
            .skip(prev)
            .map(|(p1, p2)| p1.min(*p2))
            .fold(f32::MAX, f32::min);
        if min_hp < MERGE_MIN_HP {
            merged.push(o);
        } else {
            // 間に実ダメージが無い = まだ戦闘が始まっていない。
            // 実ラウンド開始により近い「後の全快 run」を採用する
            // （リプレイ冒頭のイントロ画面の全快バーを開始点にしない）
            *merged.last_mut().unwrap() = o;
        }
    }

    detect_rounds_from_bounds(features, hp, &merged, &merged)
}

/// `FIGHT` 画像の安定表示区間からラウンドを分割する。
///
/// 前ラウンドの hard end は次の FIGHT 検出開始直前、イベントを所属させる
/// start は安定表示の末尾とする。HP は境界の決定には使用しない。
pub(crate) fn detect_rounds_from_fight_markers(
    features: &[FrameFeatures],
    hp: &[Vec<f32>; 2],
    markers: &[FightMarker],
) -> Vec<RoundInfo> {
    if features.is_empty() {
        return Vec::new();
    }
    let starts: Vec<usize> = markers
        .iter()
        .map(|marker| idx_of(features, marker.last_frame))
        .collect();
    let boundaries: Vec<usize> = markers
        .iter()
        .map(|marker| idx_of(features, marker.first_frame))
        .collect();
    detect_rounds_from_bounds(features, hp, &starts, &boundaries)
}

/// 各 start から次の boundary（または末尾）までをラウンドとし、
/// KO・安定 HUD で終端を締める。
fn detect_rounds_from_bounds(
    features: &[FrameFeatures],
    hp: &[Vec<f32>; 2],
    starts: &[usize],
    boundaries: &[usize],
) -> Vec<RoundInfo> {
    if starts.len() != boundaries.len() {
        return Vec::new();
    }
    let n = features.len();
    let mut rounds: Vec<RoundInfo> = Vec::new();
    for (k, &o) in starts.iter().enumerate() {
        let hard_end = if k + 1 < boundaries.len() {
            boundaries[k + 1].saturating_sub(1)
        } else {
            n - 1
        };
        if o > hard_end {
            continue;
        }
        // KO 探索: どちらかが KO_HP 以下を KO_MIN_RUN 持続する最初の位置
        let mut end = hard_end;
        let mut winner: Option<u8> = None;
        let mut hp_end = (hp[0][hard_end], hp[1][hard_end]);
        let last_ko_start = hard_end
            .checked_add(1)
            .and_then(|length| length.checked_sub(KO_MIN_RUN));
        if let Some(last_ko_start) = last_ko_start {
            for i in o..=last_ko_start {
                let p1_ko = (i..i + KO_MIN_RUN).all(|j| hp[0][j] <= KO_HP);
                let p2_ko = (i..i + KO_MIN_RUN).all(|j| hp[1][j] <= KO_HP);
                if p1_ko || p2_ko {
                    hp_end = (hp[0][i], hp[1][i]); // 終了 HP は KO 確定時点で読む
                    end = (i + KO_MIN_RUN + 45).min(hard_end); // KO 演出を少し含める
                    winner = if p1_ko && !p2_ko {
                        Some(2)
                    } else if p2_ko && !p1_ko {
                        Some(1)
                    } else {
                        None
                    };
                    break;
                }
            }
        }
        // KO が検出できないラウンド（KO フラッシュで最後の一撃が uncertain に
        // なった / タイムアップ / 動画途中で終了）は、両者の読み取り品質が
        // 良かった最後のフレームを起点にする。
        if winner.is_none() {
            // 「安定」は連続 8 フレーム両者品質良好を要求する。リザルト画面には
            // 品質良好に見える孤立ジャンクフレームが混ざるため、単発では信用しない。
            // さらに raw と確定値の一致を要求する: 次ラウンド前の画面は全快バーを
            // 表示するが確定値は単調クランプされたままなので raw と乖離し、
            // ラウンド実体の終端と区別できる
            let stable = |i: usize| {
                features[i].is_match_screen
                    && features[i].left_hp_raw_quality <= 0.5
                    && features[i].right_hp_raw_quality <= 0.5
                    && (features[i].left_hp_raw - hp[0][i]).abs() <= 0.05
                    && (features[i].right_hp_raw - hp[1][i]).abs() <= 0.05
            };
            let end_stable = (o..=hard_end)
                .rev()
                .find(|&i| i >= o + 7 && (i - 7..=i).all(stable))
                .or_else(|| (o..=hard_end).rev().find(|&i| stable(i)))
                .unwrap_or(hard_end);
            // SA/CA 演出では片側 HP が長時間スプライトに覆われ、最後の安定
            // 読みがコンボ開始まで巻き戻ることがある。一度安定した試合 HUD
            // を確認できた後は、同じ連続した HUD 区間の終端までイベント範囲を
            // 保持する。リザルトや次ラウンド前の孤立 HUD は non-match 区間で
            // 分断されるため越えない。
            end = (end_stable..=hard_end)
                .take_while(|&i| features[i].is_match_screen)
                .last()
                .unwrap_or(end_stable);
            hp_end = (
                (o..=end).map(|i| hp[0][i]).fold(f32::MAX, f32::min),
                (o..=end).map(|i| hp[1][i]).fold(f32::MAX, f32::min),
            );
            let (l, r) = hp_end;
            if (l - r).abs() > WINNER_HP_MARGIN {
                winner = Some(if l.total_cmp(&r).is_gt() { 1 } else { 2 });
            }
        }
        rounds.push(RoundInfo {
            round_no: k as u32 + 1,
            start_frame: features[o].frame_index,
            end_frame: features[end].frame_index,
            winner,
            p1_hp_end: hp_end.0,
            p2_hp_end: hp_end.1,
        });
    }
    rounds
}

#[cfg(test)]
mod tests;
