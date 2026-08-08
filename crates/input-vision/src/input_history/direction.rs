use super::*;

// ── 方向読み取り ─────────────────────────────────────────────────────────────

pub(super) fn dir_mask(f: &Frame, x0: usize, y0: usize) -> ([u64; DIR_H], u32) {
    let mut m = [0u64; DIR_H];
    let mut n = 0u32;
    for (ry, row) in m.iter_mut().enumerate() {
        for rx in 0..DIR_W {
            if f.is_white(x0 + rx, y0 + ry) {
                *row |= 1 << rx;
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
        while bits != 0 {
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
        out[y as usize] = if dx >= 0 {
            (row << dx) & col_mask
        } else {
            row >> (-dx)
        };
    }
    out
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
    let Some(cm) = mask_centroid(&m) else {
        return (InputDir::Unknown, false, u32::MAX);
    };

    let mut best = (InputDir::Unknown, u32::MAX);
    let mut second = u32::MAX;
    for (ti, t) in DIR_TEMPLATES.iter().enumerate() {
        let Some(ct) = mask_centroid(t) else { continue };
        let bx = (cm.0 - ct.0).round() as i32;
        let by = (cm.1 - ct.1).round() as i32;
        let mut tbest = u32::MAX;
        // 重心整列 ±1px の微調整
        for dy in (by - 1)..=(by + 1) {
            for dx in (bx - 1)..=(bx + 1) {
                if dx.abs() > 6 || dy.abs() > 6 {
                    continue;
                }
                let ts = shift_mask(t, dx, dy);
                let score = glyph_distance(&m, &ts, DIR_W as u32);
                tbest = tbest.min(score);
            }
        }
        if tbest < best.1 {
            second = best.1;
            best = (DIR_ORDER[ti], tbest);
        } else if tbest < second {
            second = tbest;
        }
    }
    if best.1 <= DIR_MAX_DIFF && second.saturating_sub(best.1) >= DIR_MIN_MARGIN {
        (best.0, false, best.1)
    } else {
        (InputDir::Unknown, true, best.1) // 判別不能（残余の遮蔽等）
    }
}
