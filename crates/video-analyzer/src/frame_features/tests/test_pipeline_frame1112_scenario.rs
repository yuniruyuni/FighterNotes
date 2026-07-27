use super::support::*;

#[test]
fn test_pipeline_frame1112_scenario() {
    // frame 1112 の完全再現シナリオ:
    // - 多数フレームで HP ≈ 97% (既に 3% ダメージあり)
    // - frame 1112-1119: 爆発エフェクトで raw=0.03 (trusted, 誤検知)
    //   → spike_hold で直前 HP（0.97）にホールドされる
    // - frame 1120+: HP 94% (3% ダメージが確認できる)
    let n = 1200usize;
    let hit = 1112usize;
    let expl_end = 1120usize;

    let mut features: Vec<FrameFeatures> = (0..n)
        .map(|i| {
            let (hp, unc) = if i < hit {
                (0.97f32, false)
            } else if i < expl_end {
                (0.03f32, false)
            }
            // 誤検知
            else {
                (0.94f32, false)
            };
            make_frame(hp, unc, true)
        })
        .collect();

    let stun: Vec<bool> = (0..n).map(|i| i >= hit && i < expl_end).collect();
    correct_hp_side(&mut features, "p1", "left", &stun);

    // ダメージ前: 0.97 を保持
    for (i, feature) in features[..hit].iter().enumerate() {
        assert!(
            (feature.own_hp - 0.97).abs() < 0.02,
            "pre-hit frame {i} → 0.97, got {:.3}",
            feature.own_hp
        );
    }
    // 爆発中: 直前 HP（0.97）でホールドされる（HP バーが見えないため）
    for (offset, feature) in features[hit..expl_end].iter().enumerate() {
        let i = hit + offset;
        assert!(
            (feature.own_hp - 0.97).abs() < 0.02,
            "hit frame {i} → held at 0.97, got {:.3}",
            feature.own_hp
        );
    }
    // 爆発後: 0.94 に下がる
    for (offset, feature) in features[expl_end..n].iter().enumerate() {
        let i = expl_end + offset;
        assert!(
            (feature.own_hp - 0.94).abs() < 0.02,
            "post-hit frame {i} → 0.94, got {:.3}",
            feature.own_hp
        );
    }
}
