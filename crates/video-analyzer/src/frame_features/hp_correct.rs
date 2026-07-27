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
    let n = features.len();

    // 試合画面フレームのみを対象とするマスク（非試合フレームは補正対象外）
    let in_match: Vec<bool> = features.iter().map(|f| f.is_match_screen).collect();

    // raw HP を抽出し、時系列中央値フィルターでフレーム間ノイズを除去する。
    //
    // 黄色 HP 域（≤25%）では raw 検出が ±5% 程度揺れ、Phase 3 の単調制約と
    // backward_fill の EPSILON 閾値の組み合わせにより「ラチェット効果」が生じる:
    //   小さな下振れフレームが補正されず、それ以降のフレームが低い値に固定される。
    //
    // 5 フレーム中央値フィルター（前後 MEDIAN_HALF フレーム）で 1〜2 フレームの
    // ノイズスパイクを除去する。本物のダメージは複数フレーム持続するため保持される。
    // raw_orig は left_hp_raw / right_hp_raw としてデバッグ表示用に保持される。
    const MEDIAN_HALF: usize = 2; // 前後 2 フレーム = 計 5 フレームウィンドウ
    let raw: Vec<f32> = {
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
        let mut smoothed = raw_orig.clone();
        let mut buf = Vec::with_capacity(MEDIAN_HALF * 2 + 1);
        for i in 0..n {
            if !in_match[i] {
                continue;
            }
            let lo = i.saturating_sub(MEDIAN_HALF);
            let hi = (i + MEDIAN_HALF + 1).min(n);
            buf.clear();
            for j in lo..hi {
                if in_match[j] {
                    buf.push(raw_orig[j]);
                }
            }
            // buf は常に 1 要素以上（自フレームが in_match のため）
            buf.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            smoothed[i] = buf[buf.len() / 2];
        }
        smoothed
    };

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

    // ラウンド境界: 非試合画面 → 試合画面 の遷移 = 新ラウンド開始
    //
    // HP ジャンプで境界を検出する旧実装には 2 つの問題があった:
    //   (1) body overlap（偽ロー回復）でも +0.5 超の HP ジャンプが発生し偽境界が生じる
    //   (2) ラウンド開始 HP アニメの途中フレーム（~0.85）が閾値を超えず境界を取りこぼす
    //
    // ラウンド間には必ず「YOU WIN」/VS 画面（is_match_screen=false の期間）が挟まるため、
    // false → true の遷移を境界とすれば (1)(2) 両方を解決できる。
    // body overlap 中は is_match_screen が true のまま変わらないため偽境界は生じない。
    let mut seg_starts = vec![0usize];
    for i in 1..n {
        if in_match[i] && !in_match[i - 1] {
            seg_starts.push(i);
        }
    }
    seg_starts.push(n);

    // ─── Phase 1 前処理: ラウンド開始フレームを 1.0 に強制リセット ──────────
    // ラウンド開始時は HP が必ず満タン（1.0）になる。しかし「ROUND!/FIGHT!」
    // オーバーレイ中の最初の in_match フレームでは HP バーが読み取れず、raw が
    // 0.99 前後の低い値になることがある（例: 0.9914）。
    // この値が backward_fill の内部前方単調パスで prev として固定されると、
    // セグメント全体の HP が 0.9914 以下に制約され、実ダメージ（6〜7%）が
    // Phase 3 で反映されなくなる（own_hp が 0.9914 から下がらない）。
    //
    // backward_fill より前に corrected を 1.0 にリセットすることで、
    // backward_fill の内部前方単調パスが prev=1.0 から正しく開始できる。
    //
    // 「ラウンド開始」= 非試合フレームの直後（is_match[seg_start-1]=false）。
    // 動画先頭から試合が始まる場合（seg_start=0）はラウンド途中から録画の可能性があり
    // HP=1.0 が保証されないためリセットしない。
    for w in seg_starts.windows(2) {
        let seg_start = w[0];
        let is_round_start = seg_start > 0 && !in_match[seg_start - 1];
        if is_round_start && seg_start < n && in_match[seg_start] {
            corrected[seg_start] = 1.0;
        }
    }

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

    // ─── Phase 3: 前方単調パス（ラウンドセグメントごと・in-match フレームのみ）───────
    // セグメント単位で実行することでラウンドリセット例外が不要になる。
    // ラウンド間の HP ジャンプはセグメント境界で prev がリセットされるため自然に許容される。
    //
    // 旧実装（全体 1 パス＋ p+0.4 例外）では backward_fill が誤って引き上げたフレームが
    // 0.4 超の前後差を生んだとき「ラウンドリセット」と誤判定して HP が増加できていた。
    // セグメント内では HP は絶対に増加しないためラウンドリセット例外は不要。
    //
    // uncertain かつ corrected ≈ 0 のフレームは HP バーが演出や遮蔽で消えた偽ロー。
    // このフレームで prev を更新すると 0 が後続フレームへ伝播するため除外する。
    for w in seg_starts.windows(2) {
        let (seg_start, seg_end) = (w[0], w[1]);
        let mut prev: Option<f32> = None;
        for i in seg_start..seg_end {
            if !in_match[i] {
                continue;
            }
            if in_uncertain[i] && corrected[i] < 0.01 {
                continue;
            }
            match prev {
                None => {
                    prev = Some(corrected[i]);
                }
                Some(p) => {
                    corrected[i] = corrected[i].min(p);
                    prev = Some(corrected[i]);
                }
            }
        }
    }

    // ─── Phase 4: 非試合フレーム（+ uncertain かつ HP≈0 の偽非試合フレーム）を次セグメント開始 HP で埋める ─────────────────
    // ラウンド開始アニメーション（ROUND/FIGHT! オーバーレイ）中や YOU WIN 画面などの
    // 非試合フレームは HP バーが視認できず raw 値が不安定。
    // これらを補正せずに own_hp へ書き込むと、グラフ表示上に偽の急落が現れる。
    //
    // 後方パスで「直後に来る試合フレームの HP」を非試合フレームに伝播することで
    // ラウンド開始アニメーション中の偽降下を除去する。
    // HP はラウンド開始時に 100% にリセットされるため、次セグメント開始 HP を使う。
    //
    // uncertain かつ corrected ≈ 0 のフレームは HP バー消失演出（演出・体重なりによる
    // 完全遮蔽）による偽ロー。is_match=True でも非試合扱いとして次 HP に補完する。
    {
        let mut next_hp = 1.0f32;
        for i in (0..n).rev() {
            let is_reliable = in_match[i] && !(in_uncertain[i] && corrected[i] < 0.01);
            if is_reliable {
                next_hp = corrected[i];
            } else {
                corrected[i] = next_hp;
            }
        }
    }

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

        // lookahead_min[li] = min(raw[i+1 .. i+W+1] の in-match フレーム)
        // backward_min[li]  = min(raw[i-W .. i]     の in-match フレーム)
        // 両方とも raw[i] 自身を含まない（自分自身と比較すると常に条件不成立になるため）
        let mut lookahead_min = vec![f32::MAX; len];
        let mut backward_min = vec![f32::MAX; len];

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
            let mut bmin = f32::MAX;
            for j in bw_start..i {
                if in_match[j] {
                    bmin = bmin.min(raw[j]);
                }
            }
            backward_min[li] = bmin;

            // 前方ウィンドウ: i の後 SPIKE_WINDOW フレームの最小値
            let fw_end = (i + SPIKE_WINDOW + 1).min(seg_end);
            let mut fmin = f32::MAX;
            for j in i + 1..fw_end {
                if in_match[j] {
                    fmin = fmin.min(raw[j]);
                }
            }
            lookahead_min[li] = fmin;
        }

        for li in 0..len {
            let i = seg_start + li;
            if !in_match[i] {
                continue;
            }

            let ahead = lookahead_min[li];
            let behind = backward_min[li];

            // 前後ウィンドウの最小値より THRESHOLD 以上高い場合のみスパイク
            // AND 条件: 一方だけ満たす場合は偽ロー前後フレームなので除外
            if ahead != f32::MAX
                && behind != f32::MAX
                && raw[i] > ahead + RISE_THRESHOLD
                && raw[i] > behind + RISE_THRESHOLD
            {
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
        let is_spike_or_uncertain = in_spike[i] || in_uncertain[i];
        let is_false_low =
            !is_spike_or_uncertain && corrected[i] < prev * 0.5 && prev - corrected[i] > 0.5;
        if is_spike_or_uncertain || is_false_low {
            corrected[i] = prev;
        } else {
            prev = corrected[i];
        }
    }
}
