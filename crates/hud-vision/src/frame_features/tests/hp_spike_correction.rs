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

    // 暗転の先にあるスパイクは、そこまで走査が届いて初めて見つかる。
    let mut later = raw.clone();
    later[150] = 0.80;
    let found = compute_spike_frames(&later, &in_match, &segments);

    assert!(found[150], "試合外のフレームで走査が止まっている");
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

// ── 閾値の境目 ───────────────────────────────────────────────────────────

/// スパイクと認めるのは、前後より決まった幅を「超えて」高いとき。
/// ちょうどの差は演出の揺らぎとして通す。
#[test]
fn the_rise_that_makes_a_spike_has_an_exact_edge() {
    let at_the_edge = {
        let mut raw = vec![0.50_f32; 200];
        raw[100] = 0.53;
        let (in_match, segments) = one_round(raw.len());
        compute_spike_frames(&raw, &in_match, &segments)
    };
    let just_over = {
        let mut raw = vec![0.50_f32; 200];
        raw[100] = 0.531;
        let (in_match, segments) = one_round(raw.len());
        compute_spike_frames(&raw, &in_match, &segments)
    };

    assert!(!at_the_edge[100], "ちょうどの差をスパイクにしている");
    assert!(just_over[100], "超えた差をスパイクにしていない");
}

/// 前を見る窓は決まった長さ。窓の外まで見ると、ラウンド終盤の低い値と
/// 序盤の平常値を比べてスパイクにしてしまう。
#[test]
fn the_forward_window_has_a_fixed_reach() {
    // 直前に一度落ちているので、後ろ側の条件は満たしている。あとは
    // 先の低い値が窓に入るかどうかだけで結果が変わる。
    let spikes_at = |distance: usize| {
        let mut raw = vec![0.60_f32; 400];
        raw[149] = 0.40;
        raw[150] = 0.62;
        raw[150 + distance] = 0.40;
        let (in_match, segments) = one_round(raw.len());
        compute_spike_frames(&raw, &in_match, &segments)[150]
    };

    assert!(spikes_at(90), "窓の中の低い値と比べていない");
    assert!(!spikes_at(91), "窓の外の低い値と比べている");
}

/// 半分ちょうどまでの下降は実ダメージ。埋めると記録から消える。
#[test]
fn a_drop_to_exactly_half_is_real_damage() {
    let mut kept = vec![1.20_f32, 1.20, 0.60, 0.60];
    let mut held = vec![1.20_f32, 1.20, 0.59, 0.59];
    let quiet = [false; 4];

    spike_hold_forward_pass(&mut kept, &[true; 4], &quiet, &quiet, 0, 4);
    spike_hold_forward_pass(&mut held, &[true; 4], &quiet, &quiet, 0, 4);

    assert_eq!(kept[2], 0.60, "半分ちょうどの下降を消している");
    assert_eq!(held[2], 1.20, "半分を割った下降を通している");
}

/// 割合で急でも、絶対量が小さければ実ダメージ。残量の少ないところからの
/// 一撃を消さないため。
#[test]
fn a_proportionally_steep_but_small_drop_is_real_damage() {
    let mut kept = vec![0.90_f32, 0.90, 0.40, 0.40];
    let mut held = vec![0.90_f32, 0.90, 0.39, 0.39];
    let quiet = [false; 4];

    spike_hold_forward_pass(&mut kept, &[true; 4], &quiet, &quiet, 0, 4);
    spike_hold_forward_pass(&mut held, &[true; 4], &quiet, &quiet, 0, 4);

    assert_eq!(kept[2], 0.40, "絶対量の小さい下降を消している");
    assert_eq!(held[2], 0.90, "絶対量の大きい下降を通している");
}

/// 試合外のフレームを挟んでも、その先のフレームは処理を続ける。
/// 打ち切ると、ラウンド途中の暗転から先が補正されないまま残る。
#[test]
fn a_frame_outside_the_match_does_not_end_the_pass() {
    let mut corrected = vec![0.80_f32, 0.30, 0.75, 0.90];
    let in_match = vec![true, false, true, true];
    let in_spike = vec![false, false, false, true];

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &[false; 4], 0, 4);

    assert_eq!(corrected[1], 0.30, "試合外を書き換えている");
    assert_eq!(corrected[3], 0.75, "試合外の先でホールドが止まっている");
}

/// 絶対量の条件はちょうどでは足りない。0.5 だけ落ちた読みは、
/// 大きくても実ダメージとして通す。
#[test]
fn a_collapse_of_exactly_the_limit_is_still_real_damage() {
    let mut kept = vec![0.75_f32, 0.75, 0.25, 0.25];
    let mut held = vec![0.75_f32, 0.75, 0.24, 0.24];
    let quiet = [false; 4];

    spike_hold_forward_pass(&mut kept, &[true; 4], &quiet, &quiet, 0, 4);
    spike_hold_forward_pass(&mut held, &[true; 4], &quiet, &quiet, 0, 4);

    assert_eq!(kept[2], 0.25, "ちょうどの落差を消している");
    assert_eq!(held[2], 0.75, "落差を超えた読みを通している");
}

/// 範囲の頭が末尾を越えていても、範囲外を読まない。
#[test]
fn a_range_at_the_very_end_reads_nothing() {
    let mut corrected = vec![0.5_f32; 4];
    let before = corrected.clone();

    spike_hold_forward_pass(&mut corrected, &[true; 4], &[true; 4], &[true; 4], 4, 4);

    assert_eq!(corrected, before);
}

/// 範囲の外は触らない。ラウンドごとに呼ぶので、前のラウンドの値を
/// 書き換えてはいけない。
#[test]
fn frames_before_the_range_are_left_alone() {
    let mut corrected = vec![0.90_f32, 0.20, 0.80, 0.90];
    let in_match = vec![true; 4];
    let in_spike = vec![true, true, false, true];

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &[false; 4], 2, 4);

    assert_eq!(corrected[0], 0.90, "範囲の手前を書き換えている");
    assert_eq!(corrected[1], 0.20, "範囲の手前を書き換えている");
    assert_eq!(corrected[3], 0.80, "範囲の中でホールドしていない");
}
