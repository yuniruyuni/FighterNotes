//! HP バー遡及補正（後方パス + スタン検証 + スパイク検出）
//!
//! frame_features.rs からの機械的分割（挙動不変）。

use super::*;

// -------------------------------------------------------------------------
// HP 遡及補正
// -------------------------------------------------------------------------

/// HP バー遡及補正。全フレーム収集後（`finish()` 内）に呼び出す。
///
/// # アルゴリズム
///
/// **Phase 1 — 後方パス（誤検知の遡及修正）**
///
/// HP は試合中に増えることがない。そのため raw_hp[i] < max(raw_hp[i+1..]) なら
/// フレーム i の HP は誤検知（被覆等）と確定でき、後続の最大値で上書きする。
///
/// **Phase 2 — スタン検証（後方パスが捉えられない持続誤検知の修正）**
///
/// ダメージを受けた側のフレームメーターは必ずスタン（stun）状態になる。
/// HP 降下が検出されてもその側のスタン状態がなければ誤検知として前の信頼値を保持する。
///
/// **Phase 3 — 前方単調パス**
///
/// 最終的に単調非増加を保証する。
///
/// `left_stun[i]` / `right_stun[i]` は各ビデオフレームでの左右プレイヤーのスタン状態。
/// 空スライスが渡された場合は Phase 2 をスキップし Phase 1 のみ適用する。
pub fn correct_hp_retroactive(
    features: &mut [FrameFeatures],
    own_side: &str,
    left_stun: &[bool],
    right_stun: &[bool],
) {
    if features.is_empty() {
        return;
    }
    for hp_side in ["left", "right"] {
        let stun = if hp_side == "left" {
            left_stun
        } else {
            right_stun
        };
        correct_hp_side(features, own_side, hp_side, stun);
    }
}

pub(crate) fn correct_hp_side(
    features: &mut [FrameFeatures],
    own_side: &str,
    hp_side: &str,
    // Phase 2（スタン検証）は現行実装では未使用。将来の再有効化に備えて
    // シグネチャは維持する
    _stun: &[bool],
) {
    // 試合画面フレームのみを対象とするマスク（非試合フレームは補正対象外）
    let in_match: Vec<bool> = features.iter().map(|f| f.is_match_screen).collect();

    let raw_orig: Vec<f32> = features
        .iter()
        .map(|f| {
            if hp_side == "left" {
                f.left_hp_raw
            } else {
                f.right_hp_raw
            }
        })
        .collect();
    let raw = median_smoothed(&raw_orig, &in_match, MEDIAN_HALF);

    let mut corrected = raw.clone();

    // アイランド検出によって疑問フレームとマークされた列（quality > 0.5）
    let in_uncertain: Vec<bool> = features
        .iter()
        .map(|f| {
            let q = if hp_side == "left" {
                f.left_hp_raw_quality
            } else {
                f.right_hp_raw_quality
            };
            q > 0.5
        })
        .collect();

    let seg_starts = round_segments(&in_match);
    reset_round_starts(&mut corrected, &in_match);

    // ─── Phase 1: スパイクホールド前方パス ──────────────────────────────────
    // 以下の 3 種を「直前の確認済みHP」でホールドする:
    //   (A) スパイク（body overlap 等で raw HP が偽ハイ）
    //   (B) uncertain フレーム（白フラッシュ・完全遮蔽で HP バーが読めない）
    //   (C) 前フレーム HP から 50% 以上かつ絶対差 0.5 超の急落（爆発エフェクト等の偽ロー）
    //       ※ SF6 で 1 フレームに HP が 50% 以上下がることは通常ない
    //
    // 旧 backward_fill（「引き上げ」ベース）との根本的な違い:
    //   旧実装: raw[i] < max_future - EPSILON → max_future に引き上げ
    //     問題点: ダメージフレームも「未来に高い値がある」と引き上げられ、
    //             続く前方単調パスの prev として制約されて実ダメージが消える
    //   新実装: 偽フレームのみ prev でホールド、正常ダメージはそのまま通過
    let in_spike = compute_spike_frames(&raw, &in_match, &seg_starts);
    for w in seg_starts.windows(2) {
        spike_hold_forward_pass(
            &mut corrected,
            &in_match,
            &in_spike,
            &in_uncertain,
            w[0],
            w[1],
        );
    }

    monotone_forward_pass(&mut corrected, &in_match, &in_uncertain, &seg_starts);
    fill_unreadable_from_the_future(&mut corrected, &in_match, &in_uncertain);

    // own_hp / opponent_hp に書き戻す
    let is_left = hp_side == "left";
    let is_own = (own_side == "p1") == is_left;
    for (i, feat) in features.iter_mut().enumerate() {
        if is_own {
            feat.own_hp = corrected[i];
        } else {
            feat.opponent_hp = corrected[i];
        }
    }
}

