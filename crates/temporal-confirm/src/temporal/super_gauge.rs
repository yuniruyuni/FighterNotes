use crate::frame_features::{FrameFeatures, MIN_SUPER_SPEND_DROP};

pub const SUPER_SPEND_CONFIRM_SAMPLES: usize = 12;
pub const SUPER_SPEND_CONFIRM_LOOKAHEAD: usize = 90;
const HIGHER_LEVEL_CONFIRM_FRAMES: usize = 12;
const HIGHER_LEVEL_LOOKAHEAD: usize = 45;
const MAX_GAIN_BASE: f32 = 0.45;
const MAX_GAIN_PER_VIDEO_FRAME: f32 = 0.003;

/// SA ゲージの単フレーム遮蔽と一時的な整数ラベル誤読を補正する。
///
/// 同じ整数ストック内でゲージが減ることはないため、少数部だけの逆行は
/// 直前値で埋める。整数部の低下（SA 消費）は、短い誤認を避けるため
/// 約 0.2 秒分の同じ低位ラベルが確認できた場合だけ受理する。全フレームを
/// 観測するため、ストック獲得時の数字アニメーションも一時的な低下として
/// 受理しない。
#[allow(clippy::ptr_arg)]
pub fn clean_super_temporal(features: &mut Vec<FrameFeatures>) {
    for side in 0..2 {
        clean_side(features, side);
    }
}

fn clean_side(features: &mut [FrameFeatures], side: usize) {
    let mut accepted: Option<(f32, bool, u32)> = None;
    for index in 0..features.len() {
        let (value, uncertain, ca_ready) = get(&features[index], side);
        let reliable = features[index].is_match_screen && !uncertain;
        if !reliable {
            if let Some((last_value, last_ca, _)) = accepted {
                set(&mut features[index], side, last_value, true, last_ca);
            }
            continue;
        }

        let frame = features[index].frame_index;
        let Some((previous, _, previous_frame)) = accepted else {
            accepted = Some((value, ca_ready, frame));
            continue;
        };
        let previous_level = stock_level(previous);
        let level = stock_level(value);

        let elapsed = frame.saturating_sub(previous_frame);
        let max_gain = MAX_GAIN_BASE + elapsed as f32 * MAX_GAIN_PER_VIDEO_FRAME;
        if value > previous + max_gain {
            set(&mut features[index], side, previous, true, ca_ready);
            continue;
        }
        if level > previous_level && !higher_level_is_stable(features, side, index, level) {
            set(&mut features[index], side, previous, true, ca_ready);
            continue;
        }
        if level == previous_level && value + 0.02 < previous {
            set(&mut features[index], side, previous, true, ca_ready);
            continue;
        }
        if level < previous_level && previous - value < MIN_SUPER_SPEND_DROP {
            set(&mut features[index], side, previous, true, ca_ready);
            continue;
        }
        if level < previous_level && !lower_level_is_stable(features, side, index, level) {
            set(&mut features[index], side, previous, true, ca_ready);
            continue;
        }

        accepted = Some((value, ca_ready, frame));
    }
}

fn lower_level_is_stable(
    features: &[FrameFeatures],
    side: usize,
    start: usize,
    expected_level: u8,
) -> bool {
    let mut confirmed = 0;
    for feature in features
        .iter()
        .skip(start)
        .take(SUPER_SPEND_CONFIRM_LOOKAHEAD)
    {
        let (value, uncertain, _) = get(feature, side);
        if !feature.is_match_screen || uncertain {
            continue;
        }
        if stock_level(value) != expected_level {
            return false;
        }
        confirmed += 1;
        if confirmed >= SUPER_SPEND_CONFIRM_SAMPLES {
            return true;
        }
    }
    false
}

fn higher_level_is_stable(
    features: &[FrameFeatures],
    side: usize,
    start: usize,
    expected_level: u8,
) -> bool {
    let mut confirmed = 0;
    for feature in features.iter().skip(start).take(HIGHER_LEVEL_LOOKAHEAD) {
        let (value, uncertain, _) = get(feature, side);
        if !feature.is_match_screen || uncertain {
            continue;
        }
        if stock_level(value) != expected_level {
            return false;
        }
        confirmed += 1;
        if confirmed >= HIGHER_LEVEL_CONFIRM_FRAMES {
            return true;
        }
    }
    false
}

fn stock_level(value: f32) -> u8 {
    value.clamp(0.0, 3.0).floor() as u8
}

