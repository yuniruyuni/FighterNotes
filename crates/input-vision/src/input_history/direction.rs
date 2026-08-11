use super::*;

// ── 方向読み取り ─────────────────────────────────────────────────────────────

pub(super) fn dir_mask(f: &Frame, x0: usize, y0: usize) -> ([u64; DIR_H], u32) {
    let mut m = [0u64; DIR_H];
    let mut n = 0u32;
    for (ry, row) in m.iter_mut().enumerate() {
        for rx in 0..DIR_W {
            if f.is_white(x0 + rx, y0 + ry) {
                *row += 1 << rx;
                n += 1;
            }
        }
    }
    (m, n)
}

pub(super) fn mask_centroid(m: &[u64; DIR_H]) -> Option<(f32, f32)> {
    let (mut sx, mut sy, mut n) = (0f32, 0f32, 0u32);
    for (y, row) in m.iter().enumerate() {
        let mut bits = *row;
        for _ in 0..bits.count_ones() {
            let x = bits.trailing_zeros();
            sx += x as f32;
            sy += y as f32;
            n += 1;
            bits &= bits - 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some((sx / n as f32, sy / n as f32))
    }
}

/// マスクを (dx, dy) だけシフト（範囲外は捨てる）
pub(super) fn shift_mask(m: &[u64; DIR_H], dx: i32, dy: i32) -> [u64; DIR_H] {
    let mut out = [0u64; DIR_H];
    let col_mask: u64 = (1u64 << DIR_W) - 1;
    for y in 0..DIR_H as i32 {
        let sy = y - dy;
        if sy < 0 || sy >= DIR_H as i32 {
            continue;
        }
        let row = m[sy as usize];
        out[y as usize] = shift_row(row, dx, col_mask);
    }
    out
}

fn shift_row(row: u64, dx: i32, col_mask: u64) -> u64 {
    match dx.cmp(&0) {
        std::cmp::Ordering::Less => row >> dx.unsigned_abs(),
        std::cmp::Ordering::Equal => row & col_mask,
        std::cmp::Ordering::Greater => (row << dx.unsigned_abs()) & col_mask,
    }
}

pub(super) fn alignment_offset(sample: (f32, f32), template: (f32, f32)) -> (i32, i32) {
    (
        (sample.0 - template.0).round() as i32,
        (sample.1 - template.1).round() as i32,
    )
}

pub(super) fn fine_offsets(center: i32) -> [i32; 3] {
    [center - 1, center, center + 1]
}

pub(super) fn within_alignment_window(dx: i32, dy: i32) -> bool {
    dx.unsigned_abs() <= 6 && dy.unsigned_abs() <= 6
}

pub(super) fn rank_direction_candidate(
    best: (InputDir, u32),
    second: u32,
    candidate: (InputDir, u32),
) -> ((InputDir, u32), u32) {
    match candidate.1.cmp(&best.1) {
        std::cmp::Ordering::Less => (candidate, best.1),
        std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => match candidate.1.cmp(&second) {
            std::cmp::Ordering::Less => (best, candidate.1),
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => (best, second),
        },
    }
}

pub(super) fn direction_score_is_accepted(best: u32, second: u32) -> bool {
    best <= DIR_MAX_DIFF && second.saturating_sub(best) >= DIR_MIN_MARGIN
}

/// 方向グリフの許容距離（背景不変 glyph_distance）。
/// 正解 実測 0-1（クリーン）/ 14-30（HITS 等が裏にある場合）、
/// 別グリフの 2 位は ≥17 のため、受理はマージン（≥8）と併用する
const DIR_MAX_DIFF: u32 = 32;
/// 1 位と 2 位の最小マージン。これ未満は判別不能として Unknown
pub(super) const DIR_MIN_MARGIN: u32 = 8;
/// グリフとして成立する白ピクセル数の範囲
const DIR_MIN_WHITE: u32 = 40;
const DIR_MAX_WHITE: u32 = 700;

/// 方向グリフをテンプレートマッチ（重心整列 + 微調整シフト）。
/// 戻り値: (dir, uncertain, スコア)
pub(super) fn read_dir(f: &Frame, x0: usize, y0: usize) -> (InputDir, bool, u32) {
    let (m, n_white) = dir_mask(f, x0, y0);
    if n_white < DIR_MIN_WHITE {
        return (InputDir::Unknown, false, u32::MAX); // 空（行なし）
    }
    if n_white > DIR_MAX_WHITE {
        return (InputDir::Unknown, true, u32::MAX); // 全面白フラッシュ
    }
    let cm = match mask_centroid(&m) {
        Some(centroid) => centroid,
        None => unreachable!("a nonempty direction mask must have a centroid"),
    };

    let mut best = (InputDir::Unknown, u32::MAX);
    let mut second = u32::MAX;
    for (ti, t) in DIR_TEMPLATES.iter().enumerate() {
        let ct = mask_centroid(t).expect("direction templates are nonempty");
        let (bx, by) = alignment_offset(cm, ct);
        let mut tbest = u32::MAX;
        // 重心整列 ±1px の微調整
        for dy in fine_offsets(by) {
            for dx in fine_offsets(bx) {
                if within_alignment_window(dx, dy) {
                    let ts = shift_mask(t, dx, dy);
                    let score = glyph_distance(&m, &ts, DIR_W as u32);
                    tbest = tbest.min(score);
                }
            }
        }
        (best, second) = rank_direction_candidate(best, second, (DIR_ORDER[ti], tbest));
    }
    if direction_score_is_accepted(best.1, second) {
        (best.0, false, best.1)
    } else {
        (InputDir::Unknown, true, best.1) // 判別不能（残余の遮蔽等）
    }
}
