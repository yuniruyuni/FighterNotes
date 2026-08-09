//! HP の読み違いを時間方向で均す処理に対するテスト。
//!
//! 1 フレームの読みは、キャラクターがバーに重なったり爆発で白飛びしたりで
//! 上にも下にも外れる。上へ外れた読み（スパイク）は前後より高いことで、
//! 下へ外れた読み（偽ロー）は落ち方の急さで見分ける。
//!
//! ここを取り違えると、実ダメージを消すか、無いダメージを作るかのどちらかに
//! なる。どちらも集計を直接壊す。

use crate::frame_features::{compute_spike_frames, spike_hold_forward_pass};

/// 1 ラウンド分の走査範囲。
fn one_round(len: usize) -> (Vec<bool>, Vec<usize>) {
    (vec![true; len], vec![0, len])
}

/// 前後より明らかに高い読みはスパイク。体が重なってバーが長く見えた
/// フレームがこれに当たる。
#[test]
fn a_reading_higher_than_both_neighbourhoods_is_a_spike() {
    let mut raw = vec![0.50_f32; 200];
    raw[100] = 0.80;
    let (in_match, segments) = one_round(raw.len());

    let spikes = compute_spike_frames(&raw, &in_match, &segments);

    assert!(spikes[100], "前後より高い読みを拾えていない");
    assert!(!spikes[99] && !spikes[101], "隣まで巻き込んでいる");
}

/// 前後どちらか片側だけが低い場合はスパイクにしない。偽ローの隣にいる
/// だけの正常な読みを消してしまう。
#[test]
fn a_reading_next_to_a_false_low_is_not_a_spike() {
    let mut raw = vec![0.50_f32; 200];
    // 1 フレームだけ大きく落ちた偽ロー。その隣は正常な 0.50。
    raw[100] = 0.05;
    let (in_match, segments) = one_round(raw.len());

    let spikes = compute_spike_frames(&raw, &in_match, &segments);

    assert!(!spikes[99], "偽ローの手前を持ち上げている");
    assert!(!spikes[101], "偽ローの直後を持ち上げている");
    assert!(!spikes[100], "偽ロー自身をスパイクと読んでいる");
}

/// 通常のダメージはスパイクではない。減ったあとは戻らないので、
/// 後ろのウィンドウだけが低くなる。
#[test]
fn ordinary_damage_is_not_a_spike() {
    let raw: Vec<f32> = (0..200)
        .map(|i| if i < 100 { 0.80 } else { 0.60 })
        .collect();
    let (in_match, segments) = one_round(raw.len());

    let spikes = compute_spike_frames(&raw, &in_match, &segments);

    assert!(
        spikes.iter().all(|value| !value),
        "被弾前のフレームをスパイクと読んでいる"
    );
}

/// 試合画面でないフレームは比較にも結果にも含めない。ラウンド間の
/// 暗転を混ぜると、直後の読みがすべてスパイクになる。
#[test]
fn frames_outside_the_match_take_no_part() {
    let mut raw = vec![0.50_f32; 200];
    let mut in_match = vec![true; 200];
    for (index, value) in raw.iter_mut().enumerate().take(120).skip(80) {
        *value = 0.0;
        in_match[index] = false;
    }
    let segments = vec![0, 200];

    let spikes = compute_spike_frames(&raw, &in_match, &segments);

    assert!(
        spikes[80..120].iter().all(|value| !value),
        "試合外を結果に含めている"
    );
    assert!(
        spikes.iter().all(|value| !value),
        "試合外の 0 を比較に含めて、周りをスパイクにしている"
    );
}

/// 区間の端は片側の比較相手を持たないので、スパイクにしない。ラウンドの
/// 頭は必ず満タンで、直前と比べようがない。
#[test]
fn the_first_frame_of_a_round_is_never_a_spike() {
    let mut raw = vec![0.50_f32; 200];
    raw[0] = 1.00;
    let (in_match, segments) = one_round(raw.len());

    let spikes = compute_spike_frames(&raw, &in_match, &segments);

    assert!(!spikes[0], "比較相手が無いのにスパイクと判定している");
}