/// 中央値をとる窓の片側の長さ。前後 2 フレームで計 5 フレーム。
const MEDIAN_HALF: usize = 2;

/// 前後 `half` フレームの中央値で読みを均す。
///
/// 残量が黄色域（25% 以下）に入ると 1 フレームの読みが ±5% 程度揺れる。
/// 揺れを残したまま単調制約をかけると、下振れした 1 フレームに以降が
/// 引きずられて残量が張り付く。
///
/// 本物のダメージは複数フレーム続くので、中央値では消えない。
/// 試合外のフレームは比較にも結果にも入れない。
pub(crate) fn median_smoothed(raw: &[f32], in_match: &[bool], half: usize) -> Vec<f32> {
    let mut smoothed = raw.to_vec();
    for index in 0..raw.len() {
        if !in_match[index] {
            continue;
        }
        let lo = index.saturating_sub(half);
        let hi = (index + half + 1).min(raw.len());
        let mut window: Vec<f32> = (lo..hi)
            .filter(|near| in_match[*near])
            .map(|near| raw[near])
            .collect();
        window.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // 自フレームが試合中なので、窓は必ず 1 要素以上ある。
        smoothed[index] = window[window.len() / 2];
    }
    smoothed
}

/// ラウンドの切れ目。返す並びは各ラウンドの開始位置に、終端を足したもの。
///
/// ラウンドの間には必ず勝敗表示や VS 画面が挟まるので、試合外から試合内へ
/// 変わったところが新しいラウンドの頭になる。残量の跳ね上がりで境目を
/// 探すと、体の重なりから戻った瞬間を新ラウンドと読んでしまう。
pub(crate) fn round_segments(in_match: &[bool]) -> Vec<usize> {
    let mut starts = vec![0usize];
    starts.extend((1..in_match.len()).filter(|index| in_match[*index] && !in_match[index - 1]));
    starts.push(in_match.len());
    starts
}

/// ラウンドの頭の残量を満タンに戻す。
///
/// ROUND!/FIGHT! の演出中はバーが読めず、頭のフレームが 0.99 前後になる。
/// この値を起点にすると、以降の残量がそこで頭打ちになって実ダメージが
/// 反映されない。
///
/// 動画の先頭から試合が映っている場合は、ラウンド途中からの録画かも
/// しれないので触らない。
pub(crate) fn reset_round_starts(corrected: &mut [f32], in_match: &[bool]) {
    for index in 1..in_match.len() {
        if in_match[index] && !in_match[index - 1] {
            corrected[index] = 1.0;
        }
    }
}

/// ラウンドの中で残量は増えない。増えている読みは直前の値まで押し下げる。
///
/// 読めなかった上にほぼ 0 のフレームは、演出でバーが消えただけ。基準に
/// 使うと 0 が以降のフレームすべてへ伝わる。
pub(crate) fn monotone_forward_pass(
    corrected: &mut [f32],
    in_match: &[bool],
    in_uncertain: &[bool],
    seg_starts: &[usize],
) {
    for w in seg_starts.windows(2) {
        let mut prev: Option<f32> = None;
        for index in w[0]..w[1] {
            if !in_match[index] {
                continue;
            }
            if in_uncertain[index] && corrected[index] < 0.01 {
                continue;
            }
            let value = prev.map_or(corrected[index], |p| corrected[index].min(p));
            corrected[index] = value;
            prev = Some(value);
        }
    }
}

/// 読めなかったフレームを、その先で最初に読めた残量で埋める。
///
/// ラウンド開始演出や勝敗画面ではバーが映らず、そのまま出すとグラフに
/// 偽の急落が現れる。残量はラウンドの頭で満タンに戻るので、後ろから
/// 遡って次の確かな値を引いてくる。末尾は満タンから始める。
///
/// 読めなかった上にほぼ 0 のフレームは、試合画面でもバーが消えている。
/// 同じく埋める対象にする。
pub(crate) fn fill_unreadable_from_the_future(
    corrected: &mut [f32],
    in_match: &[bool],
    in_uncertain: &[bool],
) {
    let mut next_hp = 1.0f32;
    for index in (0..corrected.len()).rev() {
        let is_reliable = in_match[index] && !(in_uncertain[index] && corrected[index] < 0.01);
        if is_reliable {
            next_hp = corrected[index];
        } else {
            corrected[index] = next_hp;
        }
    }
}

/// 範囲内の試合中フレームの最小値。試合中のフレームが無ければ None。
fn window_min(raw: &[f32], in_match: &[bool], range: std::ops::Range<usize>) -> Option<f32> {
    range
        .filter(|index| in_match[*index])
        .map(|index| raw[index])
        .reduce(f32::min)
}

