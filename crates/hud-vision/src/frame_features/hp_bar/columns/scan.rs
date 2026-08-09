//! HP バー ROI の列ごと走査。
//!
//! HP バーは平行四辺形で、行が下がるほど横へずれる。どの色を探すときも
//! 走査そのものは同じで、違うのは「その画素が探している色か」と
//! 「列の何割がその色なら、その列はそう見なすか」だけ。
//!
//! 同じ走査を色ごとに書き写していたため、添字計算の誤りが四箇所へ
//! 散らばりうる状態だった。走査はここ一箇所に置く。

use super::super::{hp_roi_base, HP_BAR_SLOPE, HP_COL_ROW_SKIP_BOTTOM, HP_COL_ROW_SKIP_TOP};
use crate::frame_features::scale_roi;

/// 走査する平行四辺形の位置と傾き。
pub(crate) struct ColumnScan {
    x1: usize,
    x2: usize,
    y1: usize,
    /// 上下のふちどりを除いた走査対象の行範囲（ROI 内の相対位置）。
    row_start: usize,
    row_end: usize,
    /// 1 行下がるごとの横ずれ。左右のバーで向きが逆になる。
    slope: f32,
    width: usize,
    /// ストリップ運用時の先頭行。全画面なら 0。
    y_strip_start: usize,
}

impl ColumnScan {
    /// 画面寸法と側から走査範囲を決める。ROI が潰れている場合は None。
    pub(crate) fn new(width: u32, height: u32, side: &str, y_strip_start: usize) -> Option<Self> {
        let (x1_base, x2_base, y1_base, y2_base) = hp_roi_base(side);
        let (x1, x2, y1, y2) = scale_roi(x1_base, x2_base, y1_base, y2_base, width, height);
        if x1 >= x2 || y1 >= y2 {
            return None;
        }
        let roi_h = (y2 - y1) as usize;
        let row_start = HP_COL_ROW_SKIP_TOP.min(roi_h);
        let row_end = roi_h.saturating_sub(HP_COL_ROW_SKIP_BOTTOM).max(row_start);
        Some(Self {
            x1: x1 as usize,
            x2: x2 as usize,
            y1: y1 as usize,
            row_start,
            row_end,
            slope: if side == "p1" {
                HP_BAR_SLOPE
            } else {
                -HP_BAR_SLOPE
            },
            width: width as usize,
            y_strip_start,
        })
    }

    /// ROI の横幅。列数と一致する。
    pub(crate) fn columns(&self) -> usize {
        self.x2 - self.x1
    }

    /// 列 `cx` の各画素を訪ね、ROI 内に収まった画素数と、`is_wanted` が
    /// 真を返した画素数を返す `(該当数, 有効数)`。
    ///
    /// バッファの外へ出る画素は有効数にも入れない。切り詰められた入力で
    /// 割合が歪まないようにするため。
    pub(crate) fn count_in_column(
        &self,
        rgba: &[u8],
        cx: usize,
        mut is_wanted: impl FnMut(f32, f32, f32) -> bool,
    ) -> (usize, usize) {
        let mut matched = 0usize;
        let mut effective = 0usize;
        for ry in self.row_start..self.row_end {
            let x_offset = ((ry - self.row_start) as f32 * self.slope).round() as i32;
            let gx = self.x1 as i32 + cx as i32 + x_offset;
            if gx < self.x1 as i32 || gx >= self.x2 as i32 {
                continue;
            }
            let gy = self.y1 + ry;
            let Some(row) = gy.checked_sub(self.y_strip_start) else {
                continue;
            };
            let index = (row * self.width + gx as usize) * 4;
            let Some(pixel) = rgba.get(index..index + 3) else {
                continue;
            };
            effective += 1;
            if is_wanted(pixel[0] as f32, pixel[1] as f32, pixel[2] as f32) {
                matched += 1;
            }
        }
        (matched, effective)
    }

    /// 各列について、`is_wanted` に当たった画素が `ratio` を超えるかを返す。
    pub(crate) fn columns_where(
        &self,
        rgba: &[u8],
        ratio: f32,
        mut is_wanted: impl FnMut(f32, f32, f32) -> bool,
    ) -> Vec<bool> {
        (0..self.columns())
            .map(|cx| {
                let (matched, effective) = self.count_in_column(rgba, cx, &mut is_wanted);
                effective > 0 && (matched as f32 / effective as f32) > ratio
            })
            .collect()
    }
}

#[cfg(test)]
mod tests;
