use super::support::*;

#[test]
fn test_pipeline_uncertain_high_before_false_low() {
    // フレーム 1112 シナリオ:
    // エフェクトが HP バーを覆い vivid 色が消えるが uncertain=False のまま raw≈0 になる。
    // 直前に uncertain=True の高い raw 値（真の HP を反映）が続いていた場合、
    // Phase 1.5（前方急落補正）が uncertain=True 値を prev_uncertain_hp に記録し、
    // 後続の uncertain=False 急落フレームをその値で補正する。
    //
    //   フレーム 0-10:   is_match=T, raw=1.0, uncertain=F  (通常 HP)
    //   フレーム 11-20:  is_match=T, raw=0.993, uncertain=T (体が重なって islands)
    //   フレーム 21-80:  is_match=T, raw=0.006, uncertain=F (エフェクト偽ロー)
    //   フレーム 81-99:  is_match=F                          (非試合)
    let mut features: Vec<FrameFeatures> = (0usize..100)
        .map(|i| {
            if i <= 10 {
                make_frame(1.0, false, true)
            } else if i <= 20 {
                make_frame(0.993, true, true) // uncertain=True, high raw
            } else if i <= 80 {
                make_frame(0.006, false, true) // uncertain=False, very low raw (false low)
            } else {
                make_frame(0.0, false, false)
            }
        })
        .collect();

    correct_hp_side(&mut features, "p1", "left", &[]);

    // フレーム 0-20 は HP ≈ 1.0 のまま（backward_fill で uncertain 分も補正される）
    for (i, feature) in features[..=20].iter().enumerate() {
        assert!(
            feature.own_hp > 0.95,
            "frame {i}: expected ~1.0, got {:.3}",
            feature.own_hp
        );
    }
    // フレーム 21-80: Phase 1.5 が直前の uncertain=True raw=0.993 を prev_uncertain_hp として記録し
    // 急落（0.993→0.006 = 99.4% 減）を検出して 0.993 に補正する
    for (offset, feature) in features[21..=80].iter().enumerate() {
        let i = 21 + offset;
        assert!(
            feature.own_hp > 0.95,
            "frame {i}: false-low (raw=0.006) should be corrected to ~0.993 by Phase 1.5, got {:.3}",
            feature.own_hp
        );
    }
}
