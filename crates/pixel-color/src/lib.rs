//! 画素の色空間変換。
//!
//! HUD の読み取りも入力履歴の読み取りも、色相で色を判定するために同じ
//! 変換を使う。どちらか一方の crate に置くと他方が読み取りモジュールへ
//! 依存することになるため、変換だけを独立させる。

/// これ未満の差は色みが無いものとして扱う。ちょうど同じ値は「色みあり」
/// 側に残す（`<` であって `<=` ではない）。
const FLAT: f32 = 1e-6;

/// RGB f32 → HSV f32 (H: 0–179, S: 0–255, V: 0–255)  OpenCV 互換。
///
/// H を 0–179 に収めるのは OpenCV の 8bit HSV に合わせるため。既存の
/// しきい値がこの尺度で書かれている。
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> [f32; 3] {
    let r = r / 255.0;
    let g = g / 255.0;
    let b = b / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max * 255.0;
    let s = if max > 0.0 { delta / max * 255.0 } else { 0.0 };
    let h_deg = if delta < FLAT {
        0.0_f32
    } else if (max - r).abs() < FLAT {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if (max - g).abs() < FLAT {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    let h = (h_deg / 2.0).round();
    [h, s, v]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: [f32; 3], expected: [f32; 3]) {
        for (a, e) in actual.iter().zip(expected.iter()) {
            assert!((a - e).abs() < 0.51, "{actual:?} != {expected:?}");
        }
    }

    #[test]
    fn primaries_land_on_their_hues() {
        // H は 0–179 尺度。赤 0 / 緑 60 / 青 120。
        close(rgb_to_hsv(255.0, 0.0, 0.0), [0.0, 255.0, 255.0]);
        close(rgb_to_hsv(0.0, 255.0, 0.0), [60.0, 255.0, 255.0]);
        close(rgb_to_hsv(0.0, 0.0, 255.0), [120.0, 255.0, 255.0]);
    }

    /// 赤の手前の色相は 179 側へ回り込む。負の値になると色相の距離計算が
    /// 壊れるので、剰余の向きを固定しておく。
    #[test]
    fn hues_just_below_red_wrap_to_the_top() {
        let [h, _, _] = rgb_to_hsv(255.0, 0.0, 128.0);
        assert!(h > 90.0, "expected a wrapped hue, got {h}");
    }

    #[test]
    fn grey_has_no_hue_and_no_saturation() {
        close(rgb_to_hsv(128.0, 128.0, 128.0), [0.0, 0.0, 128.0]);
        close(rgb_to_hsv(0.0, 0.0, 0.0), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn value_follows_the_brightest_channel_and_saturation_the_spread() {
        close(rgb_to_hsv(200.0, 100.0, 100.0), [0.0, 127.5, 200.0]);
    }

    /// 純色は各成分の差が最大値と一致してしまい、色相の割り算を掛け算や
    /// 剰余に変えても答えが変わらない。三方の分岐それぞれで、差と最大値が
    /// 食い違う中間色を通す。
    #[test]
    fn mid_tones_pin_the_hue_arithmetic_in_every_branch() {
        assert_eq!(rgb_to_hsv(200.0, 150.0, 100.0)[0], 15.0, "赤が最大");
        assert_eq!(rgb_to_hsv(100.0, 200.0, 150.0)[0], 75.0, "緑が最大");
        assert_eq!(rgb_to_hsv(150.0, 100.0, 200.0)[0], 135.0, "青が最大");
    }

    /// 閾値ちょうどの差は「色みなし」に倒さない。三つの比較はいずれも
    /// 等号を含まないので、ちょうどの入力で分岐先が変わってはいけない。
    #[test]
    fn a_difference_exactly_at_the_threshold_still_has_a_hue() {
        // FLAT * 255 を 255 で割ると FLAT に戻る（f32 で厳密に一致する）。
        let at = FLAT * 255.0;
        assert_eq!(at / 255.0, FLAT, "閾値ちょうどの入力を作れていない");

        // delta がちょうど閾値。赤が最大なので赤の分岐へ入る。
        assert_eq!(rgb_to_hsv(at, at / 2.0, 0.0)[0], 15.0);
        // max - r がちょうど閾値。赤ではなく緑の分岐へ入る。
        assert_eq!(rgb_to_hsv(0.0, at, 0.0)[0], 60.0);
        // max - g がちょうど閾値。緑ではなく青の分岐へ入る。
        assert_eq!(rgb_to_hsv(0.0, 0.0, at)[0], 120.0);
    }
}
