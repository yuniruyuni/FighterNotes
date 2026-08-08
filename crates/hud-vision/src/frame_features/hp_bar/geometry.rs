use super::*;

pub(crate) fn hp_roi_base(side: &str) -> (u32, u32, u32, u32) {
    if side == "p1" {
        HP_ROI_P1
    } else {
        HP_ROI_P2
    }
}

/// HP バースキャン平行四辺形の4頂点（1920×1080 スクリーン座標, inclusive ピクセル）。
///
/// classify_hp_col が走査する先頭行・末尾行それぞれの左右端ピクセル位置を表す。
/// デバッグオーバーレイや合成テスト画像の描画境界として使用する。
pub struct HpParallelogram {
    pub top_left: (i32, i32),
    pub top_right: (i32, i32),
    pub bottom_right: (i32, i32),
    pub bottom_left: (i32, i32),
}

/// side（"p1" / "p2"）の HP バースキャン平行四辺形の4頂点を返す。
pub fn hp_parallelogram(side: &str) -> HpParallelogram {
    let (x1, x2, y1, y2) = hp_roi_base(side);
    let roi_w = (x2 - x1) as i32;
    let roi_h = (y2 - y1) as i32;
    let x1 = x1 as i32;
    let y1 = y1 as i32;
    let row_start = HP_COL_ROW_SKIP_TOP as i32;
    let row_end = roi_h - HP_COL_ROW_SKIP_BOTTOM as i32;
    let slope = if side == "p1" {
        HP_BAR_SLOPE
    } else {
        -HP_BAR_SLOPE
    };
    let max_off = ((row_end - 1 - row_start) as f32 * slope).round() as i32;
    HpParallelogram {
        top_left: (x1, y1 + row_start),
        top_right: (x1 + roi_w - 1, y1 + row_start),
        bottom_right: (x1 + roi_w - 1 + max_off, y1 + row_end - 1),
        bottom_left: (x1 + max_off, y1 + row_end - 1),
    }
}
