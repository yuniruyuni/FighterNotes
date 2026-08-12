use super::{model::DriveColClass, rgb_to_hsv, SlantedRoi};

/// ドライブゲージ斜め列 1 本を分類する。
///
/// バーは平行四辺形なので、アンカー側の列は下の行ほど ROI の外へ出る。
/// 全行を取れない列はバーではなくその外側を見ているので、色ではなく
/// 「測れていない」として返す。傾き 0.625 と高さ 18 行なら、これに
/// 当たるのはアンカーから 11 列。
pub(crate) fn classify_drive_col(roi: &SlantedRoi<'_>, column: usize) -> DriveColClass {
    // 回復バーは通常のゲージより細いので、灰色の割合はバーが占める行だけで測る。
    // ROI 全高で測ると、満了間近の回復バーでも閾値に届かない。
    let gray_rows = super::burnout_row_start(roi.height)..roi.height;

    let mut n_lit = 0usize;
    let mut n_gray = 0usize;
    let mut n_foreign = 0usize;
    let mut total = 0usize;
    let mut gray_total = 0usize;

    for row in 0..roi.height {
        let Some([r, g, b]) = roi.rgb_at(column, row, 0) else {
            continue;
        };
        total += 1;
        let in_gray_rows = gray_rows.contains(&row);
        if in_gray_rows {
            gray_total += 1;
        }

        let [h, s, v] = rgb_to_hsv(r, g, b);

        if s > 120.0 && v > 120.0 {
            if (15.0..=60.0).contains(&h) {
                n_lit += 1;
            } else {
                n_foreign += 1;
            }
        } else if s < 60.0 && v > 120.0 && v < 210.0 && in_gray_rows {
            n_gray += 1;
        }
    }

    if total < roi.height {
        return DriveColClass::Outside;
    }
    let t = total as f32;
    if n_lit as f32 / t >= 0.35 {
        return DriveColClass::Lit;
    }
    if n_foreign as f32 / t >= 0.35 {
        return DriveColClass::Foreign;
    }
    if gray_total > 0 && n_gray as f32 / gray_total as f32 >= 0.40 {
        return DriveColClass::Gray;
    }
    DriveColClass::Rest
}

/// 同値の連続区間 (class, start, end)。
pub(crate) fn segment_drive_runs(cols: &[DriveColClass]) -> Vec<(DriveColClass, usize, usize)> {
    let mut runs = Vec::new();
    if cols.is_empty() {
        return runs;
    }
    let mut cur = cols[0];
    let mut start = 0usize;
    for (i, &c) in cols.iter().enumerate().skip(1) {
        if c != cur {
            runs.push((cur, start, i - 1));
            cur = c;
            start = i;
        }
    }
    runs.push((cur, start, cols.len() - 1));
    runs
}
