use super::super::*;
use super::colored_badges::BadgeSpan;

pub(super) fn chain_badges(badges: &[BadgeSpan], zone_w: usize, is_p1: bool) -> Vec<BadgeMark> {
    // ── 連鎖性ガード ─────────────────────────────────────────────────────
    // バッジは方向グリフ側から詰めて並ぶ。起点（P1 = 帯左端 / P2 = 帯右端）
    // から 34px 以内で連鎖しないバッジは、プレイフィールド上のキャラ・演出の
    // 色塊（帯を広げたことで入り込む）なので捨てる
    const BADGE_CHAIN_GAP: usize = 34;
    let chained: Vec<BadgeMark> = {
        let mut out = Vec::new();
        if is_p1 {
            let mut edge = 0usize; // 前バッジの終端（帯先頭から）
            for &(m, st, en) in badges.iter() {
                if st <= edge + BADGE_CHAIN_GAP {
                    out.push(m);
                    edge = en;
                } else {
                    break;
                }
            }
        } else {
            let mut edge = zone_w.saturating_sub(1);
            for &(m, st, en) in badges.iter().rev() {
                if en + BADGE_CHAIN_GAP >= edge {
                    out.insert(0, m);
                    edge = st;
                } else {
                    break;
                }
            }
        }
        out
    };
    chained
}
