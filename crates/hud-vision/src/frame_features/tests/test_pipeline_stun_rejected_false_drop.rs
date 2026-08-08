use super::support::*;

#[test]
fn test_pipeline_stun_rejected_false_drop() {
    // スタンなしで 3% 超の降下 → Phase 2 で hold されるか
    // HP 1.0 → raw 0.03 (trusted, stun なし) → hold されて 1.0 を維持
    let mut features: Vec<FrameFeatures> = (0usize..20)
        .map(|i| {
            let hp = if i < 10 { 1.0f32 } else { 0.03f32 };
            make_frame(hp, false, true)
        })
        .collect();

    let stun = vec![false; 20]; // スタンなし
    correct_hp_side(&mut features, "p1", "left", &stun);

    // スタンなしなので降下は拒否 → 1.0 を維持するはず
    for (offset, feature) in features[10..20].iter().enumerate() {
        let i = 10 + offset;
        assert!(
            (feature.own_hp - 1.0).abs() < 0.02,
            "no-stun false drop frame {i} → held at 1.0, got {:.3}",
            feature.own_hp
        );
    }
}
