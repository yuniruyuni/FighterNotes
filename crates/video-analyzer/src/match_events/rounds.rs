//! HP 全快持続によるラウンド分割
//!
//! match_events.rs からの機械的分割（挙動不変）。

use super::*;

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
    {
        let mut i = 0usize;
        while i < n {
            if full(i) {
                let start = i;
                while i < n && full(i) {
                    i += 1;
                }
                if i - start >= FULL_MIN_RUN {
                    onsets.push(start);
                }
            } else {
                i += 1;
            }
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
        let min_hp = (prev..o)
            .map(|i| hp[0][i].min(hp[1][i]))
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

    // 各 onset から次の onset（または末尾）までをラウンドとし、KO で終端を締める
    let mut rounds: Vec<RoundInfo> = Vec::new();
    for (k, &o) in merged.iter().enumerate() {
        let hard_end = if k + 1 < merged.len() {
            merged[k + 1] - 1
        } else {
            n - 1
        };
        // KO 探索: どちらかが KO_HP 以下を KO_MIN_RUN 持続する最初の位置
        let mut end = hard_end;
        let mut winner: Option<u8> = None;
        let mut hp_end = (hp[0][hard_end], hp[1][hard_end]);
        let mut i = o;
        while i + KO_MIN_RUN <= hard_end {
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
            i += 1;
        }
        // KO が検出できないラウンド（KO フラッシュで最後の一撃が uncertain に
        // なった / タイムアップ / 動画途中で終了）は、両者の読み取り品質が
        // 良かった最後のフレームまで巻き戻し、その時点の HP 差で勝敗を推定する
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
            end = end_stable;
            hp_end = (hp[0][end_stable], hp[1][end_stable]);
            let (l, r) = hp_end;
            if (l - r).abs() > WINNER_HP_MARGIN {
                winner = Some(if l > r { 1 } else { 2 });
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
