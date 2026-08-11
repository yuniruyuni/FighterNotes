//! 入力履歴の確定層トラッカー。
//!
//! 知覚層（input_history）の row0（現在入力）時系列を count の +1 連続性で
//! 補修する。表示のカウントは 1 動画フレームごとに正確に +1 されるため、
//! 同一入力の区間では count - frame_index が一定になる。これを利用して:
//!   - エンコーダゴースト等による単発の読み欠け（unc）を補完
//!   - 孤立した誤読（連続性に反する値）を訂正
//!   - dir / バッジ類は区間内の確か読みの最頻値で安定化
//!
//! HP の forward fill・OD の clean_drive_temporal と同じ「知覚と確定の
//! 責務分離」の確定層にあたる。

use crate::input_history::{BadgeMark, InputDir, InputRow};

/// 補修済みの現在入力（row0）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrackedInput {
    /// 継続フレーム数（補修後）。区間当てはめ不能な場合は知覚値のまま
    pub count: Option<u32>,
    pub dir: InputDir,
    pub badges: Vec<BadgeMark>,
    pub auto: bool,
    pub throw: bool,
    /// 知覚層の読みを連続性から補完・訂正したフレーム
    pub repaired: bool,
    /// 補修後も値が確定できないフレーム
    pub uncertain: bool,
}

/// 区間当てはめに必要な最小フレーム数
const MIN_WINDOW: usize = 5;
/// base 当てはめに必要な確か読みの最小数
const MIN_SUPPORT_COUNT: u32 = 4;
/// base 当てはめに必要な「確か読みのうち base に一致する」割合。
/// 分母は窓全長ではなく確か読み数（unc が多い窓でも、読めたフレーム
/// 同士が一致していれば当てはめてよい）
const MIN_SUPPORT_RATIO: f32 = 0.7;

fn starts_new_window(previous: u32, current: u32) -> bool {
    i64::from(previous) - i64::from(current) > 3
}

fn window_is_long_enough(start: usize, end: usize) -> bool {
    end.saturating_sub(start) >= MIN_WINDOW
}

fn has_enough_support(support: u32, certain: u32) -> bool {
    support >= MIN_SUPPORT_COUNT && (support as f32) >= (certain as f32) * MIN_SUPPORT_RATIO
}

/// row0 の時系列（連続フレーム）を補修する。
pub fn repair_row0_sequence(frames: &[InputRow]) -> Vec<TrackedInput> {
    let n = frames.len();
    let mut out: Vec<TrackedInput> = frames
        .iter()
        .map(|r| TrackedInput {
            count: r.count,
            dir: r.dir,
            badges: r.badges.clone(),
            auto: r.auto,
            throw: r.throw,
            repaired: false,
            uncertain: r.uncertain || r.count.is_none(),
        })
        .collect();

    // ── 窓分割: 確かな count が大きく下がる位置 = 入力変化（シフト） ──────
    let mut windows: Vec<(usize, usize)> = Vec::new();
    {
        let mut start = 0usize;
        let mut prev_certain: Option<(usize, u32)> = None;
        for (i, frame) in frames.iter().enumerate() {
            if let (false, Some(c)) = (frame.uncertain, frame.count) {
                if let Some((_, pc)) = prev_certain {
                    if starts_new_window(pc, c) {
                        windows.push((start, i));
                        start = i;
                    }
                }
                prev_certain = Some((i, c));
            }
        }
        windows.push((start, n));
    }

    for &(ws, we) in &windows {
        repair_window(frames, &mut out, ws, we);
    }

    out
}

