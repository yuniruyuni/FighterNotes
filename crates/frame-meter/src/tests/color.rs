use crate::color::{bgr_to_hsv, dim_anchor, l2_dist, quantized_bucket, QuantizedModeScratch};

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
    let mut scratch = QuantizedModeScratch::new();
    assert_eq!(scratch.mean(&pixels), [17.0, 18.0, 19.0]);
    assert_eq!(scratch.mean(&[]), [0.0; 3]);
}

#[test]
fn quantized_mode_ties_use_the_lowest_bgr_bucket() {
    let pixels = [[248, 248, 248], [249, 249, 249], [0, 0, 0], [1, 1, 1]];
    assert_eq!(QuantizedModeScratch::new().mean(&pixels), [0.5, 0.5, 0.5]);
}

#[test]
fn quantized_mode_does_not_replace_a_lower_bucket_on_a_late_tie() {
    let pixels = [[0, 0, 0], [1, 1, 1], [248, 248, 248], [249, 249, 249]];
    assert_eq!(QuantizedModeScratch::new().mean(&pixels), [0.5, 0.5, 0.5]);
}

#[test]
fn quantized_bucket_keeps_asymmetric_channel_fields() {
    assert_eq!(quantized_bucket(&[161, 82, 43]), 20 * 32 * 32 + 10 * 32 + 5);
    assert_eq!(quantized_bucket(&[7, 8, 255]), 63);
}

#[test]
fn quantized_mode_scratch_clears_counts_between_calls() {
    let mut scratch = QuantizedModeScratch::new();
    assert_eq!(scratch.mean(&[[8, 8, 8]; 3]), [8.0, 8.0, 8.0]);
    assert_eq!(
        scratch.mean(&[[8, 8, 8], [16, 16, 16], [17, 17, 17]]),
        [16.5, 16.5, 16.5]
    );
}