fn get(feature: &FrameFeatures, side: usize) -> (f32, bool, bool) {
    if side == 0 {
        (
            feature.left_super_value,
            feature.left_super_uncertain,
            feature.left_ca_ready,
        )
    } else {
        (
            feature.right_super_value,
            feature.right_super_uncertain,
            feature.right_ca_ready,
        )
    }
}

fn set(feature: &mut FrameFeatures, side: usize, value: f32, uncertain: bool, ca_ready: bool) {
    if side == 0 {
        feature.left_super_value = value;
        feature.left_super_uncertain = uncertain;
        feature.left_ca_ready = ca_ready;
    } else {
        feature.right_super_value = value;
        feature.right_super_uncertain = uncertain;
        feature.right_ca_ready = ca_ready;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::tests::support::feature;

    fn series(values: &[(f32, bool)]) -> Vec<FrameFeatures> {
        series_for(0, values)
    }

    /// 指定した側のゲージだけを与える観測列。もう片方は読めていない
    /// ままにして、側の取り違えを見えるようにする。
    fn series_for(side: usize, values: &[(f32, bool)]) -> Vec<FrameFeatures> {
        values
            .iter()
            .enumerate()
            .map(|(index, &(value, uncertain))| {
                let mut frame = feature(index as u32, 1.0);
                frame.is_match_screen = true;
                set(&mut frame, side, value, uncertain, false);
                frame
            })
            .collect()
    }

    #[test]
    fn rejects_a_single_frame_integer_drop() {
        let mut features = series(&[
            (2.6, false),
            (2.6, false),
            (0.6, false),
            (2.6, false),
            (2.7, false),
        ]);
        clean_super_temporal(&mut features);
        assert_eq!(features[2].left_super_value, 2.6);
        assert!(features[2].left_super_uncertain);
    }

    #[test]
    fn rejects_non_consecutive_lower_level_reads() {
        let mut features = series(&[
            (3.0, false),
            (0.2, false),
            (3.0, false),
            (0.2, false),
            (0.2, false),
            (3.0, false),
        ]);
        clean_super_temporal(&mut features);
        assert_eq!(features[1].left_super_value, 3.0);
        assert!(features[1].left_super_uncertain);
    }

    #[test]
    fn accepts_a_stable_spend_after_an_uncertain_cinematic_gap() {
        let mut values = vec![(3.0, false), (0.0, true), (0.0, true)];
        values.extend(std::iter::repeat_n(
            (0.2, false),
            SUPER_SPEND_CONFIRM_SAMPLES,
        ));
        let mut features = series(&values);
        clean_super_temporal(&mut features);
        assert_eq!(features[1].left_super_value, 3.0);
        assert_eq!(features[3].left_super_value, 0.2);
        assert!(!features[3].left_super_uncertain);
    }

    #[test]
    fn accepts_a_spend_observed_before_a_long_cinematic_gap() {
        let mut values = vec![(3.0, false), (0.2, false)];
        values.extend(std::iter::repeat_n((0.0, true), 50));
        values.extend(std::iter::repeat_n(
            (0.2, false),
            SUPER_SPEND_CONFIRM_SAMPLES,
        ));
        let mut features = series(&values);
        clean_super_temporal(&mut features);
        assert_eq!(features[1].left_super_value, 0.2);
        assert!(!features[1].left_super_uncertain);
    }

    #[test]
    fn same_stock_fraction_cannot_move_backwards() {
        let mut features = series(&[(1.7, false), (1.3, false), (1.8, false), (1.9, false)]);
        clean_super_temporal(&mut features);
        assert_eq!(features[1].left_super_value, 1.7);
        assert!(features[1].left_super_uncertain);
        assert_eq!(features[2].left_super_value, 1.8);
    }

    #[test]
    fn stock_boundary_jitter_is_not_a_level_one_spend() {
        let mut features = series(&[(3.0, false), (2.94, false), (2.95, false), (2.96, false)]);
        clean_super_temporal(&mut features);
        assert_eq!(features[1].left_super_value, 3.0);
        assert!(features[1].left_super_uncertain);
        assert_eq!(features[3].left_super_value, 3.0);
    }

    #[test]
    fn isolated_higher_stock_read_does_not_create_a_following_spend() {
        let mut values = vec![(1.995, false), (3.0, false), (3.0, false)];
        values.extend(std::iter::repeat_n(
            (1.75, false),
            SUPER_SPEND_CONFIRM_SAMPLES,
        ));
        let mut features = series(&values);
        clean_super_temporal(&mut features);
        assert_eq!(features[1].left_super_value, 1.995);
        assert!(features[1].left_super_uncertain);
        assert_eq!(features.last().unwrap().left_super_value, 1.995);
    }

    #[test]
    fn ten_frame_higher_label_flash_does_not_create_a_following_spend() {
        let mut values = vec![(2.995, false)];
        values.extend(std::iter::repeat_n((3.0, false), 10));
        values.extend(std::iter::repeat_n(
            (2.23, false),
            SUPER_SPEND_CONFIRM_SAMPLES,
        ));
        let mut features = series(&values);
        clean_super_temporal(&mut features);
        assert_eq!(features[1].left_super_value, 2.995);
        assert!(features[1].left_super_uncertain);
        assert_eq!(features.last().unwrap().left_super_value, 2.995);
    }

    #[test]
    fn impossible_two_stock_refill_during_one_animation_is_rejected() {
        let mut values = vec![(3.0, false)];
        values.extend(std::iter::repeat_n(
            (0.2, false),
            SUPER_SPEND_CONFIRM_SAMPLES,
        ));
        let high_start = values.len();
        values.extend(std::iter::repeat_n(
            (3.0, false),
            HIGHER_LEVEL_CONFIRM_FRAMES,
        ));
        let low_start = values.len();
        values.extend(std::iter::repeat_n(
            (0.2, false),
            SUPER_SPEND_CONFIRM_SAMPLES,
        ));
        let mut features = series(&values);
        clean_super_temporal(&mut features);
        assert_eq!(features[high_start].left_super_value, 0.2);
        assert_eq!(
            features[high_start + HIGHER_LEVEL_CONFIRM_FRAMES - 1].left_super_value,
            0.2
        );
        assert!(features[high_start].left_super_uncertain);
        assert_eq!(features[low_start].left_super_value, 0.2);
    }

    #[test]
    fn impossible_two_stock_refill_after_a_short_hidden_gap_is_rejected() {
        let mut values = vec![(3.0, false)];
        values.extend(std::iter::repeat_n(
            (0.995, false),
            SUPER_SPEND_CONFIRM_SAMPLES,
        ));
        values.push((3.0, false));
        let mut features = series(&values);
        let last = features.len() - 1;
        for (index, feature) in features.iter_mut().enumerate() {
            feature.frame_index = if index == last {
                190
            } else {
                index as u32 * 10
            };
        }
        clean_super_temporal(&mut features);
        assert_eq!(features[last].left_super_value, 0.995);
        assert!(features[last].left_super_uncertain);
    }
    // ── 左右 ─────────────────────────────────────────────────────────────

    /// 同じ揺れは右側でも同じように均す。片側だけ手を入れた実装では、
    /// 相手の SA だけが読めない解析になる。
    #[test]
    fn the_right_side_is_cleaned_the_same_way() {
        let mut features = series_for(1, &[(2.6, false), (2.6, false), (0.6, false), (2.6, false)]);

        clean_super_temporal(&mut features);

        assert_eq!(features[2].right_super_value, 2.6);
        assert!(features[2].right_super_uncertain);
    }

    /// 一度の走査で両側を均す。
    #[test]
    fn both_sides_are_cleaned_in_one_pass() {
        let mut features = series(&[(2.6, false), (2.6, false), (0.6, false), (2.6, false)]);
        for (index, frame) in features.iter_mut().enumerate() {
            let value = if index == 2 { 0.4 } else { 1.8 };
            set(frame, 1, value, false, false);
        }

        clean_super_temporal(&mut features);

        assert_eq!(features[2].left_super_value, 2.6, "左を均していない");
        assert_eq!(features[2].right_super_value, 1.8, "右を均していない");
    }

    /// 片側の読みがもう片側へ混ざらない。
    #[test]
    fn one_side_does_not_leak_into_the_other() {
        let mut features = series_for(0, &[(3.0, false), (3.0, false), (3.0, false)]);

        clean_super_temporal(&mut features);

        assert_eq!(features[2].left_super_value, 3.0);
        assert_eq!(features[2].right_super_value, 0.0, "右へ左の値が漏れている");
    }

    // ── 読めなかったフレーム ─────────────────────────────────────────────

    /// 試合画面が映っていないフレームは、直前の確定値で埋める。ただし
    /// 「読めた」とは言わない。
    #[test]
    fn a_frame_off_the_match_screen_borrows_the_last_confirmed_value() {
        let mut features = series(&[(2.5, false), (2.5, false), (0.0, false), (2.5, false)]);
        features[2].is_match_screen = false;

        clean_super_temporal(&mut features);

        assert_eq!(features[2].left_super_value, 2.5);
        assert!(
            features[2].left_super_uncertain,
            "埋めた値を読めた扱いにしている"
        );
    }

    /// 埋めるときは CA の点灯も直前のものを引き継ぐ。読めていない
    /// フレームの点灯状態は信用できない。
    #[test]
    fn the_borrowed_value_carries_the_last_critical_art_flag() {
        let mut features = series(&[(2.5, false), (2.5, false), (0.0, false)]);
        for frame in &mut features[..2] {
            frame.left_ca_ready = true;
        }
        features[2].is_match_screen = false;
        features[2].left_ca_ready = false;

        clean_super_temporal(&mut features);

        assert!(features[2].left_ca_ready, "直前の点灯を引き継いでいない");
    }

    /// まだ一度も読めていないうちは、埋める値が無い。そのまま残す。
    #[test]
    fn nothing_is_borrowed_before_the_first_reliable_reading() {
        let mut features = series(&[(1.5, true), (1.5, true), (2.5, false)]);

        clean_super_temporal(&mut features);

        assert_eq!(
            features[0].left_super_value, 1.5,
            "読めない先頭を書き換えている"
        );
        assert_eq!(features[2].left_super_value, 2.5);
    }

    /// 読めなかったフレームに当たっても、その先を均すのをやめない。
    #[test]
    fn an_unreadable_frame_does_not_stop_the_pass() {
        let mut features = series(&[(2.5, false), (2.5, true), (2.5, false), (0.5, false)]);

        clean_super_temporal(&mut features);

        assert_eq!(
            features[3].left_super_value, 2.5,
            "途中で走査を打ち切っている"
        );
        assert!(features[3].left_super_uncertain);
    }

    // ── 増え方の上限 ─────────────────────────────────────────────────────

    /// ゲージは一瞬では埋まらない。飛び抜けた増加は読み違い。
    #[test]
    fn a_jump_larger_than_the_gauge_can_fill_is_rejected() {
        let mut features = series(&[(0.2, false), (0.6, false), (0.6, false)]);

        clean_super_temporal(&mut features);

        assert_eq!(features[1].left_super_value, 0.6, "普通の増加を弾いている");

        let mut leaping = series(&[(0.2, false), (0.7, false), (0.7, false)]);
        clean_super_temporal(&mut leaping);

        assert_eq!(
            leaping[1].left_super_value, 0.2,
            "飛び抜けた増加を受けている"
        );
        assert!(leaping[1].left_super_uncertain);
    }

    /// 間が空いていれば、その分だけ増えていてよい。長い演出の後で
    /// ゲージが伸びているのは普通のこと。
    #[test]
    fn a_long_gap_allows_a_larger_increase() {
        let mut features = series(&[(0.2, false), (0.7, false), (0.7, false)]);
        // 2 フレーム目までに 300 フレーム分の時間が経っている。
        features[1].frame_index = 300;
        features[2].frame_index = 301;

        clean_super_temporal(&mut features);

        assert_eq!(features[1].left_super_value, 0.7, "経った時間を見ていない");
    }

    // ── 上のストックの確認 ───────────────────────────────────────────────

    /// 上のストックを確かめる間、読めないフレームは数にも入れず、
    /// 打ち切りもしない。演出中の欠測でストック獲得を落とさない。
    #[test]
    fn unreadable_frames_neither_confirm_nor_break_the_higher_level() {
        let mut values = vec![(1.9, false)];
        for _ in 0..HIGHER_LEVEL_CONFIRM_FRAMES {
            values.push((2.1, false));
            values.push((0.0, true));
        }
        let mut features = series(&values);

        clean_super_temporal(&mut features);

        assert_eq!(
            features[1].left_super_value, 2.1,
            "欠測で確認を打ち切っている"
        );
    }

    /// 確認の途中で別のストックが見えたら、そこで打ち切る。
    #[test]
    fn a_different_level_during_confirmation_breaks_it() {
        let mut values = vec![(1.9, false), (2.1, false), (2.1, false), (1.9, false)];
        values.extend(std::iter::repeat_n(
            (2.1, false),
            HIGHER_LEVEL_CONFIRM_FRAMES,
        ));
        let mut features = series(&values);

        clean_super_temporal(&mut features);

        assert_eq!(features[1].left_super_value, 1.9, "揺れた確認を通している");
        assert!(features[1].left_super_uncertain);
    }
}
