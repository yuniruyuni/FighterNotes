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

#[derive(Clone, Copy)]
enum Baseline {
    Missing,
    Value(f32),
}

pub(super) fn restore_confirmed_recoveries(
    values: &mut [f32],
    source: &[f32],
    match_frames: &[bool],
    reset_at: &[bool],
) {
    if source.len() != values.len() {
        return;
    }
    if match_frames.len() != values.len() {
        return;
    }
    if reset_at.len() != values.len() {
        return;
    }

    let mut previous = Baseline::Missing;
    for index in 0..values.len() {
        if reset_at[index] {
            previous = Baseline::Missing;
        }
        let value = values[index];
        if is_reliable(value, match_frames[index]) {
            match previous {
                Baseline::Missing => previous = Baseline::Value(value),
                Baseline::Value(baseline) if !is_material_drop(baseline, value) => {
                    previous = Baseline::Value(baseline.min(value));
                }
                Baseline::Value(baseline) => {
                    if let Some(end) = confirmed_recovery_end(
                        values,
                        source,
                        match_frames,
                        reset_at,
                        index,
                        baseline,
                    ) {
                        for value in values[index..=end]
                            .iter_mut()
                            .filter(|value| is_nonnegative(**value))
                        {
                            *value = baseline;
                        }
                        previous = Baseline::Value(baseline);
                    } else {
                        previous = Baseline::Value(value);
                    }
                }
            }
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
    let mut low = if is_nonnegative(source[drop_start]) {
        source[drop_start]
    } else {
        values[drop_start]
    };
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

        if recovery_sample_qualifies(index - drop_start, baseline, value, low, upward_steps) {
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
    source
        .get(index)
        .copied()
        .filter(|value| is_nonnegative(*value))
}

fn recovery_sample_qualifies(
    age: usize,
    baseline: f32,
    value: f32,
    low: f32,
    upward_steps: usize,
) -> bool {
    let drop = baseline - low;
    let returned = value - low;
    age >= MIN_RECOVERY_DELAY
        && baseline - value <= RETURN_TOLERANCE
        && drop >= MIN_DROP
        && returned >= drop * MIN_RETURN_RATIO
        && upward_steps >= MIN_UPWARD_STEPS
}

fn is_nonnegative(value: f32) -> bool {
    value >= 0.0
}

fn is_reliable(value: f32, is_match: bool) -> bool {
    is_match && is_nonnegative(value)
}

fn is_material_drop(baseline: f32, value: f32) -> bool {
    baseline - value >= MIN_DROP
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

    #[test]
    fn reliability_and_drop_predicates_include_their_exact_edges() {
        assert!(is_nonnegative(0.0));
        assert!(!is_nonnegative(-0.001));
        assert!(is_reliable(0.0, true));
        assert!(!is_reliable(-0.001, true));
        assert!(!is_reliable(0.5, false));

        assert!(is_material_drop(MIN_DROP, 0.0));
        assert!(!is_material_drop(MIN_DROP / 2.0, 0.0));
        assert!(!is_material_drop(0.5, 0.49));

        assert_eq!(reliable_source(&[0.0, -1.0], 0), Some(0.0));
        assert_eq!(reliable_source(&[0.0, -1.0], 1), None);
        assert_eq!(reliable_source(&[0.0, -1.0], 2), None);
    }

    #[test]
    fn every_recovery_evidence_threshold_is_required_at_its_edge() {
        let qualifies = |age, baseline, value, low, steps| {
            recovery_sample_qualifies(age, baseline, value, low, steps)
        };

        assert!(qualifies(
            MIN_RECOVERY_DELAY,
            MIN_DROP,
            MIN_DROP,
            0.0,
            MIN_UPWARD_STEPS,
        ));
        assert!(!qualifies(
            MIN_RECOVERY_DELAY - 1,
            MIN_DROP,
            MIN_DROP,
            0.0,
            MIN_UPWARD_STEPS,
        ));
        assert!(!qualifies(
            MIN_RECOVERY_DELAY,
            1.0,
            1.0 - RETURN_TOLERANCE - 0.001,
            0.8,
            MIN_UPWARD_STEPS,
        ));
        assert!(qualifies(
            MIN_RECOVERY_DELAY,
            1.0,
            1.0 - RETURN_TOLERANCE,
            0.8,
            MIN_UPWARD_STEPS,
        ));
        assert!(qualifies(
            MIN_RECOVERY_DELAY,
            RETURN_TOLERANCE,
            0.0,
            -0.04,
            MIN_UPWARD_STEPS,
        ));
        assert!(!qualifies(
            MIN_RECOVERY_DELAY,
            MIN_DROP - 0.001,
            MIN_DROP - 0.001,
            0.0,
            MIN_UPWARD_STEPS,
        ));

        let exact_return = MIN_DROP * MIN_RETURN_RATIO;
        assert!(qualifies(
            MIN_RECOVERY_DELAY,
            MIN_DROP,
            exact_return,
            0.0,
            MIN_UPWARD_STEPS,
        ));
        assert!(!qualifies(
            MIN_RECOVERY_DELAY,
            MIN_DROP,
            exact_return - 0.001,
            0.0,
            MIN_UPWARD_STEPS,
        ));
        assert!(!qualifies(
            MIN_RECOVERY_DELAY,
            MIN_DROP,
            MIN_DROP,
            0.0,
            MIN_UPWARD_STEPS - 1,
        ));
    }

    #[test]
    fn recovered_extension_stops_at_each_hard_boundary() {
        let values = vec![1.0; 16];
        let source = values.clone();
        let matches = vec![true; values.len()];
        let reset = vec![false; values.len()];
        assert_eq!(
            extend_recovered_level(&values, &source, &matches, &reset, 0, 1.0),
            values.len() - 1
        );

        let mut stopped_by_reset = reset.clone();
        stopped_by_reset[2] = true;
        assert_eq!(
            extend_recovered_level(&values, &source, &matches, &stopped_by_reset, 0, 1.0),
            1
        );

        let mut reset_on_confirmation = reset.clone();
        reset_on_confirmation[0] = true;
        assert_eq!(
            extend_recovered_level(&values, &source, &matches, &reset_on_confirmation, 0, 1.0,),
            values.len() - 1
        );

        let mut stopped_by_screen = matches.clone();
        stopped_by_screen[2] = false;
        assert_eq!(
            extend_recovered_level(&values, &source, &stopped_by_screen, &reset, 0, 1.0),
            1
        );

        let mut short_blind = source.clone();
        short_blind[1..=MAX_UNOBSERVED_EXTENSION].fill(-1.0);
        assert_eq!(
            extend_recovered_level(&values, &short_blind, &matches, &reset, 0, 1.0),
            values.len() - 1
        );

        let mut long_blind = source.clone();
        long_blind[1..=MAX_UNOBSERVED_EXTENSION + 1].fill(-1.0);
        assert_eq!(
            extend_recovered_level(&values, &long_blind, &matches, &reset, 0, 1.0),
            MAX_UNOBSERVED_EXTENSION
        );

        let mut later_fall = source;
        later_fall[2] = 0.5;
        assert_eq!(
            extend_recovered_level(&values, &later_fall, &matches, &reset, 0, 1.0),
            1
        );

        let mut exact_edge = values.clone();
        exact_edge[1] = 1.0 - RETURN_TOLERANCE;
        assert_eq!(
            extend_recovered_level(&values, &exact_edge, &matches, &reset, 0, 1.0),
            values.len() - 1
        );
    }

    #[test]
    fn recovery_scan_uses_the_raw_drop_and_includes_its_hard_end() {
        let mut values = vec![0.8; MAX_RECOVERY_WINDOW + 1];
        values[MAX_RECOVERY_WINDOW - 10] = 0.85;
        values[MAX_RECOVERY_WINDOW - 9] = 0.90;
        values[MAX_RECOVERY_WINDOW - 8] = 0.95;
        values[MAX_RECOVERY_WINDOW - 7..].fill(1.0);
        let matches = vec![true; values.len()];
        let reset = vec![false; values.len()];

        assert_eq!(
            confirmed_recovery_end(&values, &values, &matches, &reset, 0, 1.0),
            Some(MAX_RECOVERY_WINDOW)
        );

        let recovery = armour_recovery();
        let (recovery_matches, mut recovery_reset) = flags(recovery.len());
        recovery_reset[10] = true;
        assert!(confirmed_recovery_end(
            &recovery,
            &recovery,
            &recovery_matches,
            &recovery_reset,
            10,
            1.0,
        )
        .is_some());

        let mut corrected = vec![0.8; 48];
        let mut raw = vec![0.96; corrected.len()];
        raw[1] = 0.965;
        raw[2] = 0.975;
        raw[3..].fill(0.985);
        corrected[1..].copy_from_slice(&raw[1..]);
        assert_eq!(
            confirmed_recovery_end(
                &corrected,
                &raw,
                &vec![true; corrected.len()],
                &vec![false; corrected.len()],
                0,
                1.0,
            ),
            None
        );
    }

    #[test]
    fn a_small_baseline_dip_is_not_restored_to_the_older_higher_value() {
        let mut values = vec![1.0];
        values.extend(vec![0.99; 9]);
        for step in 0..35 {
            values.push(0.79 + 0.20 * step as f32 / 34.0);
        }
        values.extend(vec![0.99; 20]);

        let restored = restore(&values);

        assert!((restored[12] - 0.99).abs() < 1e-5);
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
