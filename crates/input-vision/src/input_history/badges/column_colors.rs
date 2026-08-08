use super::super::*;

pub(super) type ColorColumn = Option<(BadgeColor, bool)>;

pub(super) fn classify_color_columns(
    f: &Frame,
    x1: usize,
    x2: usize,
    y0: usize,
) -> Vec<ColorColumn> {
    let mut col_class: Vec<Option<(BadgeColor, bool)>> = Vec::with_capacity(x2 - x1);
    for x in x1..x2 {
        let mut counts = [0u32; 5]; // Y, O, G, B, R
        for ry in 0..DIGIT_H {
            let Some((r, g, b)) = f.px(x, y0 + ry) else {
                continue;
            };
            let (rf, gf, bf) = (r as f32, g as f32, b as f32);
            if rf > 180.0 && gf > 180.0 && bf > 180.0 {
                continue; // 白（文字・ハイライト）は色相判定から除外
            }
            let [h, s, v] = pixel_color::rgb_to_hsv(rf, gf, bf);
            if s > 90.0 && v > 90.0 {
                if (24.0..=40.0).contains(&h) {
                    counts[0] += 1;
                } else if (5.0..24.0).contains(&h) {
                    counts[1] += 1;
                }
                // SP 箱 実測 6-17
                else if (85.0..=100.0).contains(&h) {
                    counts[2] += 1;
                }
                // teal 円 実測 92-99
                else if (100.0..=135.0).contains(&h) {
                    counts[3] += 1;
                }
                // DP 箱 実測 102-110
                else if h >= 168.0 {
                    counts[4] += 1;
                } // 赤円 実測 173-178
            }
        }
        let (bi, &bc) = counts.iter().enumerate().max_by_key(|(_, &c)| c).unwrap();
        // 白文字が乗るボックス列は有彩色画素が痩せる（実測 n=2-4）ため
        // 2 段階分類: 強（≥3、ラン開始可能）/ 弱（≥2、継続のみ）
        col_class.push(if bc >= 2 {
            let color = match bi {
                0 => BadgeColor::Yellow,
                1 => BadgeColor::Orange,
                2 => BadgeColor::Green,
                3 => BadgeColor::Blue,
                _ => BadgeColor::Red,
            };
            Some((color, bc >= 3))
        } else {
            None
        });
    }
    col_class
}
