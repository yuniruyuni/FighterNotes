use super::support::*;

#[test]
fn test_backward_fill_uncertain_dip_corrected() {
    // uncertain フレーム (0.0) が prev（1.0）でホールドされるか
    // パターン: [1.0 x5] [0.0 uncertain x3] [0.97 x5]
    let n = 13usize;
    let in_match = vec![true; n];
    let in_spike = vec![false; n];
    let mut in_uncertain = vec![false; n];
    in_uncertain[5] = true;
    in_uncertain[6] = true;
    in_uncertain[7] = true;
    let mut corrected = vec![1.0f32; 5];
    corrected.extend_from_slice(&[0.0, 0.0, 0.0]);
    corrected.extend_from_slice(&[0.97f32; 5]);

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &in_uncertain, 0, n);

    // uncertain フレームは prev=1.0 でホールドされる
    for (offset, &value) in corrected[5..8].iter().enumerate() {
        let i = 5 + offset;
        assert!(
            (value - 1.0).abs() < 0.01,
            "uncertain frame {i} → held at 1.0, got {:.3}",
            value
        );
    }
    // その後 0.97 に下がる
    for (offset, &value) in corrected[8..n].iter().enumerate() {
        let i = 8 + offset;
        assert!(
            (value - 0.97).abs() < 0.01,
            "post-uncertain frame {i} → 0.97, got {:.3}",
            value
        );
    }
}
