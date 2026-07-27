use super::support::*;

#[test]
fn test_backward_fill_trusted_false_low_corrected() {
    // trusted だが大幅急落（0.03）→ prev（1.0）でホールドされるか
    // パターン: [1.0 x5] [0.03 trusted x3] [0.97 x5]
    let n = 13usize;
    let in_match = vec![true; n];
    let in_spike = vec![false; n];
    let in_uncertain = vec![false; n];
    let mut corrected = vec![1.0f32; 5];
    corrected.extend_from_slice(&[0.03, 0.03, 0.03]);
    corrected.extend_from_slice(&[0.97f32; 5]);

    spike_hold_forward_pass(&mut corrected, &in_match, &in_spike, &in_uncertain, 0, n);

    // 偽ロー（0.03）は prev=1.0 でホールドされる（1.0 の 50% 未満 かつ 差 0.97 > 0.5）
    for (offset, &value) in corrected[5..8].iter().enumerate() {
        let i = 5 + offset;
        assert!(
            (value - 1.0).abs() < 0.01,
            "false-low frame {i} → held at 1.0, got {:.3}",
            value
        );
    }
    // その後 0.97（差 0.03 < 0.5）に下がる
    for (offset, &value) in corrected[8..n].iter().enumerate() {
        let i = 8 + offset;
        assert!(
            (value - 0.97).abs() < 0.01,
            "post-false-low frame {i} → 0.97, got {:.3}",
            value
        );
    }
}

// ── correct_hp_side 統合テスト ────────────────────────────────────────────
