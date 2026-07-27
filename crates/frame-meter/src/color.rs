use std::collections::BTreeMap;

pub(crate) type Bgr = [f32; 3];

pub(crate) fn dim_anchor(bgr: Bgr) -> Bgr {
    [
        (bgr[0] * 0.75 * 10.0).round() / 10.0,
        (bgr[1] * 0.75 * 10.0).round() / 10.0,
        (bgr[2] * 0.75 * 10.0).round() / 10.0,
    ]
}

pub(crate) fn bgr_to_hsv(bgr: Bgr) -> [f32; 3] {
    let b = bgr[0] / 255.0;
    let g = bgr[1] / 255.0;
    let r = bgr[2] / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let v = max * 255.0;
    let s = if max > 0.0 { delta / max * 255.0 } else { 0.0 };
    let h = if delta < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * ((g - b) / delta).rem_euclid(6.0)
    } else if (max - g).abs() < 1e-6 {
        60.0 * ((b - r) / delta + 2.0)
    } else {
        60.0 * ((r - g) / delta + 4.0)
    };
    [(h / 2.0).round(), s, v]
}

pub(crate) fn l2_dist(a: Bgr, b: Bgr) -> f32 {
    let db = a[0] - b[0];
    let dg = a[1] - b[1];
    let dr = a[2] - b[2];
    (db * db + dg * dg + dr * dr).sqrt()
}

pub(crate) fn quantized_mode_mean(pixels: &[[u8; 3]]) -> Bgr {
    if pixels.is_empty() {
        return [0.0, 0.0, 0.0];
    }
    let mut counts: BTreeMap<u32, usize> = BTreeMap::new();
    for px in pixels {
        let key = (px[0] as u32 / 8) * 10000 + (px[1] as u32 / 8) * 100 + (px[2] as u32 / 8);
        *counts.entry(key).or_insert(0) += 1;
    }
    let max_count = *counts.values().max().expect("non-empty color counts");
    let best_key = *counts
        .iter()
        .find(|(_, count)| **count == max_count)
        .expect("mode color")
        .0;
    let mut sum = [0.0f32; 3];
    let mut count = 0usize;
    for px in pixels {
        let key = (px[0] as u32 / 8) * 10000 + (px[1] as u32 / 8) * 100 + (px[2] as u32 / 8);
        if key == best_key {
            sum[0] += px[0] as f32;
            sum[1] += px[1] as f32;
            sum[2] += px[2] as f32;
            count += 1;
        }
    }
    [
        sum[0] / count as f32,
        sum[1] / count as f32,
        sum[2] / count as f32,
    ]
}
