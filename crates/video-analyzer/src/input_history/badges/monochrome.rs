use super::super::*;
use super::column_colors::ColorColumn;

pub(super) fn detect_monochrome_controls(
    f: &Frame,
    mono_range: (u32, u32),
    x1: usize,
    zone_w: usize,
    col_class: &[ColorColumn],
    badges: &[BadgeMark],
    y0: usize,
) -> (bool, bool) {
    // 無彩色バッジ（AUTO 箱 / 投げ円）: 有彩色列を除いた暗部・白部の量で判定。
    //   AUTO 箱: 暗い箱 28x12（暗部 ≥80px 実測 ≈150）+ 白文字（≥25px）
    //   投げ円: 暗い円 21px + 白い手アイコン（実測 明部 ≈120 / 暗部 ≈50）
    // 明るい背景透けは暗部が無いため誤検出しない。
    // 検出帯は先頭スロット群（mono_range）に限定（広帯だとノイズ蓄積）
    let (mx1, mx2) = (mono_range.0 as usize, mono_range.1 as usize);
    let mono_w = mx2 - mx1;
    let mut n_dark = 0u32;
    let mut n_bright = 0u32;
    let mut dark_top = 0u32; // 行上端バンド（円の上リング）
    let mut dark_mid = 0u32;
    let mut dark_bot = 0u32; // 行下端バンド（円の下リング）
    let mut bright_col = vec![0u32; mono_w];
    for (ci, x) in (mx1..mx2).enumerate() {
        let zi = x.saturating_sub(x1);
        if zi < zone_w && col_class[zi].is_some() {
            continue;
        }
        for ry in 0..DIGIT_H {
            let Some((r, g, b)) = f.px(x, y0 + ry) else {
                continue;
            };
            let mn = r.min(g).min(b);
            if mn < 60 {
                n_dark += 1;
                if ry <= 3 {
                    dark_top += 1;
                } else if ry >= 14 {
                    dark_bot += 1;
                } else {
                    dark_mid += 1;
                }
            } else if mn > 190 {
                n_bright += 1;
                bright_col[ci] += 1;
            }
        }
    }
    // AUTO 箱の暗部実測 ≥150、投げ円は背景次第で 50-81 まで上がるため
    // 閾値はその中間の 110 に置く
    let auto = n_dark >= 110 && n_bright >= 25;
    // 投げ円: 白い手アイコンが帯の内側に孤立した明部ブロックを作る
    // （実測: 幅 16 列・列あたり 6-12px・周囲ゼロ）。
    // 帯の端に接する明部は背景キャラ・エフェクトの漏れ込みなので除外する。
    let mut hand_blob = false;
    {
        let mut i = 0usize;
        while i < mono_w {
            if bright_col[i] >= 4 {
                let start = i;
                while i < mono_w && bright_col[i] >= 4 {
                    i += 1;
                }
                let w = i - start;
                if (10..=24).contains(&w) && start >= 3 && i + 3 <= mono_w {
                    hand_blob = true;
                }
            } else {
                i += 1;
            }
        }
    }
    // 上下の暗リング（実測 top23/mid15/bot20）+ 中央暗部過多（背景影）の除外
    let throw = !auto
        && hand_blob
        && n_bright >= 60
        && dark_top >= 10
        && dark_bot >= 10
        && dark_mid < dark_top + dark_bot;
    // クラシックの投げは 弱P+弱K 同時押し（Modern の手アイコン円は出ない）
    let throw = throw || classic_throw(badges);
    (auto, throw)
}
