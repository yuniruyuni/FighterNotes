use super::{FULL_HP, FULL_MIN_RUN};
use crate::frame_features::FrameFeatures;

/// A stage-colored overlay can hide the internal white cap while leaving the
/// rest of a visually full HP bar intact. The observed false edge is around
/// 91%, so this threshold is only used together with round-transition and
/// full-drive evidence; it is never a general full-HP threshold.
const STRUCTURAL_FULL_HP: f32 = 0.90;
const FULL_DRIVE: f32 = 0.95;
const ROUND_GAP_MIN: usize = 20;
const ROUND_GAP_LOOKBACK: usize = 180;
const ROUND_RECOVERY_MIN: f32 = 0.08;
const OPENING_EDGE_JITTER: f32 = 0.015;

pub(super) fn round_reset_frames(
    own: &[f32],
    opponent: &[f32],
    match_frames: &[bool],
) -> Vec<bool> {
    let is_full =
        |index: usize| match_frames[index] && own[index] >= FULL_HP && opponent[index] >= FULL_HP;
    let mut reset_at = vec![false; own.len()];
    let mut run_start = None;

    for index in 0..own.len() {
        if is_full(index) {
            run_start.get_or_insert(index);
        } else if let Some(start) = run_start.take() {
            mark_reset(&mut reset_at, start, index);
        }
    }
    if let Some(start) = run_start {
        mark_reset(&mut reset_at, start, own.len());
    }

    reset_at
}

/// Promote a conservatively confirmed round-opening run to full HP.
///
/// A relaxed HP value is insufficient on its own: the run must last as long
/// as a normal full-health run, both drive gauges must be visibly full and
/// reliable, a sustained non-match transition must occur shortly beforehand,
/// and HP must have recovered materially from the previous match section.
/// These conditions keep ordinary 90% HP neutral situations inside a round
/// from becoming resets.
pub(super) fn normalize_structural_full_runs(
    features: &[FrameFeatures],
    own: &mut [f32],
    opponent: &mut [f32],
    match_frames: &[bool],
) {
    let mut runs = Vec::new();
    let mut run_start = None;
    for (index, _) in own.iter().enumerate() {
        if is_structural_full(features, own, opponent, match_frames, index) {
            run_start.get_or_insert(index);
        } else if let Some(start) = run_start.take() {
            runs.push((start, index));
        }
    }
    if let Some(start) = run_start {
        runs.push((start, own.len()));
    }

    for (start, end) in runs {
        if end - start >= FULL_MIN_RUN {
            if let Some((gap_start, _gap_end)) = recent_non_match_gap(match_frames, start) {
                if has_round_recovery(own, opponent, match_frames, gap_start, start) {
                    let mut own_baseline = own[start];
                    let mut opponent_baseline = opponent[start];
                    for (&own_value, &opponent_value) in
                        own.iter().zip(opponent.iter()).take(end).skip(start)
                    {
                        own_baseline = own_baseline.min(own_value);
                        opponent_baseline = opponent_baseline.min(opponent_value);
                    }
                    promote_opening_side(own, match_frames, start, own_baseline);
                    promote_opening_side(opponent, match_frames, start, opponent_baseline);
                }
            }
        }
    }
}

fn promote_opening_side(values: &mut [f32], match_frames: &[bool], start: usize, baseline: f32) {
    let mut promoting = true;
    for (value, &is_match) in values.iter_mut().zip(match_frames).skip(start) {
        if promoting && is_match && *value >= baseline - OPENING_EDGE_JITTER {
            *value = 1.0;
        } else {
            promoting = false;
        }
    }
}

fn is_structural_full(
    features: &[FrameFeatures],
    own: &[f32],
    opponent: &[f32],
    match_frames: &[bool],
    index: usize,
) -> bool {
    let Some(feature) = features.get(index) else {
        return false;
    };
    match_frames.get(index).copied().unwrap_or(false)
        && own
            .get(index)
            .is_some_and(|value| *value >= STRUCTURAL_FULL_HP)
        && opponent
            .get(index)
            .is_some_and(|value| *value >= STRUCTURAL_FULL_HP)
        && feature.left_drive_ratio >= FULL_DRIVE
        && feature.right_drive_ratio >= FULL_DRIVE
        && !feature.left_burnout
        && !feature.right_burnout
        && !feature.left_drive_uncertain
        && !feature.right_drive_uncertain
}

