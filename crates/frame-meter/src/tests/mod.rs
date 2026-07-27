mod classification;
mod color;
mod digits;
mod edge;
mod model;
mod palette;
mod rescue;

pub(super) fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

pub(super) fn bgr_from_hsv(h: f32, s: f32, v: f32) -> [f32; 3] {
    let hue = h * 2.0;
    let chroma = v * s / 255.0;
    let x = chroma * (1.0 - ((hue / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = v - chroma;
    let (r, g, b) = match hue {
        value if value < 60.0 => (chroma, x, 0.0),
        value if value < 120.0 => (x, chroma, 0.0),
        value if value < 180.0 => (0.0, chroma, x),
        value if value < 240.0 => (0.0, x, chroma),
        value if value < 300.0 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    [b + m, g + m, r + m]
}
