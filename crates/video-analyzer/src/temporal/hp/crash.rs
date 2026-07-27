/// 1フレームでこれを超える HP 低下は、画面遷移によるバーワイプを疑う。
const CRASH_STEP: f32 = 0.12;
/// 急落が実 K.O. として確定するのに必要な低値持続フレーム数。
pub(super) const CRASH_CONFIRM: usize = 60;

/// 急落が短時間で跳ね返る場合、直前値で埋めて画面遷移ノイズを除去する。
pub(super) fn reject_hp_crashes(values: &mut [f32], match_frames: &[bool]) {
    for index in 1..values.len() {
        if values[index] >= 0.0 && values[index] < values[index - 1] - CRASH_STEP {
            let base = values[index - 1];
            let crashed = values[index];
            let mut seen = 0;
            let mut bounced = false;
            for (&value, &is_match) in values[index..].iter().zip(&match_frames[index..]) {
                if is_match {
                    seen += 1;
                    if value > crashed + 0.10 {
                        bounced = true;
                        break;
                    }
                    if seen == CRASH_CONFIRM {
                        break;
                    }
                }
            }

            if bounced || seen < CRASH_CONFIRM / 2 {
                for value in &mut values[index..] {
                    if *value < 0.0 || *value > base - CRASH_STEP {
                        break;
                    }
                    *value = base;
                }
            }
        }
    }
}
