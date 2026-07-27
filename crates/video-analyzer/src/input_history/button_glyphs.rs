use super::*;

// ── クラシックボタングリフ分類 ───────────────────────────────────────────────

/// バッジ円のグリフ照合ボックス幅（円チャンク実測幅 22-23px）
pub(super) const BTN_GLYPH_W: u32 = 23;
/// グリフ受理の最大距離（ラベル付き実測 p95=37 / 劣化・遮蔽は 44 超）
const BTN_GLYPH_MAX: u32 = 45;
/// P/K の曖昧マージン（これ未満は判定保留 = glyph None）
const BTN_GLYPH_AMBIG: u32 = 3;
/// グリフ暗画素の閾値。彩色 fill は min(RGB) が低く「暗」、グリフの
/// 淡色ストロークは「明」に落ちる（グリフ = fill 内の明るい穴として写る）
const BTN_GLYPH_DARK: u8 = 100;
/// グリフ照合に必要な内部の明画素（穴）量。Modern の無地円は fill が
/// 内部を埋め尽くし穴が無い（実測 16-26）。クラシック実測 105-205
const BTN_GLYPH_MIN_LIGHT: u32 = 60;

/// 円チャンク内部の暗画素マスクから拳/足グリフを分類する。
/// Modern の無地円はグリフ画素がほぼ無く距離が大きくなり None に落ちる。
///
/// P2 側のアイコンが鏡像かは未実測のため、正順と左右反転の両方を試して
/// 良い方を採る（Modern 無地円はどちらでも棄却される）。
pub(super) fn classify_btn_glyph(f: &Frame, x_start: usize, y0: usize) -> Option<BtnGlyph> {
    // 暗画素マスク（楕円内部限定。円外の透過背景ノイズを避ける）
    let mut mask = [0u64; DIGIT_H];
    for (ry, row) in mask.iter_mut().enumerate() {
        let mut bits = 0u64;
        for k in 0..BTN_GLYPH_W as usize {
            let Some((r, g, b)) = f.px(x_start + k, y0 + ry) else {
                continue;
            };
            if r.min(g).min(b) < BTN_GLYPH_DARK {
                bits |= 1 << k;
            }
        }
        *row = bits & BTN_GLYPH_INTERIOR[ry];
    }

    // 無地円ゲート: グリフは fill 内の明るい穴として写る。穴が無い円は
    // Modern の無地ボタンなので照合しない（全面ダークだと P テンプレートの
    // 侵食スコアが偶然低く出て誤マッチする）
    let interior_total: u32 = BTN_GLYPH_INTERIOR.iter().map(|r| r.count_ones()).sum();
    let dark_total: u32 = mask.iter().map(|r| r.count_ones()).sum();
    if interior_total - dark_total < BTN_GLYPH_MIN_LIGHT {
        return None;
    }

    let flip = |m: &[u64; DIGIT_H]| -> [u64; DIGIT_H] {
        let mut out = [0u64; DIGIT_H];
        for (i, &row) in m.iter().enumerate() {
            let mut rev = 0u64;
            for k in 0..BTN_GLYPH_W {
                if row & (1 << k) != 0 {
                    rev |= 1 << (BTN_GLYPH_W - 1 - k);
                }
            }
            out[i] = rev;
        }
        out
    };

    // ±1px シフト込みの最小距離（チャンク開始列は色検出で ±1 揺れる）
    let dist_shifted = |m: &[u64; DIGIT_H], t: &[u64; DIGIT_H]| -> u32 {
        let col_mask: u64 = (1u64 << BTN_GLYPH_W) - 1;
        let mut best = u32::MAX;
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                let mut s = [0u64; DIGIT_H];
                for (i, srow) in s.iter_mut().enumerate() {
                    let si = i as i32 - dy;
                    if !(0..DIGIT_H as i32).contains(&si) {
                        continue;
                    }
                    let r = m[si as usize];
                    *srow = if dx >= 0 {
                        (r << dx) & col_mask
                    } else {
                        r >> (-dx)
                    };
                    *srow &= BTN_GLYPH_INTERIOR[i];
                }
                best = best.min(glyph_distance(&s, t, BTN_GLYPH_W));
            }
        }
        best
    };

    let flipped = flip(&mask);
    let dp = dist_shifted(&mask, &BTN_GLYPH_PUNCH).min(dist_shifted(&flipped, &BTN_GLYPH_PUNCH));
    let dk = dist_shifted(&mask, &BTN_GLYPH_KICK).min(dist_shifted(&flipped, &BTN_GLYPH_KICK));
    let (best, margin) = if dp <= dk {
        (dp, dk - dp)
    } else {
        (dk, dp - dk)
    };
    if best > BTN_GLYPH_MAX || margin < BTN_GLYPH_AMBIG {
        return None;
    }
    Some(if dp <= dk {
        BtnGlyph::Punch
    } else {
        BtnGlyph::Kick
    })
}

/// クラシックの投げ入力（弱P+弱K 同時押し）をバッジ列から検出する。
pub fn classic_throw(badges: &[BadgeMark]) -> bool {
    let has = |g: BtnGlyph| {
        badges
            .iter()
            .any(|b| !b.boxed && b.color == BadgeColor::Green && b.glyph == Some(g))
    };
    has(BtnGlyph::Punch) && has(BtnGlyph::Kick)
}
