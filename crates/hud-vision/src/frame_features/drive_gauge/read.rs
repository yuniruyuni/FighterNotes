use super::{
    classification::{classify_drive_col, segment_drive_runs},
    decode::decode_drive_runs,
    model::{DriveColClass, DriveGaugeRead},
    scale_roi, SlantedRoi, DRIVE_BAR_SLOPE, DRIVE_ROI_LEFT, DRIVE_ROI_RIGHT,
};

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
    let (x1u, x2u, y1u, y2u) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
    if x1u >= x2u || y1u >= y2u {
        return DriveGaugeRead {
            value: 0.0,
            burnout: false,
            recovery: 0.0,
            uncertain: true,
        };
    }
    let x1 = x1u as usize;
    let x2 = x2u as usize;
    let y1 = y1u as usize;
    let roi_w = x2 - x1;
    let roi_h = y2u as usize - y1;
    let slope: f32 = if is_left {
        DRIVE_BAR_SLOPE
    } else {
        -DRIVE_BAR_SLOPE
    };
    let roi = SlantedRoi {
        rgba,
        frame_width: width as usize,
        x: x1..x2,
        y_start: y1,
        height: roi_h,
        strip_y: y_strip_start,
        slope,
    };

    // 全列をアンカー起点（index 0 = 画面中央側）で分類。
    // 左ゲージはアンカーが右端なので逆順、右ゲージは左端なのでそのまま。
    let classify = |column: usize| classify_drive_col(&roi, column);
    let mut cols: Vec<DriveColClass> = if is_left {
        (0..roi_w).rev().map(classify).collect()
    } else {
        (0..roi_w).map(classify).collect()
    };

    // 外縁リム装飾（ROI 末尾 ≈10px @1080p）は満タン時グロー/バーンアウト枠が
    // 不定に読めるため除外。6 セル実体は先頭 ≈314px に収まる。
    let span = roi_w * (324 - 10) / 324;
    cols.truncate(span.max(1));

    let runs = segment_drive_runs(&cols);
    decode_drive_runs(&runs, cols.len())
}