fn recent_non_match_gap(match_frames: &[bool], start: usize) -> Option<(usize, usize)> {
    let search_start = start.saturating_sub(ROUND_GAP_LOOKBACK);
    let mut latest = None;
    let mut gap_start = None;
    for (index, &is_match) in match_frames
        .iter()
        .enumerate()
        .take(start)
        .skip(search_start)
    {
        if !is_match {
            gap_start.get_or_insert(index);
        } else if let Some(gap_start) = gap_start.take() {
            if index - gap_start >= ROUND_GAP_MIN {
                latest = Some((gap_start, index));
            }
        }
    }
    if let Some(gap_start) = gap_start {
        if start - gap_start >= ROUND_GAP_MIN {
            latest = Some((gap_start, start));
        }
    }
    latest
}

fn has_round_recovery(
    own: &[f32],
    opponent: &[f32],
    match_frames: &[bool],
    gap_start: usize,
    candidate_start: usize,
) -> bool {
    let candidate_min = own[candidate_start].min(opponent[candidate_start]);
    let prior_min = own
        .iter()
        .zip(opponent.iter())
        .zip(match_frames.iter())
        .take(gap_start)
        .rev()
        .filter_map(|((&own, &opponent), &is_match)| {
            let value = own.min(opponent);
            (is_match && value >= 0.0).then_some(value)
        })
        .take(ROUND_GAP_LOOKBACK)
        .reduce(f32::min);

    prior_min.is_none_or(|previous| candidate_min >= previous + ROUND_RECOVERY_MIN)
}

fn mark_reset(reset_at: &mut [bool], start: usize, end: usize) {
    if end - start >= FULL_MIN_RUN {
        reset_at[start] = true;
    }
}