/// ラウンドセグメント内で「偽ハイ」フレームを特定する。
///
/// **ローカル外れ値検出（前後ウィンドウ最小値との比較）**
///
/// HP はラウンド内で単調非増加のはず。フレーム i の raw HP が
/// 前後の SPIKE_WINDOW フレームのウィンドウ最小値より RISE_THRESHOLD 以上高い場合に
/// スパイク（偽ハイ）と判断する。前後両方の条件を AND することで、
/// 偽ロー（遮蔽など）の前後フレームを誤ってスパイクと判定しない。
///
/// - **偽ハイ（髪の毛など）**: 前後とも高い → 両ウィンドウ最小値より高い → スパイク
/// - **偽ロー（スプライト遮蔽など）**: 自身が低い → lookahead_min が自身に引っ張られ
///   第 1 条件が不成立 → スパイク非該当
/// - **偽ロー前後フレーム**: 前後どちらかのウィンドウに偽ローが含まれ AND 条件の
///   一方が不成立 → スパイク非該当
///
/// 以前の `confirmed_max` ベースの実装はスタン外の偽ローで `confirmed_max` が
/// 誤って下がり、回復フレームをスパイクと誤判定する問題があった。
pub(crate) fn compute_spike_frames(
    raw: &[f32],
    in_match: &[bool],
    seg_starts: &[usize],
) -> Vec<bool> {
    const RISE_THRESHOLD: f32 = 0.03; // 前後ウィンドウ最小値より 3% 以上高ければスパイク
    const SPIKE_WINDOW: usize = 90; // 前後 90 フレーム（60fps で 1.5 秒、長い遮蔽に対応）
    let mut in_spike = vec![false; raw.len()];

    for w in seg_starts.windows(2) {
        let (seg_start, seg_end) = (w[0], w[1]);
        let len = seg_end - seg_start;

        // 前後それぞれの窓の最小値。自分自身は入れない（入れると自分と
        // 比べることになり、条件が常に成り立たなくなる）。窓に試合中の
        // フレームが一つも無ければ比較相手が居ない。
        let mut lookahead_min: Vec<Option<f32>> = vec![None; len];
        let mut backward_min: Vec<Option<f32>> = vec![None; len];

        for li in 0..len {
            let i = seg_start + li;
            if !in_match[i] {
                continue;
            }

            // 後方ウィンドウ: i の前 SPIKE_WINDOW フレームの最小値
            let bw_start = if i > seg_start + SPIKE_WINDOW {
                i - SPIKE_WINDOW
            } else {
                seg_start
            };
            backward_min[li] = window_min(raw, in_match, bw_start..i);

            // 前方ウィンドウ: i の後 SPIKE_WINDOW フレームの最小値
            let fw_end = (i + SPIKE_WINDOW + 1).min(seg_end);
            lookahead_min[li] = window_min(raw, in_match, i + 1..fw_end);
        }

        for li in 0..len {
            let i = seg_start + li;
            if !in_match[i] {
                continue;
            }

            let ahead = lookahead_min[li];
            let behind = backward_min[li];

            // 前後どちらの窓の最小値より高い場合だけスパイク。片側だけだと、
            // 偽ローの隣にいるだけの正常な読みまで拾ってしまう。
            let above = |floor: Option<f32>| floor.is_some_and(|f| raw[i] > f + RISE_THRESHOLD);
            if above(ahead) && above(behind) {
                in_spike[i] = true;
            }
        }
    }
    in_spike
}

/// ラウンドセグメント [start, end) 内でスパイク・偽ローフレームを前フレームの値でホールドする。
///
/// 旧 backward_fill（「引き上げ」ベース）を廃止し、よりシンプルな前方パスに置き換えた。
/// 旧実装では raw < max_future - EPSILON → max_future に引き上げていたが、
/// これがダメージフレームも引き上げて実ダメージを消す原因だった。
///
/// 新実装ではホールド対象のみ prev に置き換え、それ以外はそのまま通過させる:
///   - スパイク（体重なり等で偽ハイ）→ prev でホールド
///   - uncertain（白フラッシュ・完全遮蔽）→ prev でホールド
///   - prev から 50% 以上かつ絶対差 0.5 超の急落（爆発エフェクト等の偽ロー）→ ホールド
///   - 通常のダメージ（数 % の降下）→ そのまま通過
pub(crate) fn spike_hold_forward_pass(
    corrected: &mut [f32],
    in_match: &[bool],
    in_spike: &[bool],
    in_uncertain: &[bool],
    start: usize,
    end: usize,
) {
    if start >= end {
        return;
    }
    let mut prev = corrected[start];
    for i in start..end {
        if !in_match[i] {
            continue;
        }
        // 直前から半分以下まで、しかも絶対量でも大きく落ちた読みは、
        // 爆発などでバーが隠れた偽ロー。割合だけで判ずると、残量の少ない
        // ところからのとどめの一撃が消える。
        let collapsed = corrected[i] < prev * 0.5 && prev - corrected[i] > 0.5;
        if in_spike[i] || in_uncertain[i] || collapsed {
            corrected[i] = prev;
        } else {
            prev = corrected[i];
        }
    }
}
