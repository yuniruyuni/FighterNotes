//! Confirmed recoverable-HP restoration.
//!
//! Raw HP occasionally rises because of perception noise, so ordinary upward
//! movement must still be rejected by the monotonic pass.  Armor damage is
//! different: the bar stays below its previous level, then visibly recovers in
//! several reliable steps over a few seconds.  Only that strong temporal shape
//! is restored here, before permanent HP is made monotonic.

const MIN_DROP: f32 = 0.02;
const RETURN_TOLERANCE: f32 = 0.015;
const MIN_RETURN_RATIO: f32 = 0.70;
const MIN_RECOVERY_DELAY: usize = 30;
const MAX_RECOVERY_WINDOW: usize = 600;
const MIN_UPWARD_STEPS: usize = 3;
const MIN_UPWARD_STEP: f32 = 0.001;
const RECOVERY_CONFIRM_RUN: usize = 8;
const MAX_UNOBSERVED_EXTENSION: usize = 10;

pub(super) fn restore_confirmed_recoveries(
    values: &mut [f32],
    source: &[f32],
    match_frames: &[bool],
    reset_at: &[bool],
) {
    if values.is_empty()
        || source.len() != values.len()
        || match_frames.len() != values.len()
        || reset_at.len() != values.len()
    {
        return;
    }

    let mut previous = None;
    let mut index = 0;
    while index < values.len() {
        if reset_at[index] {
            previous = None;
        }
        let value = values[index];
        if value < 0.0 || !match_frames[index] {
            index += 1;
            continue;
        }
        let Some(baseline) = previous else {
            previous = Some(value);
            index += 1;
            continue;
        };

        if baseline - value < MIN_DROP {
            previous = Some(baseline.min(value));
            index += 1;
            continue;
        }

        if let Some(end) =
            confirmed_recovery_end(values, source, match_frames, reset_at, index, baseline)
        {
            for value in &mut values[index..=end] {
                if *value >= 0.0 {
                    *value = baseline;
                }
            }
            previous = Some(baseline);
            index = end + 1;
        } else {
            previous = Some(value);
            index += 1;
        }
    }
}

fn confirmed_recovery_end(
    values: &[f32],
    source: &[f32],
    match_frames: &[bool],
    reset_at: &[bool],
    drop_start: usize,
    baseline: f32,
) -> Option<usize> {
    let hard_end = drop_start
        .saturating_add(MAX_RECOVERY_WINDOW)
        .min(values.len().saturating_sub(1));
    let mut low = reliable_source(source, drop_start).unwrap_or(values[drop_start]);
    let mut previous_observed = None;
    let mut upward_steps = 0;
    let mut return_run = 0;

    for index in drop_start..=hard_end {
        if (index > drop_start && reset_at[index]) || !match_frames[index] {
            break;
        }
        let Some(value) = reliable_source(source, index) else {
            return_run = 0;
            continue;
        };

        low = low.min(value);
        if previous_observed.is_some_and(|previous| value >= previous + MIN_UPWARD_STEP) {
            upward_steps += 1;
        }
        previous_observed = Some(value);

        let drop = baseline - low;
        let returned = value - low;
        let near_baseline = baseline - value <= RETURN_TOLERANCE;
        let enough_return = drop >= MIN_DROP && returned >= drop * MIN_RETURN_RATIO;
        if index - drop_start >= MIN_RECOVERY_DELAY
            && near_baseline
            && enough_return
            && upward_steps >= MIN_UPWARD_STEPS
        {
            return_run += 1;
            if return_run >= RECOVERY_CONFIRM_RUN {
                return Some(extend_recovered_level(
                    values,
                    source,
                    match_frames,
                    reset_at,
                    index,
                    baseline,
                ));
            }
        } else {
            return_run = 0;
        }
    }
    None
}

