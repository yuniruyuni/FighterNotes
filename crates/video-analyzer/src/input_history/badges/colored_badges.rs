use super::super::*;
use super::column_colors::ColorColumn;

pub(super) type BadgeSpan = (BadgeMark, usize, usize);

pub(super) fn collect_colored_badges(
    f: &Frame,
    x1: usize,
    y0: usize,
    col_class: &[ColorColumn],
) -> Vec<BadgeSpan> {
    // 同色チャンクをバッジに結合する。文字付きボックスは白文字が有彩色列を
    // 侵食して 2-4px のギャップを作る（DP 箱 実測）ため、同色なら ≤4px の
    // ギャップを橋渡しする。異なるバッジ同士の実ギャップは ≥5px なので安全。
    // 幅（span = チャンク外接幅）≥8px を採用、≥23px + 内部白文字 = 箱
    let mut badges: Vec<(BadgeMark, usize, usize)> = Vec::new();
    let mut i = 0usize;
    let n = col_class.len();
    while i < n {
        // ラン開始は強列のみ（弱列・ギャップからは始めない）
        let Some((c0, true)) = col_class[i] else {
            i += 1;
            continue;
        };
        let start = i;
        let mut end = i;
        let mut j = i + 1;
        let mut gap = 0usize;
        while j < n {
            match col_class[j] {
                Some((c, _)) if c == c0 => {
                    end = j;
                    gap = 0;
                    j += 1;
                }
                None => {
                    gap += 1;
                    if gap > 4 {
                        break;
                    }
                    j += 1;
                }
                Some(_) => break, // 異色に接触
            }
        }
        let w = end - start + 1;
        if w >= 8 {
            // 箱判定: 幅に加えて「内部の白文字」を要求する。
            // 円のハイライトは上縁の数 px のみ、箱の文字は中央帯に ≥10px
            let mut bright_inside = 0u32;
            for ci in start..=end {
                for ry in 4..14 {
                    let Some((r, g, b)) = f.px(x1 + ci, y0 + ry) else {
                        continue;
                    };
                    if r.min(g).min(b) > 190 {
                        bright_inside += 1;
                    }
                }
            }
            // 箱判定は中央帯の白文字量のみで行う（円 実測 0 / 箱 実測 ≥29、
            // HITS 文字の偽箱は ≤18 と分離。幅条件だと文字侵食で分断された
            // 劣化箱を取りこぼす）
            let boxed = bright_inside >= 25;

            // ── 偽バッジ排除（透過部のキャラ・演出色対策） ──────────────
            // 1. 分類学的拒否: 実在するバッジは 円=teal/黄/赤、箱=橙SP/青DP/
            //    tealDI のみ。橙円・青円は存在しない（肌・炎・HITS 文字の色）
            let taxonomy_ok = if boxed {
                matches!(
                    c0,
                    BadgeColor::Orange | BadgeColor::Green | BadgeColor::Blue
                )
            } else {
                matches!(c0, BadgeColor::Green | BadgeColor::Yellow | BadgeColor::Red)
            };
            // 2. 不透明バッジは上下に黒縁リングを持つ（実測 82-137、
            //    HITS 文字の偽箱は 36）
            let mut rim_luma_dark = 0u32;
            let mut rim_max_160 = 0u32;
            for ci in start..=end {
                for ry in [0usize, 1, 2, 3, 14, 15, 16, 17] {
                    let Some((r, g, b)) = f.px(x1 + ci, y0 + ry) else {
                        continue;
                    };
                    if r.max(g).max(b) < 160 {
                        rim_max_160 += 1;
                    }
                    let luma = (u32::from(r) + u32::from(g) + u32::from(b)) / 3;
                    if luma < 80 {
                        rim_luma_dark += 1;
                    }
                }
            }
            // 3. 円の内部はべた塗り（隣接差 実測 ≤5）。透過部の背景は
            //    ディザで描画されるため隣接差が大きい（肌 実測 28）。
            //    クラシックの円はグリフ（拳/足）のエッジで隣接差が 14-24 に
            //    なるため、グリフ照合の一致を平滑性の代替として認める
            //    （ディザ背景の偽円はグリフテンプレートに一致しない）
            let mut smooth_ok = true;
            if !boxed {
                let (mut diff_sum, mut diff_n) = (0u32, 0u32);
                for ci in start..end {
                    for ry in 4..14 {
                        let Some((r1, g1, b1)) = f.px(x1 + ci, y0 + ry) else {
                            continue;
                        };
                        let Some((r2, g2, b2)) = f.px(x1 + ci + 1, y0 + ry) else {
                            continue;
                        };
                        if r1.min(g1).min(b1) > 190 || r2.min(g2).min(b2) > 190 {
                            continue;
                        }
                        diff_sum += (r1.abs_diff(r2) as u32
                            + g1.abs_diff(g2) as u32
                            + b1.abs_diff(b2) as u32)
                            / 3;
                        diff_n += 1;
                    }
                }
                smooth_ok = diff_n == 0 || diff_sum / diff_n <= 12;
            }
            // グリフ照合の前提: 本物のクラシック円は白画素をほぼ持たず
            // （実測 0-1）幅 22-23px。白文字で分断された箱の断片（bright
            // 24-34, w 11-12）が円+グリフとしてすり抜けるのを防ぐ
            let glyph = if !boxed && bright_inside <= 5 && w >= 16 {
                classify_btn_glyph_in_span(f, x1 + start, y0, w)
            } else {
                None
            };

            // リムは無彩色の真黒だけに限定しない。半透明パネルでは背景色が
            // 混ざるため、低輝度または全チャンネルが中暗度の画素を認める。
            // 一方、彩度の高い明るい幕は平均輝度だけを暗さと誤認しない。
            // 円・箱の実幅を大きく超える色帯も背景として棄却する。
            let rim_ok =
                w <= 40 && (rim_luma_dark >= (w as u32 / 8).max(2) || rim_max_160 >= 2 * w as u32);
            if taxonomy_ok && rim_ok && (smooth_ok || glyph.is_some()) {
                // 同一箱の断片化による重複を防ぐ（同色・箱・近接なら統合）
                if boxed {
                    if let Some((last, _, _)) = badges.last() {
                        if *last
                            == (BadgeMark {
                                color: c0,
                                boxed: true,
                                glyph: None,
                            })
                        {
                            i = (end + 1).max(i + 1);
                            continue;
                        }
                    }
                }
                badges.push((
                    BadgeMark {
                        color: c0,
                        boxed,
                        glyph,
                    },
                    start,
                    end,
                ));
            }
        }
        i = (end + 1).max(i + 1);
    }
    badges
}
