use crate::frame_features::FrameFeatures;

const MIN_STABLE: usize = 8;
// value ±0.25（セル単位）/6。バーンアウト回復は約 0.003/f なので十分余裕がある。
const MAX_STEP: f32 = 0.25 / 6.0;

/// ドライブ読み取りの時間方向クリーニング。
///
/// 遮蔽体がゲージ外側だけを覆うと、単フレームでは正常なバーと区別できない
/// 短い偽値が生じる。連続8フレーム以上安定していない確定値を uncertain 化し、
/// 直前の信頼値で埋める。uncertain フラグ自体は保持する。
// Vec is retained as part of the public compatibility surface.
#[allow(clippy::ptr_arg)]
pub fn clean_drive_temporal(features: &mut Vec<FrameFeatures>) {
    let frame_count = features.len();
    for side in 0..2 {
        let get = |feature: &FrameFeatures| -> (f32, bool, bool) {
            if side == 0 {
                (
                    feature.left_drive_ratio,
                    feature.left_burnout,
                    feature.left_drive_uncertain,
                )
            } else {
                (
                    feature.right_drive_ratio,
                    feature.right_burnout,
                    feature.right_drive_uncertain,
                )
            }
        };
        let set = |feature: &mut FrameFeatures, ratio: f32, burnout: bool, uncertain: bool| {
            if side == 0 {
                feature.left_drive_ratio = ratio;
                feature.left_burnout = burnout;
                feature.left_drive_uncertain = uncertain;
            } else {
                feature.right_drive_ratio = ratio;
                feature.right_burnout = burnout;
                feature.right_drive_uncertain = uncertain;
            }
        };

        let mut start = 0;
        while start < frame_count {
            let (first_ratio, first_burnout, first_uncertain) = get(&features[start]);
            if first_uncertain || !features[start].is_match_screen {
                start = start.saturating_add(1);
                continue;
            }

            let mut end = frame_count;
            let mut previous_ratio = first_ratio;
            for (candidate, feature) in features.iter().enumerate().skip(start.saturating_add(1)) {
                let (ratio, burnout, uncertain) = get(feature);
                if uncertain
                    || !feature.is_match_screen
                    || burnout != first_burnout
                    || (ratio - previous_ratio).abs() > MAX_STEP
                {
                    end = candidate;
                    break;
                }
                previous_ratio = ratio;
            }
            if end - start < MIN_STABLE {
                for feature in &mut features[start..end] {
                    let (ratio, burnout, _) = get(feature);
                    set(feature, ratio, burnout, true);
                }
            }
            start = end;
        }

        // 直前に信用できた読み。型を明示しておく（推論だけに頼ると、
        // ソースを変換する解析ツールの下で解決できなくなる）。
        let mut last_trusted: Option<(f32, bool)> = None;
        for feature in features.iter_mut() {
            if !feature.is_match_screen {
                last_trusted = None;
                continue;
            }
            let (ratio, burnout, uncertain) = get(feature);
            if uncertain {
                if let Some((trusted_ratio, trusted_burnout)) = last_trusted {
                    set(feature, trusted_ratio, trusted_burnout, true);
                }
            } else {
                last_trusted = Some((ratio, burnout));
            }
        }
    }
}
