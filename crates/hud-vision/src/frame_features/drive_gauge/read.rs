use super::{
    classification::{classify_drive_col, segment_drive_runs},
    decode::decode_drive_runs,
    model::DriveGaugeRead,
    scale_roi, SlantedRoi, DRIVE_BAR_SLOPE, DRIVE_ROI_LEFT, DRIVE_ROI_RIGHT,
};

/// 1080p での ROI の幅。他の解像度は、この幅を基準に比例で扱う。
const ROI_REFERENCE_WIDTH: usize = 324;
/// ROI の外縁にあるリム装飾の幅（1080p 換算）。満タン時のグローや
/// バーンアウト枠がここに乗って不定に読めるので、読み取りから外す。
const RIM_WIDTH: usize = 10;

/// 6 セルの実体が収まる範囲の列数。リム装飾を除いた分。
///
/// 潰れた ROI でも 1 列は残す。空の列並びを読むと、決められない状態と
/// 空のゲージが区別できなくなる。
fn cells_span(roi_w: usize) -> usize {
    (roi_w * (ROI_REFERENCE_WIDTH - RIM_WIDTH) / ROI_REFERENCE_WIDTH).max(1)
}

pub(crate) fn drive_gauge_read_impl(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
) -> DriveGaugeRead {
    let is_left = side == "left";
    let (x1_base, x2_base, y1_base, y2_base) = if is_left {
        DRIVE_ROI_LEFT
    } else {
        DRIVE_ROI_RIGHT
    };
    // 潰れた ROI は列が一本も取れず、そのまま「読めなかった」に落ちる。
    let (x1u, x2u, y1u, y2u) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
    let x1 = x1u as usize;
    let x2 = x2u as usize;
    let y1 = y1u as usize;
    let roi_w = x2 - x1;
    let roi_h = y2u as usize - y1;
    let slope: f32 = match side {
        "left" => DRIVE_BAR_SLOPE,
        _ => -DRIVE_BAR_SLOPE,
    };
    let roi = SlantedRoi {
        rgba,
        frame_width: width as usize,
        x: std::ops::Range { start: x1, end: x2 },
        y_start: y1,
        height: roi_h,
        strip_y: y_strip_start,
        slope,
    };

    // 全列をアンカー起点（index 0 = 画面中央側）で分類。
    // 左ゲージはアンカーが右端なので逆順、右ゲージは左端なのでそのまま。
    let mut column = 0usize;
    let mut cols = Vec::new();
    cols.resize_with(roi_w, || {
        let class = classify_drive_col(&roi, column);
        column += 1;
        class
    });
    if is_left {
        cols.reverse();
    }

    cols.truncate(cells_span(roi_w));

    let runs = segment_drive_runs(&cols);
    decode_drive_runs(&runs, cols.len())
}

#[cfg(test)]
mod tests;
