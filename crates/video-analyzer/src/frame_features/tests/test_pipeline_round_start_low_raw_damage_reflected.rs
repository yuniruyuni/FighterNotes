use super::support::*;

#[test]
fn test_pipeline_round_start_low_raw_damage_reflected() {
    // ラウンド開始直後の raw=0.9914（ROUND!オーバーレイで 0.86% 誤読）の後、
    // ダメージ（最大 6.45%: 0.9355）が own_hp に正しく反映されるか検証する。
    //
    // 再現シナリオ（実測 fi=582-700）:
    //   fi=0-99:   非試合フレーム（YOU WIN など）
    //   fi=100:    ラウンド開始フレーム（raw=0.9914, is_match=true）
    //   fi=101-64: raw=1.0（ダメージ前）
    //   fi=165-168: ダメージ (0.9828, 0.9599, 0.9527, 0.9355)
    //   fi=169-189: body overlap スパイク (raw=1.0)
    //   fi=190-200: 安定フレーム (raw=0.9355)
    let n = 210usize;
    let non_match_end = 100usize;
    let damage_start = 165usize;
    let damage_end = 169usize;
    let spike_end = 190usize;
    let damage_vals = [0.9828f32, 0.9599, 0.9527, 0.9355];

    let mut features: Vec<FrameFeatures> = (0..n)
        .map(|i| {
            let (hp, unc, is_match) = if i < non_match_end {
                (1.0f32, false, false)
            } else if i == non_match_end {
                (0.9914f32, false, true) // ラウンド開始フレーム（誤読）
            } else if i < damage_start {
                (1.0f32, false, true)
            } else if i < damage_end {
                (damage_vals[i - damage_start], false, true)
            } else if i < spike_end {
                (1.0f32, false, true) // body overlap スパイク
            } else {
                (0.9355f32, false, true) // ダメージ安定
            };
            let mut f = make_frame(hp, unc, is_match);
            f.frame_index = i as u32;
            f
        })
        .collect();

    let stun: Vec<bool> = (0..n).map(|i| i >= damage_start && i < spike_end).collect();
    correct_hp_side(&mut features, "p1", "left", &stun);

    // ダメージ後のフレームは 0.9355 前後になるはず
    for (offset, feature) in features[spike_end..n].iter().enumerate() {
        let i = spike_end + offset;
        assert!(
            feature.own_hp < 0.96,
            "damage should be reflected: frame {i} expected <0.96, got {:.4}",
            feature.own_hp
        );
    }
    // 最大ダメージフレームの own_hp が 0.9355 前後
    assert!(
        (features[damage_end - 1].own_hp - 0.9355).abs() < 0.05,
        "max damage frame expected ≈0.9355, got {:.4}",
        features[damage_end - 1].own_hp
    );
}