fn repair_window(frames: &[InputRow], out: &mut [TrackedInput], ws: usize, we: usize) {
    if !window_is_long_enough(ws, we) {
        return;
    }
    // base = mode(count_i - i) を確か読みから推定
    let mut offs: std::collections::HashMap<i64, u32> = std::collections::HashMap::new();
    let mut n_certain = 0u32;
    for (offset, frame) in frames[ws..we].iter().enumerate() {
        let i = ws + offset;
        if !frame.uncertain {
            if let Some(c) = frame.count {
                *offs.entry(c as i64 - i as i64).or_insert(0) += 1;
                n_certain += 1;
            }
        }
    }
    let Some((&base, &support)) = offs.iter().max_by_key(|(_, &value)| value) else {
        return;
    };
    if !has_enough_support(support, n_certain) {
        return;
    }

    // dir / バッジ類の最頻値（確か読みのみ）
    let mode_dir = mode_by(frames, ws, we, |row| {
        (!row.uncertain && row.dir != InputDir::Unknown).then_some(row.dir)
    });
    let mode_marks = mode_by(frames, ws, we, |row| {
        (!row.uncertain).then(|| (row.badges.clone(), row.auto, row.throw))
    });

    for (offset, tracked) in out[ws..we].iter_mut().enumerate() {
        let index = ws + offset;
        let expected = base + index as i64;
        if expected >= 1 {
            let expected = expected as u32;
            if tracked.count != Some(expected) || tracked.uncertain {
                tracked.count = Some(expected);
                tracked.repaired = true;
            }
            tracked.uncertain = false;
            if let Some(direction) = mode_dir {
                if tracked.dir != direction {
                    tracked.dir = direction;
                    tracked.repaired = true;
                }
            }
            if let Some((ref badges, auto, throw)) = mode_marks {
                if tracked.badges != *badges || tracked.auto != auto || tracked.throw != throw {
                    tracked.badges = badges.clone();
                    tracked.auto = auto;
                    tracked.throw = throw;
                    tracked.repaired = true;
                }
            }
        }
    }
}

/// 窓内の最頻値（確か読みのみ対象）。
fn mode_by<T: Clone + PartialEq>(
    frames: &[InputRow],
    ws: usize,
    we: usize,
    f: impl Fn(&InputRow) -> Option<T>,
) -> Option<T> {
    let mut items: Vec<(T, u32)> = Vec::new();
    for r in &frames[ws..we] {
        if let Some(v) = f(r) {
            match items.iter().position(|(item, _)| *item == v) {
                Some(index) => items[index].1 += 1,
                None => items.push((v, 1)),
            }
        }
    }
    items.into_iter().max_by_key(|(_, c)| *c).map(|(v, _)| v)
}

