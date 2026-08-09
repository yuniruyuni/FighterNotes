//! HP バーの画素がどの色に当たるかの判定。
//!
//! SF6 は同じ帯の上で四つのことを色で伝える。残っている HP（P1 は赤系、
//! P2 は青系）、減った直後の赤み、危険域の黄、そして空。判定は色相・彩度・
//! 明度の帯で書けるので、走査から切り離してここに集める。
//!
//! 閾値そのものが仕様なので、由来の分かる形で名前を付けておく。

use pixel_color::rgb_to_hsv;

/// ダメージ直後の橙。
///
/// 明度に上限を置かない。低 HP の黄色バーは色相で外れるため、上限を
/// 設けると高輝度の橙を取りこぼす方が害が大きい。
pub(crate) fn is_damage_orange(r: f32, g: f32, b: f32) -> bool {
    let [hue, saturation, value] = rgb_to_hsv(r, g, b);
    (10.0..=27.0).contains(&hue) && saturation > 60.0 && value > 80.0
}

/// 危険域（残 25% 以下）の黄。橙より高い彩度と明度を要求して、
/// 「いま減った」と「残り少ない」を分ける。
pub(crate) fn is_low_health_yellow(r: f32, g: f32, b: f32) -> bool {
    let [hue, saturation, value] = rgb_to_hsv(r, g, b);
    (22.0..=35.0).contains(&hue) && saturation > 120.0 && value > 200.0
}

/// 残っている HP の色。側で色相帯が変わる。
///
/// P1 の彩度下限が高いのは、HP ROI に重なるキャラクタースプライトの
/// 暗赤（彩度 60 前後）を落とすため。バー本体は 220 前後で通る。
pub(crate) fn is_remaining_health(side: &str, r: f32, g: f32, b: f32) -> bool {
    let [hue, saturation, value] = rgb_to_hsv(r, g, b);
    if side == "p1" {
        (hue <= 20.0 || hue >= 145.0) && saturation > 100.0 && value > 60.0
    } else {
        (88.0..=160.0).contains(&hue) && saturation > 45.0 && value > 60.0
    }
}

/// 列がその色だと見なすのに必要な割合。
pub(crate) mod ratio {
    /// 橙。演出の途中でまだらに乗るので低め。
    pub(crate) const DAMAGE_ORANGE: f32 = 0.15;
    /// 黄。
    pub(crate) const LOW_HEALTH_YELLOW: f32 = 0.15;
    /// P1 の残量。フレームメーターのディザリングで抜けるため低め。
    pub(crate) const REMAINING_P1: f32 = 0.10;
    /// P2 の残量。遮蔽ノイズを許容する。
    pub(crate) const REMAINING_P2: f32 = 0.15;
    /// 残量としての黄。髪などのテクスチャは列の一部しか黄にならないので、
    /// 列のほとんどが黄であることを求める。
    pub(crate) const REMAINING_YELLOW: f32 = 0.60;
}

#[cfg(test)]
mod tests;
