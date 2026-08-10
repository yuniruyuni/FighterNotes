//! HP バーのゾーン列を読む状態機械に対するテスト。
//!
//! バーは右端の白い枠から始まり、残っている部分、いま減った橙の帯、
//! 空き、そして左端の白い枠、と決まった順に並ぶ。順序から外れた並びは
//! 遮蔽か演出なので、値を出さずに諦める。
//!
//! ここで諦め損ねると、遮蔽された画から作った数字が確定値として通る。

use super::support::*;

/// ROI の全幅。ゾーン列は端から端まで隙間なく並べる。
const COLUMNS: usize = 681;

/// 幅の並びからゾーン列を作り、余りを空きで埋める。
fn zones(spec: &[(HpColColor, usize)]) -> Vec<HpZone> {
    let used: usize = spec.iter().map(|&(_, w)| w).sum();
    let mut spec = spec.to_vec();
    if used < COLUMNS {
        spec.push((HpColColor::Dark, COLUMNS - used));
    }
    zones_from(&spec)
}

/// 残っている部分のあとに橙の帯があれば、そこがダメージ。境界の白枠まで
/// 読んで終える。
#[test]
fn damage_runs_from_the_fill_edge_to_the_left_cap() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 200),
            (White, 3),
            (Orange, 100),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(!decode.uncertain);
    assert!(decode.orange_fill > 0.0, "ダメージ帯を読めていない");
    assert!(
        decode.damage_left_a.is_some(),
        "ダメージ帯の左端を記録していない"
    );
}

/// 橙と残量の境目に出る滲みの色も、ダメージ帯の一部として続ける。
/// ここで切ると、ダメージが実際より狭く出る。
#[test]
fn the_blended_edge_colour_continues_the_damage_zone() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 200),
            (White, 3),
            (YellowWhite, 4),
            (Orange, 96),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(!decode.uncertain);
    assert!(decode.orange_fill > 0.0, "滲みで帯が切れている");
}

/// 消えかけの残像もダメージ帯として続ける。演出の途中で色が薄くなる。
#[test]
fn a_fading_ghost_is_still_part_of_the_damage_zone() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 200),
            (White, 3),
            (Orange, 60),
            (Ghost, 40),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(!decode.uncertain);
    assert!(decode.orange_fill > 0.0);
}

/// ダメージ帯が空きに変わったら、そこで終わる。残量は読めているので
/// 諦めはしないが、帯の左端は枠か滲みでしか確定しないので幅は出さない。
#[test]
fn the_damage_zone_ends_where_the_empty_part_begins() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 200),
            (White, 3),
            (Orange, 60),
            (Dark, 50),
        ]),
        COLUMNS,
    );

    assert!(!decode.uncertain, "残量が読めているのに諦めている");
    assert!(decode.fill_ratio > 0.0);
    assert_eq!(
        decode.orange_fill, 0.0,
        "左端を見ていないのに帯の幅を出している"
    );
}

/// 太い白帯は枠ではなく遮蔽。読み取りを諦める。白い演出がバーに
/// 重なった場面で、当て推量の値を出さないため。
#[test]
fn a_wide_white_band_is_an_occlusion_not_a_cap() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 200),
            (White, 3),
            (Orange, 60),
            (White, 10),
        ]),
        COLUMNS,
    );

    assert!(decode.uncertain, "太い白帯を枠と読んでいる");
}

/// ダメージが無い場面では、残量のあとは空きが続いて左端の枠で終わる。
#[test]
fn a_stable_bar_runs_from_the_fill_edge_to_the_empty_end() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[(White, 3), (Fill, 200), (White, 3), (Dark, 400), (White, 3)]),
        COLUMNS,
    );

    assert!(!decode.uncertain);
    assert_eq!(decode.orange_fill, 0.0, "無いダメージを作っている");
    assert!(decode.fill_ratio > 0.0, "残量を読めていない");
}

/// 残量の途中に入る細い暗帯は、フレームメーターの描画。残量は途切れて
/// いないものとして続ける。
#[test]
fn a_thin_dark_stripe_inside_the_fill_does_not_end_it() {
    use HpColColor::*;
    let with_stripe = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (Dark, 8),
            (Fill, 92),
            (White, 3),
            (Dark, 400),
            (White, 3),
        ]),
        COLUMNS,
    );
    let without_stripe = decode_hp_zones(
        &zones(&[(White, 3), (Fill, 200), (White, 3), (Dark, 400), (White, 3)]),
        COLUMNS,
    );

    assert!(!with_stripe.uncertain, "描画の暗帯で読み取りを諦めている");
    assert_eq!(
        with_stripe.fill_ratio, without_stripe.fill_ratio,
        "暗帯を挟むと残量がずれる"
    );
}

/// 残量の途中に広い暗帯が現れたら、それが空きの始まりなのか重なった
/// スプライトなのかはゾーンの並びから区別できない。残量の見込みは
/// 出しつつ、確定値としては扱わない。
#[test]
fn a_wide_dark_band_inside_the_fill_is_not_a_confident_reading() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[(White, 3), (Fill, 200), (Dark, 400), (White, 3)]),
        COLUMNS,
    );

    assert!(decode.uncertain, "遮蔽かもしれない読みを確定にしている");
    assert!(decode.fill_ratio > 0.0, "残量の見込みすら出していない");
}

/// 枠の手前が全部空きなら残量ゼロ。KO の瞬間がこれに当たる。ここは
/// 遮蔽と違って迷う余地がないので、確定値として通す。
#[test]
fn an_empty_bar_right_after_the_cap_reads_zero() {
    use HpColColor::*;
    let decode = decode_hp_zones(&zones(&[(White, 3), (Dark, 675), (White, 3)]), COLUMNS);

    assert!(!decode.uncertain, "空のバーを読めていない");
    assert_eq!(decode.fill_ratio, 0.0);
}