fn extend_recovered_level(
    values: &[f32],
    source: &[f32],
    match_frames: &[bool],
    reset_at: &[bool],
    confirmed_at: usize,
    baseline: f32,
) -> usize {
    let mut end = confirmed_at;
    let mut unobserved_run = 0;
    for index in confirmed_at + 1..values.len() {
        if reset_at[index] || !match_frames[index] {
            break;
        }
        let Some(value) = reliable_source(source, index) else {
            unobserved_run += 1;
            if unobserved_run > MAX_UNOBSERVED_EXTENSION {
                break;
            }
            end = index;
            continue;
        };
        unobserved_run = 0;
        if value < baseline - RETURN_TOLERANCE {
            break;
        }
        end = index;
    }
    end
}

fn reliable_source(source: &[f32], index: usize) -> Option<f32> {
    source.get(index).copied().filter(|value| *value >= 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 全ての観測が読めていて、全て試合画面だった前提の付属列。
    fn flags(length: usize) -> (Vec<bool>, Vec<bool>) {
        (vec![true; length], vec![false; length])
    }

    /// アーマーで削られ、数秒かけて元の高さまで戻る HP 列。
    fn armour_recovery() -> Vec<f32> {
        let mut values = vec![1.0f32; 10];
        for step in 0..35 {
            values.push(0.8 + 0.2 * step as f32 / 34.0);
        }
        values.extend(std::iter::repeat_n(1.0, 20));
        values
    }

    fn restore(values: &[f32]) -> Vec<f32> {
        let (match_frames, reset_at) = flags(values.len());
        let mut restored = values.to_vec();
        restore_confirmed_recoveries(&mut restored, values, &match_frames, &reset_at);
        restored
    }

    /// 削られてから数秒かけて元の高さへ戻る動きは、アーマーの回復。
    /// 削られていた区間ごと元の高さへ戻す。
    #[test]
    fn a_bar_that_climbs_back_to_where_it_was_is_restored() {
        let restored = restore(&armour_recovery());

        assert!(
            restored.iter().all(|value| (*value - 1.0).abs() < 1e-5),
            "回復を戻していない: {:?}",
            &restored[10..20]
        );
    }

    /// 戻ってこない下降は、ただの被弾。触らない。
    #[test]
    fn a_bar_that_never_climbs_back_is_left_alone() {
        let mut values = vec![1.0f32; 10];
        values.extend(std::iter::repeat_n(0.8, 60));

        let restored = restore(&values);

        assert_eq!(restored, values);
    }

    /// すぐ戻るのは読み違い。アーマーの回復には時間がかかる。
    #[test]
    fn an_instant_return_is_a_misread_not_a_recovery() {
        let mut values = vec![1.0f32; 10];
        values.extend(std::iter::repeat_n(0.8, 5));
        values.extend(std::iter::repeat_n(1.0, 60));

        let restored = restore(&values);

        assert_eq!(restored[12], 0.8, "一瞬の戻りを回復にしている");
    }

    /// 途中までしか戻らない動きは回復ではない。
    #[test]
    fn a_partial_return_is_not_a_recovery() {
        let mut values = vec![1.0f32; 10];
        for step in 0..35 {
            values.push(0.8 + 0.1 * step as f32 / 34.0);
        }
        values.extend(std::iter::repeat_n(0.9, 20));

        let restored = restore(&values);

        assert_eq!(restored[12], values[12], "半端な戻りを回復にしている");
    }

    /// 揺れずに一気に戻った値も回復ではない。回復は少しずつ上がる。
    #[test]
    fn a_single_leap_back_is_not_a_recovery() {
        let mut values = vec![1.0f32; 10];
        values.extend(std::iter::repeat_n(0.8, 40));
        values.extend(std::iter::repeat_n(1.0, 20));

        let restored = restore(&values);

        assert_eq!(restored[20], 0.8, "一段の跳ね上がりを回復にしている");
    }

    /// 読み取りの揺れ程度の下降は、回復を待つほどのことではない。
    #[test]
    fn a_dip_too_small_to_be_armour_damage_is_ignored() {
        let mut values = vec![1.0f32; 10];
        values.extend(std::iter::repeat_n(0.99, 40));
        values.extend(std::iter::repeat_n(1.0, 20));

        let restored = restore(&values);

        assert_eq!(restored[20], 0.99);
    }

    /// ラウンドが変わればそこで打ち切る。前のラウンドの削りを、次の
    /// ラウンドの全快で「回復した」と読まない。
    #[test]
    fn a_round_boundary_ends_the_recovery_window() {
        let values = armour_recovery();
        let (match_frames, mut reset_at) = flags(values.len());
        reset_at[30] = true;
        let mut restored = values.clone();

        restore_confirmed_recoveries(&mut restored, &values, &match_frames, &reset_at);

        assert_eq!(restored[12], values[12], "ラウンドをまたいで戻している");
    }

    /// 試合画面が途切れればそこで打ち切る。
    #[test]
    fn leaving_the_match_screen_ends_the_recovery_window() {
        let values = armour_recovery();
        let (mut match_frames, reset_at) = flags(values.len());
        match_frames[30] = false;
        let mut restored = values.clone();

        restore_confirmed_recoveries(&mut restored, &values, &match_frames, &reset_at);

        assert_eq!(restored[12], values[12], "画面外をまたいで戻している");
    }

    /// 読めなかったフレームは戻す対象にしない。読めていないものを
    /// 「回復した」とは言えない。
    #[test]
    fn unreadable_frames_are_not_given_a_value() {
        let mut values = armour_recovery();
        values[15] = -1.0;
        let (match_frames, reset_at) = flags(values.len());
        let source = values.clone();
        let mut restored = values.clone();

        restore_confirmed_recoveries(&mut restored, &source, &match_frames, &reset_at);

        assert_eq!(restored[15], -1.0, "読めないフレームに値を入れている");
        assert_eq!(restored[14], 1.0, "その前後は戻している");
    }

    /// 長さの噛み合わない列は触らない。呼び手の取り違えで、別の
    /// 試合の観測を混ぜない。
    #[test]
    fn mismatched_inputs_are_left_untouched() {
        let values = armour_recovery();
        let length = values.len();

        for (source_len, match_len, reset_len) in [
            (length - 1, length, length),
            (length, length - 1, length),
            (length, length, length - 1),
        ] {
            let mut restored = values.clone();
            restore_confirmed_recoveries(
                &mut restored,
                &values[..source_len],
                &vec![true; match_len],
                &vec![false; reset_len],
            );
            assert_eq!(restored, values, "長さが違うのに書き換えている");
        }
    }

    /// 空の列でも落ちない。
    #[test]
    fn an_empty_series_is_handled() {
        let mut values: Vec<f32> = Vec::new();

        restore_confirmed_recoveries(&mut values, &[], &[], &[]);

        assert!(values.is_empty());
    }
    // ── 確認に必要な形 ───────────────────────────────────────────────────

    /// 元の高さに戻った状態が続いて初めて回復と認める。1 フレーム
    /// 足りなければ認めない。
    #[test]
    fn the_return_must_hold_for_a_run_of_frames() {
        let restored_with = |hold: usize| {
            let mut values = vec![1.0f32; 10];
            for step in 0..35 {
                values.push(0.8 + 0.2 * step as f32 / 34.0);
            }
            values.extend(std::iter::repeat_n(1.0, hold));
            values.push(0.5);
            values.extend(std::iter::repeat_n(0.5, 20));
            restore(&values)[12]
        };

        // 上がりきってから 1.0 が続く長さで分かれる。
        assert!(
            (restored_with(RECOVERY_CONFIRM_RUN + 6) - 1.0).abs() < 1e-5,
            "十分に続いた戻りを認めていない"
        );
        assert!(restored_with(1) < 0.9, "続かない戻りを回復にしている");
    }

    /// 削られてから認めるまでには時間を置く。すぐ元へ戻る動きは、
    /// 遮蔽が晴れただけかもしれない。
    #[test]
    fn a_return_confirmed_too_soon_after_the_drop_is_not_a_recovery() {
        let restored_after = |hold: usize| {
            let mut values = vec![1.0f32; 10];
            for step in 0..10 {
                values.push(0.8 + 0.2 * step as f32 / 9.0);
            }
            values.extend(std::iter::repeat_n(1.0, hold));
            restore(&values)[12]
        };

        assert!(
            restored_after(10) < 0.9,
            "削られた直後の戻りを回復にしている"
        );
        assert!(
            (restored_after(40) - 1.0).abs() < 1e-5,
            "十分に間を置いた戻りを認めていない"
        );
    }

    /// 上がる段が少なすぎる動きは回復ではない。回復は何度も上がる。
    #[test]
    fn a_return_without_enough_upward_steps_is_not_a_recovery() {
        let with_steps = |steps: usize| {
            let mut values = vec![1.0f32; 10];
            values.extend(std::iter::repeat_n(0.8, 35));
            // 指定した回数だけ段を踏んで戻る。
            for step in 1..=steps {
                values.push(0.8 + 0.2 * step as f32 / steps as f32);
            }
            values.extend(std::iter::repeat_n(1.0, 20));
            restore(&values)[12]
        };

        assert!(
            (with_steps(MIN_UPWARD_STEPS + 2) - 1.0).abs() < 1e-5,
            "段を踏んだ戻りを認めていない"
        );
        assert!(
            with_steps(MIN_UPWARD_STEPS - 1) < 0.9,
            "段の足りない戻りを回復にしている"
        );
    }

    /// 回復を待つ窓には限りがある。何十秒も後の全快は、次のラウンドの
    /// バーであって回復ではない。
    #[test]
    fn a_return_far_beyond_the_window_is_not_a_recovery() {
        let mut values = vec![1.0f32; 10];
        values.extend(std::iter::repeat_n(0.8, MAX_RECOVERY_WINDOW + 10));
        for step in 0..35 {
            values.push(0.8 + 0.2 * step as f32 / 34.0);
        }
        values.extend(std::iter::repeat_n(1.0, 20));

        assert_eq!(restore(&values)[12], 0.8, "窓の外の戻りを拾っている");
    }

    // ── 戻した高さをどこまで伸ばすか ─────────────────────────────────────

    /// 回復を認めた後は、その高さが続く限り戻し続ける。
    #[test]
    fn the_restored_level_runs_on_while_the_bar_holds() {
        let mut values = armour_recovery();
        values.extend(std::iter::repeat_n(1.0, 30));
        let tail = values.len() - 1;

        let restored = restore(&values);

        assert!((restored[tail] - 1.0).abs() < 1e-5, "末尾まで戻していない");
    }

    /// 高さが落ちればそこで止める。次の被弾まで戻さない。
    #[test]
    fn the_restored_level_stops_where_the_bar_falls() {
        let mut values = armour_recovery();
        let fall = values.len();
        values.extend(std::iter::repeat_n(0.6, 20));

        let restored = restore(&values);

        assert!((restored[fall - 1] - 1.0).abs() < 1e-5);
        assert_eq!(restored[fall], 0.6, "落ちた後まで戻している");
    }

    /// 短い欠測は跨いで戻す。演出で数フレーム隠れるのは普通のこと。
    #[test]
    fn a_short_blind_stretch_is_carried_over() {
        let mut values = armour_recovery();
        let gap = values.len();
        values.extend(std::iter::repeat_n(-1.0, MAX_UNOBSERVED_EXTENSION));
        values.extend(std::iter::repeat_n(1.0, 10));
        let tail = values.len() - 1;

        let restored = restore(&values);

        assert!((restored[tail] - 1.0).abs() < 1e-5, "短い欠測で止めている");
        assert_eq!(restored[gap], -1.0, "欠測に値を入れている");
    }

    /// 長い欠測の先は戻さない。見えていない間に何が起きたか分からない。
    #[test]
    fn a_long_blind_stretch_ends_the_restored_level() {
        let mut values = armour_recovery();
        values.extend(std::iter::repeat_n(-1.0, MAX_UNOBSERVED_EXTENSION + 1));
        let after = values.len();
        values.extend(std::iter::repeat_n(0.4, 10));

        let restored = restore(&values);

        assert_eq!(restored[after], 0.4, "長い欠測を跨いで戻している");
    }
}
