use super::support::*;

#[test]
fn test_pipeline_uncertain_prolonged_zero_corrected() {
    // フレーム 4079 シナリオ: uncertain かつ raw=0 が大量に続いた後、次ラウンドが始まる。
    // backward_fill は後続 trusted フレームが 0 のため修正不可。
    // Phase 3 修正前は prev=0 が後続フレームに伝播。
    // Phase 4 修正後は uncertain&&corrected≈0 フレームを next_hp で補完する。
    //
    //   フレーム 0-99:   is_match=T, raw=1.0, uncertain=F (HP 100%)
    //   フレーム 100-149: is_match=T, raw=0.0, uncertain=T (HP バー消失演出)
    //   フレーム 150-199: is_match=F (非試合)
    //   フレーム 200-299: is_match=T, raw=1.0, uncertain=F (次ラウンド)
    let mut features: Vec<FrameFeatures> = (0usize..300)
        .map(|i| {
            if i < 100 {
                make_frame(1.0, false, true)
            } else if i < 150 {
                make_frame(0.0, true, true)
            } else if i < 200 {
                make_frame(0.0, false, false)
            } else {
                make_frame(1.0, false, true)
            }
        })
        .collect();

    correct_hp_side(&mut features, "p1", "left", &[]);

    // フレーム 0-99: HP 1.0 のまま
    for (i, feature) in features[..100].iter().enumerate() {
        assert!(
            (feature.own_hp - 1.0).abs() < 0.02,
            "frame {i}: expected ~1.0, got {:.3}",
            feature.own_hp
        );
    }
    // フレーム 100-149 (uncertain&&raw=0): 次ラウンド HP = 1.0 に補完される
    for (offset, feature) in features[100..150].iter().enumerate() {
        let i = 100 + offset;
        assert!(
            feature.own_hp > 0.95,
            "frame {i}: uncertain_zero should be filled to next_hp, got {:.3}",
            feature.own_hp
        );
    }
    // フレーム 200-299: HP 1.0 のまま
    for (offset, feature) in features[200..300].iter().enumerate() {
        let i = 200 + offset;
        assert!(
            (feature.own_hp - 1.0).abs() < 0.02,
            "frame {i}: expected ~1.0, got {:.3}",
            feature.own_hp
        );
    }
}
