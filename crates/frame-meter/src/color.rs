pub(crate) type Bgr = [f32; 3];

const QUANTIZED_CHANNEL_BITS: usize = 5;
const QUANTIZED_BUCKET_COUNT: usize = 1 << (QUANTIZED_CHANNEL_BITS * 3);

pub(crate) struct QuantizedModeScratch {
    counts: Vec<u32>,
    touched: Vec<u16>,
}

impl QuantizedModeScratch {
    pub(crate) fn new() -> Self {
        Self {
            counts: vec![0; QUANTIZED_BUCKET_COUNT],
            touched: Vec::new(),
        }
    }

    pub(crate) fn mean(&mut self, pixels: &[[u8; 3]]) -> Bgr {
        if pixels.is_empty() {
            return [0.0, 0.0, 0.0];
        }

        let mut best_bucket = usize::MAX;
        let mut best_count = 0u32;
        for px in pixels {
            let bucket = quantized_bucket(px);
            let count = &mut self.counts[bucket];
            self.touched.push(bucket as u16);
            *count += 1;
            if *count > best_count || (*count == best_count && bucket.cmp(&best_bucket).is_lt()) {
                best_bucket = bucket;
                best_count = *count;
            }
        }

        let mut sum = [0.0f32; 3];
        for px in pixels {
            if quantized_bucket(px) == best_bucket {
                sum[0] += px[0] as f32;
                sum[1] += px[1] as f32;
                sum[2] += px[2] as f32;
            }
        }
        for bucket in self.touched.drain(..) {
            self.counts[bucket as usize] = 0;
        }

        [
            sum[0] / best_count as f32,
            sum[1] / best_count as f32,
            sum[2] / best_count as f32,
        ]
    }
}

pub(crate) fn quantized_bucket(px: &[u8; 3]) -> usize {
    let blue = usize::from(px[0] >> 3);
    let green = usize::from(px[1] >> 3);
    let red = usize::from(px[2] >> 3);
    (blue * 32 + green) * 32 + red
}

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
