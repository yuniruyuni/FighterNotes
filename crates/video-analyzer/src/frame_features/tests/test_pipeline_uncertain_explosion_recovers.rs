use super::support::*;

#[test]
fn test_pipeline_uncertain_explosion_recovers() {
    // シナリオ: HP 100% → 爆発エフェクト(uncertain=true, raw=0.03) → HP 94%
    // backward_fill が uncertain フレームを 0.94 に補正し、Phase3 で単調を維持する
    let mut features: Vec<FrameFeatures> = (0usize..30)
        .map(|i| {
            let (hp, unc) = if i < 10 {
                (1.00f32, false)
            } else if i < 16 {
                (0.03f32, true)
            } else {
                (0.94f32, false)
            };
            make_frame(hp, unc, true)
        })
        .collect();

    let stun: Vec<bool> = (0..30).map(|i| (10..16).contains(&i)).collect();
    correct_hp_side(&mut features, "p1", "left", &stun);

    // 爆発中（uncertain）: 直前の HP（1.0）でホールドされる
    for (offset, feature) in features[10..16].iter().enumerate() {
        let i = 10 + offset;
        assert!(
            (feature.own_hp - 1.0).abs() < 0.02,
            "explosion frame {i} → held at 1.0, got {:.3}",
            feature.own_hp
        );
    }
    // 爆発後: 0.94 に下がる
    for (offset, feature) in features[16..30].iter().enumerate() {
        let i = 16 + offset;
        assert!(
            (feature.own_hp - 0.94).abs() < 0.02,
            "post-explosion frame {i} → 0.94, got {:.3}",
            feature.own_hp
        );
    }
}