// ─────────────────────────────────────────────────────────────────────────────
// テスト
// ─────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_history::BadgeColor;

    fn row(count: Option<u32>, dir: InputDir, unc: bool) -> InputRow {
        InputRow {
            count,
            dir,
            badges: vec![],
            auto: false,
            throw: false,
            empty: false,
            uncertain: unc,
        }
    }

    #[test]
    fn test_repair_isolated_hole() {
        // 20,21,22,?,24,25,26 → 23 を補完（f1516 型）
        let mut fs: Vec<InputRow> = (20..=26)
            .map(|c| row(Some(c), InputDir::DownLeft, false))
            .collect();
        fs[3] = row(None, InputDir::DownLeft, true);
        let t = repair_row0_sequence(&fs);
        assert_eq!(t[3].count, Some(23));
        assert!(t[3].repaired);
        assert!(!t[3].uncertain);
        assert!(!t[0].repaired);
    }

    #[test]
    fn test_repair_isolated_misread() {
        // 5,6,9,8,9 → 7 に訂正（孤立誤読）
        let counts = [5u32, 6, 9, 8, 9];
        let fs: Vec<InputRow> = counts
            .iter()
            .map(|&c| row(Some(c), InputDir::Neutral, false))
            .collect();
        let t = repair_row0_sequence(&fs);
        assert_eq!(t[2].count, Some(7));
        assert!(t[2].repaired);
        assert_eq!(t[4].count, Some(9));
    }

    #[test]
    fn test_shift_splits_windows() {
        // 10,11,12 | 1,2,3,4,5（入力変化で窓分割、両窓とも当てはめ）
        // 前窓は 3 フレームで MIN_WINDOW 未満 → そのまま。後窓は補修対象
        let counts = [10u32, 11, 12, 1, 2, 3, 4, 5];
        let mut fs: Vec<InputRow> = counts
            .iter()
            .map(|&c| row(Some(c), InputDir::Left, false))
            .collect();
        fs[5] = row(None, InputDir::Unknown, true); // 後窓に穴
        let t = repair_row0_sequence(&fs);
        assert_eq!(t[5].count, Some(3));
        assert!(t[5].repaired);
        assert_eq!(t[0].count, Some(10));
    }

    #[test]
    fn test_unfit_window_passthrough() {
        // 支持率不足（バラバラの読み）→ 補修しない
        let counts = [3u32, 9, 15, 2, 30];
        let fs: Vec<InputRow> = counts
            .iter()
            .map(|&c| row(Some(c), InputDir::Neutral, false))
            .collect();
        let t = repair_row0_sequence(&fs);
        assert_eq!(t[1].count, Some(9));
        assert!(!t[1].repaired);
    }

    #[test]
    fn test_dir_and_marks_stabilized() {
        // 窓内の dir 単発誤読も最頻値で訂正
        let mut fs: Vec<InputRow> = (1..=8)
            .map(|c| row(Some(c), InputDir::Down, false))
            .collect();
        fs[4].dir = InputDir::DownLeft; // 単発の誤読
        let t = repair_row0_sequence(&fs);
        assert_eq!(t[4].dir, InputDir::Down);
        assert!(t[4].repaired);
    }

    #[test]
    fn window_and_support_boundaries_are_exact() {
        assert!(!starts_new_window(10, 7));
        assert!(starts_new_window(10, 6));
        assert!(!window_is_long_enough(10, 14));
        assert!(window_is_long_enough(10, 15));
        assert!(has_enough_support(4, 5));
        assert!(has_enough_support(7, 10));
        assert!(!has_enough_support(6, 10));
        assert!(!has_enough_support(3, 3));
    }

    #[test]
    fn an_expected_count_of_one_is_repaired() {
        let mut frames: Vec<_> = (1..=5)
            .map(|count| row(Some(count), InputDir::Down, false))
            .collect();
        frames[0] = row(None, InputDir::Unknown, true);

        let tracked = repair_row0_sequence(&frames);

        assert_eq!(tracked[0].count, Some(1));
        assert!(tracked[0].repaired);
        assert!(!tracked[0].uncertain);
    }

    #[test]
    fn negative_early_expectations_do_not_stop_later_repairs() {
        let mut frames = vec![row(None, InputDir::Unknown, true); 7];
        for (index, count) in (1..=4).enumerate() {
            frames[index + 2] = row(Some(count), InputDir::Down, false);
        }

        let tracked = repair_row0_sequence(&frames);

        assert_eq!(tracked[6].count, Some(5));
        assert!(tracked[6].repaired);
    }

    #[test]
    fn support_ratio_counts_every_certain_observation() {
        let mut frames: Vec<_> = [10, 11, 12, 13, 100, 200]
            .into_iter()
            .map(|count| row(Some(count), InputDir::Down, false))
            .collect();
        frames.push(row(None, InputDir::Unknown, true));

        let tracked = repair_row0_sequence(&frames);

        assert_eq!(tracked[6].count, None);
        assert!(tracked[6].uncertain);
        assert!(!tracked[6].repaired);
    }

    #[test]
    fn every_mark_field_is_stabilized_independently() {
        let common_badges = vec![BadgeMark {
            color: BadgeColor::Green,
            boxed: false,
            glyph: None,
        }];
        let mut frames: Vec<_> = (1..=8)
            .map(|count| {
                let mut input = row(Some(count), InputDir::Down, false);
                input.badges = common_badges.clone();
                input.auto = true;
                input.throw = true;
                input
            })
            .collect();
        frames[0].badges.clear();
        frames[1].auto = false;
        frames[2].throw = false;

        let tracked = repair_row0_sequence(&frames);

        for input in tracked.iter().take(3) {
            assert_eq!(input.badges, common_badges);
            assert!(input.auto);
            assert!(input.throw);
            assert!(input.repaired);
        }
    }

    #[test]
    fn mode_uses_only_the_requested_window_and_counts_duplicates() {
        let directions = [
            InputDir::Right,
            InputDir::Right,
            InputDir::Right,
            InputDir::Right,
            InputDir::Left,
            InputDir::Left,
            InputDir::Right,
        ];
        let frames: Vec<_> = directions
            .into_iter()
            .map(|direction| row(Some(1), direction, false))
            .collect();

        assert_eq!(
            mode_by(&frames, 4, 7, |input| Some(input.dir)),
            Some(InputDir::Left)
        );
    }
}