/// ゾーンが一つも無ければ何も読めない。
#[test]
fn an_empty_zone_list_reads_nothing() {
    let decode = decode_hp_zones(&[], COLUMNS);

    assert_eq!(decode.fill_ratio, 0.0);
    assert_eq!(decode.orange_fill, 0.0);
}

// ── 走査を打ち切る位置 ───────────────────────────────────────────────────
//
// 読み取りは、答えが出た時点でも諦めた時点でも、そこで打ち切る。打ち切り
// 損ねると、その先にある別のゲージや演出の断片が読みに混ざる。
// 以下はどれも「打ち切った先に、続けたら結果を変えるゾーンがある」形。

/// 太い白帯で諦めたあと、その先に正しい枠が並んでいても読み直さない。
#[test]
fn a_reading_abandoned_at_a_wide_white_band_does_not_resume() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 7),
            (White, 3),
            (Fill, 200),
            (White, 3),
            (Dark, 400),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(decode.uncertain, "諦めたはずの走査が先で読み直している");
    assert_eq!(decode.fill_ratio, 0.0);
}

/// 枠より先に残量が出るのは枠が塞がれているということ。その先に
/// 正しい枠があっても読み直さない。
#[test]
fn a_blocked_cap_is_not_recovered_by_a_later_cap() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (Fill, 50),
            (White, 3),
            (Fill, 200),
            (White, 3),
            (Dark, 400),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(decode.uncertain, "塞がれた枠を無視して読んでいる");
}

/// 残量ゼロと読んだら、その先に残量の色があっても数えない。別のゲージが
/// 重なっている場面。
#[test]
fn an_empty_bar_does_not_pick_up_fill_from_further_along() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Dark, 200),
            (Fill, 100),
            (White, 3),
            (Dark, 372),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert_eq!(decode.fill_ratio, 0.0, "先の残量を拾っている");
}

/// 残量の途中の太い白帯で諦めたあと、その先の枠を充填端にしない。
#[test]
fn a_white_band_inside_the_fill_does_not_defer_to_a_later_edge() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (White, 10),
            (Fill, 50),
            (White, 3),
            (Dark, 512),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(decode.uncertain);
    assert_eq!(decode.fill_ratio, 0.0, "遮蔽の先を充填端にしている");
}

/// 残量の中に橙が出るのは順序が壊れている。その先の枠を充填端にしない。
#[test]
fn orange_inside_the_fill_does_not_defer_to_a_later_edge() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (Orange, 50),
            (White, 3),
            (Dark, 522),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(decode.uncertain);
    assert_eq!(decode.fill_ratio, 0.0, "壊れた並びから充填端を出している");
}

/// 充填端の先の太い白帯で諦めたあと、その先の橙をダメージにしない。
#[test]
fn a_white_band_past_the_fill_edge_does_not_yield_a_damage_zone() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (White, 3),
            (Dark, 20),
            (White, 10),
            (Orange, 30),
            (White, 3),
            (Dark, 509),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(decode.uncertain);
    assert!(
        decode.damage_left_a.is_none(),
        "遮蔽の先の橙をダメージと読んでいる"
    );
}

/// 左端の枠まで読んで終えたら、その先の橙はダメージではない。
#[test]
fn the_reading_ends_at_the_left_cap() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (White, 3),
            (Dark, 20),
            (White, 3),
            (Orange, 30),
            (White, 3),
            (Dark, 516),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(!decode.uncertain);
    assert!(
        decode.damage_left_a.is_none(),
        "枠の先の橙をダメージと読んでいる"
    );
}

/// ダメージ帯が空きに変わって終えたら、その先の橙は別物。
#[test]
fn the_damage_zone_does_not_resume_after_the_empty_part() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (White, 3),
            (Orange, 30),
            (Dark, 20),
            (Orange, 30),
            (White, 3),
            (Dark, 489),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(
        decode.damage_left_a.is_none(),
        "空きを跨いだ先の橙を繋いでいる"
    );
}

/// ダメージ帯の中の太い白帯で諦めたあと、その先の枠を帯の左端にしない。
#[test]
fn a_white_band_inside_the_damage_zone_does_not_yield_a_boundary() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (White, 3),
            (Orange, 30),
            (White, 10),
            (Orange, 20),
            (White, 3),
            (Dark, 509),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert!(decode.uncertain);
    assert!(
        decode.damage_left_a.is_none(),
        "遮蔽の先を帯の左端にしている"
    );
}

/// ダメージ帯の左端は最初に見つけた枠。その先にもう一つ枠があっても
/// 乗り換えない。
#[test]
fn the_damage_boundary_is_the_first_cap_found() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (White, 3),
            (Orange, 30),
            (White, 3),
            (Orange, 20),
            (White, 3),
            (Dark, 516),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert_eq!(
        decode.damage_left_a,
        Some(138),
        "先の枠に乗り換えて帯を広げている"
    );
}

/// 滲みで帯が終わった場合も、そこが左端。
#[test]
fn a_blended_edge_also_ends_the_damage_zone() {
    use HpColColor::*;
    let decode = decode_hp_zones(
        &zones(&[
            (White, 3),
            (Fill, 100),
            (White, 3),
            (Orange, 30),
            (YellowWhite, 3),
            (Orange, 20),
            (White, 3),
            (Dark, 516),
            (White, 3),
        ]),
        COLUMNS,
    );

    assert_eq!(
        decode.damage_left_a,
        Some(138),
        "滲みの先まで帯を伸ばしている"
    );
}
