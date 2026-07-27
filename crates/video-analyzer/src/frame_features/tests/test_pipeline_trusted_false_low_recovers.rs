use super::support::*;

#[test]
fn test_pipeline_trusted_false_low_recovers() {
    // シナリオ: HP 100% → 爆発エフェクト(trusted, raw=0.03 誤検知) → HP 94%
    // spike_hold では大幅急落（50%超）を偽ローとして直前 HP でホールドする
    let mut features: Vec<FrameFeatures> = (0usize..30)
        .map(|i| {
            let (hp, unc) = if i < 10 {
                (1.00f32, false)
            } else if i < 16 {
                (0.03f32, false)
            }
            // trusted だが誤検知
            else {
                (0.94f32, false)
            };
            make_frame(hp, unc, true)
        })
        .collect();

    let stun: Vec<bool> = (0..30).map(|i| (10..16).contains(&i)).collect();
    correct_hp_side(&mut features, "p1", "left", &stun);

    // 偽ロー（0.03）は prev=1.0 でホールドされる
    for (offset, feature) in features[10..16].iter().enumerate() {
        let i = 10 + offset;
        assert!(
            (feature.own_hp - 1.0).abs() < 0.02,
            "false-low frame {i} → held at 1.0, got {:.3}",
            feature.own_hp
        );
    }
    // エフェクト後: 0.94 に下がる
    for (offset, feature) in features[16..30].iter().enumerate() {
        let i = 16 + offset;
        assert!(
            (feature.own_hp - 0.94).abs() < 0.02,
            "post-damage frame {i} → 0.94, got {:.3}",
            feature.own_hp
        );
    }
}
