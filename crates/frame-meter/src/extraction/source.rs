use std::collections::BTreeSet;

use crate::color::bgr_to_hsv;
use crate::constants::{REGION_REF_ROWS, ROW_X1, ROW_X2};

pub(crate) struct RowSource<'a> {
    rgba: &'a [u8],
    width: i32,
    height: i32,
    scale_x: f32,
    scale_y: f32,
    strip_y: i32,
}

impl<'a> RowSource<'a> {
    pub(crate) fn new(rgba: &'a [u8], width: u32, height: u32, strip_y: i32) -> Self {
        Self {
            rgba,
            width: width as i32,
            height: height as i32,
            scale_x: width as f32 / 1920.0,
            scale_y: height as f32 / 1080.0,
            strip_y,
        }
    }

    pub(crate) fn read_row(
        &self,
        y1: i32,
        y2: i32,
        region1_reference: &[usize],
        region2_reference: &[usize],
    ) -> Option<RowPixels> {
        if self.width <= 0 {
            return None;
        }
        if self.height <= 0 {
            return None;
        }
        let x1 = ((ROW_X1 as f32 * self.scale_x) as i32).max(0);
        let x2 = ((ROW_X2 as f32 * self.scale_x) as i32)
            .max(0)
            .min(self.width);
        let y1 = ((y1 as f32 * self.scale_y) as i32).max(0);
        let y2 = ((y2 as f32 * self.scale_y) as i32).max(0).min(self.height);
        let height = (y2 - y1).max(0) as usize;
        let width = (x2 - x1).max(0) as usize;
        if height == 0 || width == 0 {
            return None;
        }

        let trim_y = (height / 6).max(1);
        let patch_height = height.saturating_sub(2 * trim_y);
        if patch_height == 0 {
            return None;
        }

        let mut bgr = vec![[0; 3]; width * height];
        let mut value = vec![0.0; width * height];
        let mut saturation = vec![0.0; width * height];
        for row in 0..height {
            let strip_y = y1 + row as i32 - self.strip_y;
            if strip_y < 0 {
                continue;
            }
            for column in 0..width {
                let global_x = x1 as usize + column;
                let source_index = (strip_y as usize * self.width as usize + global_x) * 4;
                if source_index + 3 >= self.rgba.len() {
                    continue;
                }
                let red = self.rgba[source_index];
                let green = self.rgba[source_index + 1];
                let blue = self.rgba[source_index + 2];
                let target_index = row * width + column;
                bgr[target_index] = [blue, green, red];
                let hsv = bgr_to_hsv([blue as f32, green as f32, red as f32]);
                value[target_index] = hsv[2];
                saturation[target_index] = hsv[1];
            }
        }

        Some(RowPixels {
            width,
            height,
            trim_y,
            patch_height,
            region1_rows: scale_rows(region1_reference, patch_height),
            region2_rows: scale_rows(region2_reference, patch_height),
            bgr,
            value,
            saturation,
        })
    }
}

pub(crate) struct RowPixels {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) trim_y: usize,
    pub(crate) patch_height: usize,
    pub(crate) region1_rows: Vec<usize>,
    pub(crate) region2_rows: Vec<usize>,
    pub(crate) bgr: Vec<[u8; 3]>,
    pub(crate) value: Vec<f32>,
    pub(crate) saturation: Vec<f32>,
}

fn scale_rows(rows: &[usize], patch_height: usize) -> Vec<usize> {
    let mut scaled_rows = BTreeSet::new();
    for &row in rows {
        let scaled = ((row * patch_height) as f32 / REGION_REF_ROWS as f32).round() as usize;
        scaled_rows.insert(scaled.min(patch_height.saturating_sub(1)));
    }
    scaled_rows.into_iter().collect()
}
