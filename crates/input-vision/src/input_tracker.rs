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
                    if (c as i64) < pc as i64 - 3 {
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
        let len = we - ws;
        if len < MIN_WINDOW {
            continue;
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
        let Some((&base, &support)) = offs.iter().max_by_key(|(_, &v)| v) else {
            continue;
        };
        let _ = len;
        if support < MIN_SUPPORT_COUNT || (support as f32) < (n_certain as f32) * MIN_SUPPORT_RATIO
        {
            continue;
        }

        // dir / バッジ類の最頻値（確か読みのみ）
        let mode_dir = mode_by(frames, ws, we, |r| {
            (!r.uncertain && r.dir != InputDir::Unknown).then_some(r.dir)
        });
        let mode_marks = mode_by(frames, ws, we, |r| {
            (!r.uncertain).then(|| (r.badges.clone(), r.auto, r.throw))
        });

        for (offset, t) in out[ws..we].iter_mut().enumerate() {
            let i = ws + offset;
            let expect = base + i as i64;
            if expect < 1 {
                continue;
            }
            let expect = expect as u32;
            let mismatch = t.count != Some(expect);
            if mismatch || t.uncertain {
                t.count = Some(expect);
                t.repaired = true;
            }
            t.uncertain = false;
            if let Some(d) = mode_dir {
                if t.dir != d {
                    t.dir = d;
                    t.repaired = true;
                }
            }
            if let Some((ref b, a, th)) = mode_marks {
                if t.badges != *b || t.auto != a || t.throw != th {
                    t.badges = b.clone();
                    t.auto = a;
                    t.throw = th;
                    t.repaired = true;
                }
            }
        }
    }

    out
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
            if let Some(e) = items.iter_mut().find(|(x, _)| *x == v) {
                e.1 += 1;
            } else {
                items.push((v, 1));
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
}