pub(super) fn enforce_monotonic(values: &mut [f32], reset_at: &[bool]) {
    let mut previous = None;
    for (index, value) in values.iter_mut().enumerate() {
        if reset_at[index] {
            previous = None;
        }
        if *value < 0.0 {
            continue;
        }
        if let Some(previous) = previous {
            *value = (*value).min(previous);
        }
        previous = Some(*value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 両者が全快で映っているフレームの列。
    fn full_run(length: usize, full: std::ops::Range<usize>) -> (Vec<f32>, Vec<f32>, Vec<bool>) {
        let mut own = vec![0.5f32; length];
        let mut opponent = vec![0.5f32; length];
        for index in full {
            own[index] = 1.0;
            opponent[index] = 1.0;
        }
        (own, opponent, vec![true; length])
    }

    /// 両者が全快で並ぶ区間の頭が、ラウンドの切れ目。
    #[test]
    fn a_run_of_full_health_marks_a_round_boundary() {
        let (own, opponent, match_frames) = full_run(100, 20..60);

        let reset_at = round_reset_frames(&own, &opponent, &match_frames);

        assert!(reset_at[20], "全快の並びの頭を切れ目にしていない");
        assert_eq!(reset_at.iter().filter(|marked| **marked).count(), 1);
    }

    /// 全快が一瞬しか続かなければ切れ目ではない。
    #[test]
    fn a_flash_of_full_health_is_not_a_boundary() {
        let (own, opponent, match_frames) = full_run(100, 20..20 + FULL_MIN_RUN - 1);

        assert!(round_reset_frames(&own, &opponent, &match_frames)
            .iter()
            .all(|marked| !marked));
    }

    /// ちょうどの長さは切れ目に数える。
    #[test]
    fn a_run_of_exactly_the_minimum_is_a_boundary() {
        let (own, opponent, match_frames) = full_run(100, 20..20 + FULL_MIN_RUN);

        assert!(round_reset_frames(&own, &opponent, &match_frames)[20]);
    }

    /// 片方だけ全快でも切れ目ではない。ラウンドの頭は両者が全快。
    #[test]
    fn one_side_at_full_health_is_not_a_boundary() {
        let (own, mut opponent, match_frames) = full_run(100, 20..60);
        for value in &mut opponent[20..60] {
            *value = 0.5;
        }

        assert!(round_reset_frames(&own, &opponent, &match_frames)
            .iter()
            .all(|marked| !marked));
    }

    /// 試合画面の外の全快は数えない。リザルト画面のバーはラウンドの
    /// 頭ではない。
    #[test]
    fn full_health_outside_the_match_screen_is_not_a_boundary() {
        let (own, opponent, mut match_frames) = full_run(100, 20..60);
        for flag in &mut match_frames[20..60] {
            *flag = false;
        }

        assert!(round_reset_frames(&own, &opponent, &match_frames)
            .iter()
            .all(|marked| !marked));
    }

    /// 列の末尾まで全快が続いた場合も切れ目に数える。
    #[test]
    fn a_run_reaching_the_end_of_the_video_is_still_a_boundary() {
        let (own, opponent, match_frames) = full_run(100, 60..100);

        assert!(round_reset_frames(&own, &opponent, &match_frames)[60]);
    }

    // ── ラウンド内の単調化 ───────────────────────────────────────────────

    /// ラウンドの中で HP は増えない。
    #[test]
    fn health_never_climbs_inside_a_round() {
        let mut values = vec![1.0, 0.8, 0.9, 0.7, 0.75];
        let reset_at = vec![false; values.len()];

        enforce_monotonic(&mut values, &reset_at);

        assert_eq!(values, vec![1.0, 0.8, 0.8, 0.7, 0.7]);
    }

    /// ラウンドが変われば、そこから測り直す。
    #[test]
    fn a_round_boundary_starts_the_measurement_over() {
        let mut values = vec![1.0, 0.5, 1.0, 0.9];
        let mut reset_at = vec![false; values.len()];
        reset_at[2] = true;

        enforce_monotonic(&mut values, &reset_at);

        assert_eq!(values, vec![1.0, 0.5, 1.0, 0.9]);
    }

    /// 読めなかったフレームは基準にも結果にもしない。
    #[test]
    fn unreadable_frames_are_skipped_without_becoming_the_baseline() {
        let mut values = vec![1.0, 0.8, -1.0, 0.9];
        let reset_at = vec![false; values.len()];

        enforce_monotonic(&mut values, &reset_at);

        assert_eq!(values, vec![1.0, 0.8, -1.0, 0.8]);
    }
    // ── 覆われた全快バーの復元 ───────────────────────────────────────────

    use crate::temporal::tests::support::feature;

    /// ラウンド開始の全快バーが背景に覆われ、9 割ほどに見えている映像。
    struct Opening {
        features: Vec<FrameFeatures>,
        own: Vec<f32>,
        opponent: Vec<f32>,
        match_frames: Vec<bool>,
    }

    impl Opening {
        /// 前のラウンド（50f）→ 画面切り替え（30f）→ 新しいラウンド（40f）。
        fn new() -> Self {
            let mut features = Vec::new();
            let mut own = Vec::new();
            let mut opponent = Vec::new();
            let mut match_frames = Vec::new();
            for index in 0..120u32 {
                let mut frame = feature(index, 1.0);
                frame.left_drive_ratio = 1.0;
                frame.right_drive_ratio = 1.0;
                let (in_match, hp) = match index {
                    0..=49 => (true, 0.5),
                    50..=79 => (false, 0.5),
                    _ => (true, 0.92),
                };
                frame.is_match_screen = in_match;
                features.push(frame);
                own.push(hp);
                opponent.push(hp);
                match_frames.push(in_match);
            }
            Self {
                features,
                own,
                opponent,
                match_frames,
            }
        }

        fn normalize(mut self) -> (Vec<f32>, Vec<f32>) {
            normalize_structural_full_runs(
                &self.features,
                &mut self.own,
                &mut self.opponent,
                &self.match_frames,
            );
            (self.own, self.opponent)
        }
    }

    #[test]
    fn structural_full_checks_every_signal_and_exact_threshold() {
        let mut frame = feature(0, 1.0);
        frame.is_match_screen = true;
        frame.left_drive_ratio = FULL_DRIVE;
        frame.right_drive_ratio = FULL_DRIVE;
        let matches = [true];
        let hp = [STRUCTURAL_FULL_HP];
        let check = |frame: &FrameFeatures| {
            is_structural_full(std::slice::from_ref(frame), &hp, &hp, &matches, 0)
        };

        assert!(check(&frame));

        for changed in 0..6 {
            let mut invalid = frame.clone();
            match changed {
                0 => invalid.left_drive_ratio = FULL_DRIVE - 0.001,
                1 => invalid.right_drive_ratio = FULL_DRIVE - 0.001,
                2 => invalid.left_burnout = true,
                3 => invalid.right_burnout = true,
                4 => invalid.left_drive_uncertain = true,
                5 => invalid.right_drive_uncertain = true,
                _ => unreachable!(),
            }
            assert!(!check(&invalid), "signal {changed} was ignored");
        }

        assert!(!is_structural_full(
            &[frame.clone()],
            &[0.899],
            &hp,
            &matches,
            0
        ));
        assert!(!is_structural_full(&[frame], &hp, &[0.899], &matches, 0));
        assert!(!is_structural_full(&[], &[], &[], &[], 0));
        assert!(!is_structural_full(&[feature(0, 1.0)], &hp, &hp, &[], 0));

        let mut invalid_first = feature(0, 1.0);
        invalid_first.is_match_screen = true;
        invalid_first.left_drive_ratio = 0.0;
        let mut valid_second = feature(1, 1.0);
        valid_second.is_match_screen = true;
        valid_second.left_drive_ratio = FULL_DRIVE;
        valid_second.right_drive_ratio = FULL_DRIVE;
        assert!(!is_structural_full(
            &[invalid_first, valid_second],
            &[STRUCTURAL_FULL_HP; 2],
            &[STRUCTURAL_FULL_HP; 2],
            &[true; 2],
            0
        ));
        assert!(is_structural_full(
            &[feature(0, 1.0), {
                let mut frame = feature(1, 1.0);
                frame.is_match_screen = true;
                frame.left_drive_ratio = FULL_DRIVE;
                frame.right_drive_ratio = FULL_DRIVE;
                frame
            }],
            &[STRUCTURAL_FULL_HP; 2],
            &[STRUCTURAL_FULL_HP; 2],
            &[true; 2],
            1
        ));
    }

    #[test]
    fn opening_extension_includes_its_edge_and_stops_after_it() {
        let baseline = 0.92;
        let edge = baseline - OPENING_EDGE_JITTER;
        let mut values = [baseline, edge, edge - 0.001, baseline];

        promote_opening_side(&mut values, &[true; 4], 0, baseline);

        assert_eq!(values[0], 1.0);
        assert_eq!(values[1], 1.0);
        assert_eq!(values[2], edge - 0.001);
        assert_eq!(values[3], baseline, "途切れた後から昇格を再開している");
    }

    #[test]
    fn an_exact_structural_run_uses_each_sides_lowest_opening_sample() {
        let mut opening = Opening::new();
        opening.own[90] = STRUCTURAL_FULL_HP;
        opening.opponent[91] = STRUCTURAL_FULL_HP;
        opening.own[100] = 0.904;
        opening.opponent[100] = 0.904;
        opening.features[100].left_drive_ratio = 0.0;
        opening.own[101] = 0.7;
        opening.opponent[101] = 0.7;

        let (own, opponent) = opening.normalize();

        assert_eq!(own[100], 1.0);
        assert_eq!(opponent[100], 1.0);
        assert_eq!(own[101], 0.7);
        assert_eq!(opponent[101], 0.7);
    }

    #[test]
    fn transition_search_has_exact_length_and_lookback_edges() {
        let mut exact = vec![true; 300];
        exact[80..80 + ROUND_GAP_MIN].fill(false);
        assert_eq!(
            recent_non_match_gap(&exact, 200),
            Some((80, 80 + ROUND_GAP_MIN))
        );

        let mut short = vec![true; 300];
        short[80..80 + ROUND_GAP_MIN - 1].fill(false);
        assert_eq!(recent_non_match_gap(&short, 200), None);

        let mut too_old = vec![true; 300];
        too_old[0..ROUND_GAP_MIN].fill(false);
        assert_eq!(recent_non_match_gap(&too_old, 250), None);

        let start = 100;
        let mut candidate_frame_is_not_part_of_the_gap = vec![true; 150];
        candidate_frame_is_not_part_of_the_gap[start - ROUND_GAP_MIN + 1..=start].fill(false);
        assert_eq!(
            recent_non_match_gap(&candidate_frame_is_not_part_of_the_gap, start),
            None
        );

        let mut ending_at_candidate = vec![true; 150];
        ending_at_candidate[start - ROUND_GAP_MIN..start].fill(false);
        assert_eq!(
            recent_non_match_gap(&ending_at_candidate, start),
            Some((start - ROUND_GAP_MIN, start))
        );
    }

    #[test]
    fn round_recovery_includes_the_exact_material_gain() {
        let own = [0.50, 0.50, 0.50, 0.50 + ROUND_RECOVERY_MIN];
        let opponent = own;
        let matches = [true, true, true, true];
        assert!(has_round_recovery(&own, &opponent, &matches, 3, 3));

        let below = [0.50, 0.50, 0.50, 0.50 + ROUND_RECOVERY_MIN - 0.001];
        assert!(!has_round_recovery(&below, &below, &matches, 3, 3));
        assert!(has_round_recovery(&own, &opponent, &[false; 4], 3, 3));
    }

    /// 画面の切り替わりの後に続く、両者ほぼ満タン・ドライブも満タンの
    /// 区間は、ラウンド開始の全快。覆われて欠けた分を戻す。
    #[test]
    fn a_covered_opening_bar_is_restored_to_full() {
        let (own, opponent) = Opening::new().normalize();

        assert_eq!(own[100], 1.0, "自分側を戻していない");
        assert_eq!(opponent[100], 1.0, "相手側を戻していない");
        assert_eq!(own[10], 0.5, "前のラウンドまで戻している");
    }

    /// 短い区間は戻さない。ラウンドの途中に 9 割の場面はいくらでもある。
    #[test]
    fn a_short_stretch_of_near_full_health_is_not_an_opening() {
        let mut opening = Opening::new();
        // 新しいラウンドの区間を最小の長さより 1 つ短くする。
        for index in 80 + FULL_MIN_RUN - 1..120 {
            opening.own[index] = 0.5;
            opening.opponent[index] = 0.5;
        }

        let (own, _) = opening.normalize();

        assert_eq!(own[100], 0.5);
        assert_eq!(own[85], 0.92, "短い区間まで戻している");
    }

    /// 直前に画面の切り替わりが無ければ、ラウンドの頭ではない。
    #[test]
    fn without_a_screen_transition_it_is_not_an_opening() {
        let mut opening = Opening::new();
        for index in 50..80 {
            opening.features[index].is_match_screen = true;
            opening.match_frames[index] = true;
        }

        let (own, _) = opening.normalize();

        assert_eq!(own[100], 0.92, "切り替わり無しで戻している");
    }

    /// 切り替わりが一瞬なら、ラウンドの区切りではない。
    #[test]
    fn a_brief_flicker_is_not_a_round_transition() {
        let mut opening = Opening::new();
        for index in 50..80 - ROUND_GAP_MIN + 1 {
            opening.features[index].is_match_screen = true;
            opening.match_frames[index] = true;
        }

        let (own, _) = opening.normalize();

        assert_eq!(own[100], 0.92, "一瞬の途切れで戻している");
    }

    /// 切り替わりの前と HP がほとんど変わらないなら、ラウンドは
    /// 変わっていない。
    #[test]
    fn without_a_real_recovery_it_is_not_a_new_round() {
        let mut opening = Opening::new();
        for index in 0..50 {
            opening.own[index] = 0.9;
            opening.opponent[index] = 0.9;
        }

        let (own, _) = opening.normalize();

        assert_eq!(own[100], 0.92, "回復していないのに戻している");
    }

    /// ドライブゲージが満タンでなければ、ラウンドの頭ではない。
    #[test]
    fn a_drive_gauge_short_of_full_rules_out_an_opening() {
        let mut opening = Opening::new();
        for index in 80..120 {
            opening.features[index].right_drive_ratio = 0.9;
        }

        let (own, _) = opening.normalize();

        assert_eq!(own[100], 0.92);
    }

    /// バーンアウト中はラウンドの頭ではない。
    #[test]
    fn a_burnt_out_side_rules_out_an_opening() {
        let mut opening = Opening::new();
        for index in 80..120 {
            opening.features[index].left_burnout = true;
        }

        let (own, _) = opening.normalize();

        assert_eq!(own[100], 0.92);
    }

    /// ドライブの読みが怪しいフレームは根拠にしない。
    #[test]
    fn an_unreliable_drive_reading_rules_out_an_opening() {
        let mut opening = Opening::new();
        for index in 80..120 {
            opening.features[index].right_drive_uncertain = true;
        }

        let (own, _) = opening.normalize();

        assert_eq!(own[100], 0.92);
    }

    /// 片方だけ大きく削れていれば、ラウンドの頭ではない。
    #[test]
    fn one_side_far_from_full_rules_out_an_opening() {
        let mut opening = Opening::new();
        for index in 80..120 {
            opening.opponent[index] = 0.6;
        }

        let (own, _) = opening.normalize();

        assert_eq!(own[100], 0.92);
    }
}
