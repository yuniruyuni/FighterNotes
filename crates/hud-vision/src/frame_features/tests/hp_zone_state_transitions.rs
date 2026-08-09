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
