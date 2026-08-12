use super::{
    classification::{classify_drive_col, segment_drive_runs},
    decode::decode_drive_runs,
    model::DriveColClass,
    scale_roi, SlantedRoi, DRIVE_BAR_SLOPE, DRIVE_ROI_LEFT, DRIVE_ROI_RIGHT,
};

/// ドライブゲージのデバッグ情報を JSON で返す（examples/debug_hp_bar 用）。
pub fn drive_bar_debug_json(rgba: &[u8], width: u32, height: u32, side: &str) -> String {
    let is_left = side == "left";
    let (x1_base, x2_base, y1_base, y2_base) = if is_left {
        DRIVE_ROI_LEFT
    } else {
        DRIVE_ROI_RIGHT
    };
    let (x1u, x2u, y1u, y2u) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
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
        strip_y: 0,
        slope,
    };
    let classify = |column: usize| classify_drive_col(&roi, column);
    // 画面順の全列分類（可視化用、リムクリップ前）
    let cols_screen: Vec<DriveColClass> = (0..roi_w).map(classify).collect();
    // アンカー正規化 + リムクリップ（デコード用、drive_gauge_read_impl と同一）
    let mut cols: Vec<DriveColClass> = if is_left {
        cols_screen.iter().rev().copied().collect()
    } else {
        cols_screen.clone()
    };
    let span = roi_w * (324 - 10) / 324;
    cols.truncate(span.max(1));
    let runs = segment_drive_runs(&cols);
    let d = decode_drive_runs(&runs, cols.len());

    fn class_name(c: DriveColClass) -> &'static str {
        match c {
            DriveColClass::Lit => "Lit",
            DriveColClass::Gray => "Gray",
            DriveColClass::Foreign => "Foreign",
            DriveColClass::Rest => "Rest",
            DriveColClass::Outside => "Outside",
        }
    }
    fn class_char(c: DriveColClass) -> char {
        match c {
            DriveColClass::Lit => 'L',
            DriveColClass::Gray => 'G',
            DriveColClass::Foreign => 'F',
            DriveColClass::Rest => '.',
            DriveColClass::Outside => 'o',
        }
    }
    let runs_json: Vec<String> = runs
        .iter()
        .map(|&(c, s, e)| {
            format!(
                r#"{{"c":"{}","s":{},"e":{},"w":{}}}"#,
                class_name(c),
                s,
                e,
                e - s + 1
            )
        })
        .collect();
    let cols_str: String = cols_screen.iter().map(|&c| class_char(c)).collect();

    format!(
        r#"{{"value":{:.3},"burnout":{},"recovery":{:.3},"uncertain":{},"roi":{{"x1":{},"x2":{},"y1":{},"y2":{},"slope":{}}},"cols":"{}","runs":[{}]}}"#,
        d.value,
        d.burnout,
        d.recovery,
        d.uncertain,
        x1,
        x2,
        y1,
        y2u,
        slope,
        cols_str,
        runs_json.join(","),
    )
}