/// 区間を分ければ、境界の向こう側は比較に入らない。またいで比べると、
/// 次のラウンドの低い値が前ラウンド終盤を「周りより高い」に見せてしまう。
#[test]
fn the_rounds_are_compared_separately() {
    // 第 1 ラウンドは 0.50 で推移し、途中に一度だけ 0.40 まで下がる。
    // 第 2 ラウンドは 0.10 から始まる（前ラウンドで削られた側）。
    let mut raw: Vec<f32> = (0..400)
        .map(|i| if i < 200 { 0.50 } else { 0.10 })
        .collect();
    raw[150] = 0.40;
    let in_match = vec![true; 400];

    let across = compute_spike_frames(&raw, &in_match, &[0, 400]);
    let separated = compute_spike_frames(&raw, &in_match, &[0, 200, 400]);

    assert!(
        across[195],
        "またいで比べると、次のラウンドの低さに引っ張られてスパイクになる"
    );
    assert!(
        !separated[195],
        "ラウンド内で比べれば、ただの平常値をスパイクにしない"
    );
}

/// スパイクと判定したフレームは、直前の確かな値で埋める。埋めないと
/// 上振れがそのまま残り、次の下降が過大なダメージに見える。
#[test]
fn a_spike_is_replaced_by_the_last_trusted_reading() {
    let mut corrected = vec![0.50_f32, 0.50, 0.90, 0.50];
    let in_match = vec![true; 4];
    let in_spike = vec![false, false, true, false];
    let in_uncertain = vec![false; 4];

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &in_uncertain, 0, 4);

    assert_eq!(corrected, vec![0.50, 0.50, 0.50, 0.50]);
}

/// 読み取れなかったフレームも同じく直前の値で埋める。白飛びや完全な
/// 遮蔽がこれに当たる。
#[test]
fn an_unreadable_frame_holds_the_previous_value() {
    let mut corrected = vec![0.70_f32, 0.70, 0.00, 0.65];
    let in_match = vec![true; 4];
    let in_spike = vec![false; 4];
    let in_uncertain = vec![false, false, true, false];

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &in_uncertain, 0, 4);

    assert_eq!(corrected, vec![0.70, 0.70, 0.70, 0.65]);
}

/// 半分以下まで一気に落ちた読みは偽ロー。爆発エフェクトでバーが
/// 隠れたフレームがこれに当たる。
#[test]
fn a_sudden_collapse_is_treated_as_a_misread() {
    let mut corrected = vec![0.90_f32, 0.90, 0.10, 0.88];
    let in_match = vec![true; 4];
    let in_spike = vec![false; 4];
    let in_uncertain = vec![false; 4];

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &in_uncertain, 0, 4);

    assert_eq!(corrected[2], 0.90, "偽ローを埋めていない");
    assert_eq!(corrected[3], 0.88, "その後の正常な読みまで潰している");
}

/// 大きくても半分を下回らない下降は、実際のダメージとして通す。
/// ここを埋めてしまうと、大ダメージが記録から消える。
#[test]
fn a_large_but_believable_drop_is_real_damage() {
    let mut corrected = vec![0.90_f32, 0.90, 0.50, 0.50];
    let in_match = vec![true; 4];
    let in_spike = vec![false; 4];
    let in_uncertain = vec![false; 4];

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &in_uncertain, 0, 4);

    assert_eq!(corrected[2], 0.50, "実ダメージを消している");
}

/// 残量が少ないところからの下降は、割合では急でも絶対量が小さい。
/// 割合だけで判断すると、とどめの一撃が消える。
#[test]
fn a_finishing_blow_from_low_health_is_not_a_misread() {
    let mut corrected = vec![0.30_f32, 0.30, 0.00, 0.00];
    let in_match = vec![true; 4];
    let in_spike = vec![false; 4];
    let in_uncertain = vec![false; 4];

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &in_uncertain, 0, 4);

    assert_eq!(corrected[2], 0.00, "KO のフレームを埋めている");
}

/// 試合外のフレームは書き換えず、直前の値の引き継ぎにも影響させない。
#[test]
fn frames_outside_the_match_are_left_untouched() {
    let mut corrected = vec![0.80_f32, 0.80, 0.00, 0.78];
    let in_match = vec![true, true, false, true];
    let in_spike = vec![false; 4];
    let in_uncertain = vec![false; 4];

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &in_uncertain, 0, 4);

    assert_eq!(corrected[2], 0.00, "試合外を書き換えている");
    assert_eq!(corrected[3], 0.78, "試合外を挟んで引き継ぎが壊れている");
}

/// 空の範囲では何もしない。
#[test]
fn an_empty_range_changes_nothing() {
    let mut corrected = vec![0.5_f32; 4];
    let before = corrected.clone();

    spike_hold_forward_pass(&mut corrected, &[true; 4], &[true; 4], &[true; 4], 2, 2);

    assert_eq!(corrected, before);
}
