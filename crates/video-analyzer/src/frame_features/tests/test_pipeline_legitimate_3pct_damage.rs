use super::support::*;

#[test]
fn test_pipeline_legitimate_3pct_damage() {
    // 正当な 3% ダメージ: HP 1.0 → 0.97 (stun あり, 3% 降下 = THRESHOLD に等しい)
    // STUN_DROP_THRESHOLD は strict greater than → 3% はチェックなしで通過するはず
    let mut features: Vec<FrameFeatures> = (0usize..20)
        .map(|i| {
            let hp = if i < 10 { 1.0f32 } else { 0.97f32 };
            make_frame(hp, false, true)
        })
        .collect();

    let stun: Vec<bool> = (0..20).map(|i| i >= 10).collect();
    correct_hp_side(&mut features, "p1", "left", &stun);

    for (offset, feature) in features[10..20].iter().enumerate() {
        let i = 10 + offset;
        assert!(
            (feature.own_hp - 0.97).abs() < 0.02,
            "3% damage frame {i} → 0.97, got {:.3}",
            feature.own_hp
        );
    }
}
