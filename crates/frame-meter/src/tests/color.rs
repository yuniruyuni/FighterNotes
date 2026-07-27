use crate::color::{bgr_to_hsv, dim_anchor, l2_dist, quantized_mode_mean};

use super::assert_close;

#[test]
fn dim_anchor_scales_and_rounds_each_channel() {
    assert_eq!(dim_anchor([1.0, 2.0, 3.0]), [0.8, 1.5, 2.3]);
}

#[test]
fn hsv_conversion_covers_achromatic_and_each_max_channel() {
    for (bgr, expected) in [
        ([0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
        ([64.0, 64.0, 64.0], [0.0, 0.0, 64.0]),
        ([0.0, 0.0, 255.0], [0.0, 255.0, 255.0]),
        ([0.0, 255.0, 0.0], [60.0, 255.0, 255.0]),
        ([255.0, 0.0, 0.0], [120.0, 255.0, 255.0]),
        ([0.0, 255.0, 255.0], [30.0, 255.0, 255.0]),
    ] {
        let actual = bgr_to_hsv(bgr);
        for index in 0..3 {
            assert_close(actual[index], expected[index]);
        }
    }
}

#[test]
fn hsv_conversion_preserves_hue_at_epsilon_branch_boundaries() {
    let epsilon_channel = 0.000_255;
    assert_close(bgr_to_hsv([0.0, epsilon_channel, 0.0])[0], 60.0);
    assert_close(
        bgr_to_hsv([0.0, 2.0 * epsilon_channel, epsilon_channel])[0],
        45.0,
    );
    assert_close(
        bgr_to_hsv([2.0 * epsilon_channel, epsilon_channel, 0.0])[0],
        105.0,
    );
}

#[test]
fn euclidean_distance_uses_all_channels() {
    assert_close(l2_dist([1.0, 2.0, 3.0], [4.0, 6.0, 15.0]), 13.0);
}

#[test]
fn quantized_mode_uses_most_frequent_bucket_and_its_mean() {
    let pixels = [
        [8, 8, 8],
        [16, 17, 18],
        [17, 18, 19],
        [18, 19, 20],
        [40, 40, 40],
    ];
    assert_eq!(quantized_mode_mean(&pixels), [17.0, 18.0, 19.0]);
    assert_eq!(quantized_mode_mean(&[]), [0.0; 3]);
}
