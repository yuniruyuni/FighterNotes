use crate::input_history::{match_digit_gray, Frame};

use super::{
    model::{
        AttackAttribute, AttackInfoFrameInspection, AttackInfoRoi, AttackInfoRois, AttackInfoSide,
        AttackInfoSideInspection, AttackInfoSideRois,
    },
    templates::ATTRIBUTE_TEMPLATES,
};

const BASE_WIDTH: usize = 1920;
const NUMERIC_WIDTH: usize = 190;
const NUMERIC_HEIGHT: usize = 56;
const ATTRIBUTE_WIDTH: usize = 32;
const ATTRIBUTE_HEIGHT: usize = 20;
const ROW_SCAN_HEIGHT: usize = 29;

const P1_NUMERIC_SOURCE: (usize, usize) = (600, 174);
const P1_ATTRIBUTE_SOURCE: (usize, usize) = (749, 236);
const P2_NUMERIC_SOURCE: (usize, usize) = (1136, 174);
const P2_ATTRIBUTE_SOURCE: (usize, usize) = (1141, 236);

const P1_NUMERIC_PACKED: (usize, usize) = (0, 0);
const P1_ATTRIBUTE_PACKED: (usize, usize) = (200, 0);
const P2_NUMERIC_PACKED: (usize, usize) = (1559, 0);
const P2_ATTRIBUTE_PACKED: (usize, usize) = (1759, 0);

const FULL_ROIS: AttackInfoRois = AttackInfoRois {
    p1: AttackInfoSideRois {
        numeric: AttackInfoRoi {
            x1: P1_NUMERIC_SOURCE.0 as u32,
            x2: (P1_NUMERIC_SOURCE.0 + NUMERIC_WIDTH) as u32,
            y1: P1_NUMERIC_SOURCE.1 as u32,
            y2: (P1_NUMERIC_SOURCE.1 + NUMERIC_HEIGHT) as u32,
        },
        attribute: AttackInfoRoi {
            x1: P1_ATTRIBUTE_SOURCE.0 as u32,
            x2: (P1_ATTRIBUTE_SOURCE.0 + ATTRIBUTE_WIDTH) as u32,
            y1: P1_ATTRIBUTE_SOURCE.1 as u32,
            y2: (P1_ATTRIBUTE_SOURCE.1 + ATTRIBUTE_HEIGHT) as u32,
        },
    },
    p2: AttackInfoSideRois {
        numeric: AttackInfoRoi {
            x1: P2_NUMERIC_SOURCE.0 as u32,
            x2: (P2_NUMERIC_SOURCE.0 + NUMERIC_WIDTH) as u32,
            y1: P2_NUMERIC_SOURCE.1 as u32,
            y2: (P2_NUMERIC_SOURCE.1 + NUMERIC_HEIGHT) as u32,
        },
        attribute: AttackInfoRoi {
            x1: P2_ATTRIBUTE_SOURCE.0 as u32,
            x2: (P2_ATTRIBUTE_SOURCE.0 + ATTRIBUTE_WIDTH) as u32,
            y1: P2_ATTRIBUTE_SOURCE.1 as u32,
            y2: (P2_ATTRIBUTE_SOURCE.1 + ATTRIBUTE_HEIGHT) as u32,
        },
    },
};

#[derive(Clone, Copy)]
struct NumericRead {
    last_damage: u32,
    scaling_percent: u32,
    combo_damage: u32,
    max_combo_damage: u32,
    score: u32,
}

#[derive(Clone, Copy)]
struct AttributeRead {
    value: AttackAttribute,
    score: u32,
    margin: u32,
}

#[derive(Clone, Copy, Default)]
struct Component {
    min_x: usize,
    min_y: usize,
    max_x: usize,
    max_y: usize,
    pixels: usize,
}

#[derive(Clone, Copy)]
struct DigitCandidate {
    x: usize,
    end_x: usize,
    digit: u32,
    score: u32,
}

#[derive(Clone, Copy)]
enum NumericRowKind {
    Damage,
    Combo,
}

pub fn read_attack_info(rgba: &[u8], width: u32, height: u32) -> AttackInfoFrameInspection {
    if width as usize != BASE_WIDTH || rgba.len() < width as usize * height as usize * 4 {
        return empty_inspection();
    }
    read_frame(
        rgba,
        width as usize,
        height as usize,
        P1_NUMERIC_SOURCE,
        P1_ATTRIBUTE_SOURCE,
        P2_NUMERIC_SOURCE,
        P2_ATTRIBUTE_SOURCE,
    )
}

pub fn read_attack_info_from_meter_strip(rgba: &[u8], width: u32) -> AttackInfoFrameInspection {
    let height = rgba.len() / (width as usize).saturating_mul(4).max(1);
    if width as usize != BASE_WIDTH || height < NUMERIC_HEIGHT {
        return empty_inspection();
    }
    read_frame(
        rgba,
        width as usize,
        height,
        P1_NUMERIC_PACKED,
        P1_ATTRIBUTE_PACKED,
        P2_NUMERIC_PACKED,
        P2_ATTRIBUTE_PACKED,
    )
}

fn read_frame(
    rgba: &[u8],
    width: usize,
    height: usize,
    p1_numeric: (usize, usize),
    p1_attribute: (usize, usize),
    p2_numeric: (usize, usize),
    p2_attribute: (usize, usize),
) -> AttackInfoFrameInspection {
    AttackInfoFrameInspection {
        p1: read_side(rgba, width, height, p1_numeric, p1_attribute),
        p2: read_side(rgba, width, height, p2_numeric, p2_attribute),
        rois: FULL_ROIS,
    }
}

fn read_side(
    rgba: &[u8],
    width: usize,
    height: usize,
    numeric_origin: (usize, usize),
    attribute_origin: (usize, usize),
) -> Option<AttackInfoSideInspection> {
    if numeric_origin.0 + NUMERIC_WIDTH > width
        || numeric_origin.1 + NUMERIC_HEIGHT > height
        || attribute_origin.0 + ATTRIBUTE_WIDTH > width
        || attribute_origin.1 + ATTRIBUTE_HEIGHT > height
    {
        return None;
    }
    let attribute = read_attribute(rgba, width, attribute_origin)?;
    let numeric = read_numeric(rgba, width, numeric_origin)?;
    Some(AttackInfoSideInspection {
        value: AttackInfoSide {
            last_damage: numeric.last_damage,
            scaling_percent: numeric.scaling_percent,
            combo_damage: numeric.combo_damage,
            max_combo_damage: numeric.max_combo_damage,
            attribute: attribute.value,
        },
        numeric_score: numeric.score,
        attribute_score: attribute.score,
        attribute_margin: attribute.margin,
    })
}

fn read_numeric(rgba: &[u8], width: usize, origin: (usize, usize)) -> Option<NumericRead> {
    for threshold in [210, 180] {
        let Some(damage) = read_numeric_row(
            rgba,
            width,
            origin,
            0,
            25,
            threshold,
            NumericRowKind::Damage,
        ) else {
            continue;
        };
        let Some(combo) = read_numeric_row(
            rgba,
            width,
            origin,
            27,
            NUMERIC_HEIGHT,
            threshold,
            NumericRowKind::Combo,
        ) else {
            continue;
        };
        if !numeric_values_are_plausible(damage.0, damage.1, combo.0, combo.1) {
            continue;
        }
        return Some(NumericRead {
            last_damage: damage.0,
            scaling_percent: damage.1,
            combo_damage: combo.0,
            max_combo_damage: combo.1,
            score: damage.2.saturating_add(combo.2),
        });
    }
    None
}

fn read_numeric_row(
    rgba: &[u8],
    width: usize,
    origin: (usize, usize),
    row_y1: usize,
    row_y2: usize,
    threshold: u8,
    kind: NumericRowKind,
) -> Option<(u32, u32, u32)> {
    let row_height = row_y2.saturating_sub(row_y1);
    if row_height == 0 || row_height > ROW_SCAN_HEIGHT {
        return None;
    }
    let mut white = [false; NUMERIC_WIDTH * ROW_SCAN_HEIGHT];
    for y in 0..row_height {
        for x in 0..NUMERIC_WIDTH {
            white[y * NUMERIC_WIDTH + x] =
                gray(rgba, width, origin.0 + x, origin.1 + row_y1 + y) > threshold;
        }
    }

    let mut visited = [false; NUMERIC_WIDTH * ROW_SCAN_HEIGHT];
    let mut stack = [0usize; NUMERIC_WIDTH * ROW_SCAN_HEIGHT];
    let mut components = Vec::with_capacity(20);
    for start in 0..row_height * NUMERIC_WIDTH {
        if !white[start] || visited[start] {
            continue;
        }
        let mut component = Component {
            min_x: start % NUMERIC_WIDTH,
            max_x: start % NUMERIC_WIDTH,
            min_y: start / NUMERIC_WIDTH,
            max_y: start / NUMERIC_WIDTH,
            pixels: 0,
        };
        let mut stack_len = 1;
        stack[0] = start;
        visited[start] = true;
        while stack_len > 0 {
            stack_len -= 1;
            let current = stack[stack_len];
            let x = current % NUMERIC_WIDTH;
            let y = current / NUMERIC_WIDTH;
            component.min_x = component.min_x.min(x);
            component.max_x = component.max_x.max(x);
            component.min_y = component.min_y.min(y);
            component.max_y = component.max_y.max(y);
            component.pixels += 1;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= NUMERIC_WIDTH as i32 || ny >= row_height as i32 {
                        continue;
                    }
                    let next = ny as usize * NUMERIC_WIDTH + nx as usize;
                    if white[next] && !visited[next] {
                        visited[next] = true;
                        stack[stack_len] = next;
                        stack_len += 1;
                    }
                }
            }
        }
        components.push(component);
    }

    let frame = Frame::new(rgba, width, 0, threshold);
    let mut candidates = Vec::with_capacity(12);
    for component in &components {
        let component_width = component.max_x - component.min_x + 1;
        let component_height = component.max_y - component.min_y + 1;
        // 数字は16〜18px。括弧は19〜20px、%の各部は9px以下なので、
        // 高さで句読記号を先に除外すると細い「1」も安全に残せる。
        if !(3..=11).contains(&component_width)
            || !(14..=18).contains(&component_height)
            || component.pixels < 12
        {
            continue;
        }
        let x0 = (origin.0 + component.min_x).saturating_sub(1);
        let y0 = (origin.1 + row_y1 + component.min_y).saturating_sub(1);
        let (digit, score, margin) = match_digit_gray(&frame, x0, y0);
        let accepted = score <= 28 || (score <= 40 && margin >= 15);
        if accepted && margin >= 3 {
            candidates.push(DigitCandidate {
                x: component.min_x,
                end_x: component.max_x,
                digit,
                score,
            });
        }
    }
    candidates.sort_by_key(|candidate| candidate.x);

    parse_anchored_row(&components, &candidates, kind)
}

fn numeric_values_are_plausible(
    last_damage: u32,
    scaling_percent: u32,
    combo_damage: u32,
    max_combo_damage: u32,
) -> bool {
    // リプレイ中の開始体力とKO時の超過ダメージに余裕を持たせる。
    // 桁混入は括弧アンカーで防ぎ、ここでは明らかな5桁異常だけを落とす。
    const MAX_TOTAL_DAMAGE: u32 = 20_000;
    scaling_percent <= 100
        && last_damage <= combo_damage
        && (last_damage == 0) == (combo_damage == 0)
        && combo_damage <= max_combo_damage
        && max_combo_damage <= MAX_TOTAL_DAMAGE
}

fn parse_anchored_row(
    components: &[Component],
    candidates: &[DigitCandidate],
    kind: NumericRowKind,
) -> Option<(u32, u32, u32)> {
    let parentheses = components.iter().filter(is_parenthesis).collect::<Vec<_>>();
    let mut reads = Vec::new();
    for (left_index, left) in parentheses.iter().enumerate() {
        for right in parentheses.iter().skip(left_index + 1) {
            let inner_width = right.min_x.saturating_sub(left.max_x);
            if !(12..=62).contains(&inner_width) {
                continue;
            }
            let Some(first_group) = digits_before_parenthesis(candidates, left) else {
                continue;
            };
            let second_group = match kind {
                NumericRowKind::Damage => scaling_digits(components, candidates, left, right),
                NumericRowKind::Combo => digits_between_parentheses(candidates, left, right),
            };
            let Some(second_group) = second_group else {
                continue;
            };
            let (first, first_score) = parse_digit_group(first_group);
            let (second, second_score) = parse_digit_group(second_group);
            reads.push((first, second, first_score.saturating_add(second_score)));
        }
    }
    let first = *reads.first()?;
    if reads
        .iter()
        .any(|candidate| candidate.0 != first.0 || candidate.1 != first.1)
    {
        return None;
    }
    reads.into_iter().min_by_key(|candidate| candidate.2)
}

fn is_parenthesis(component: &&Component) -> bool {
    let width = component.max_x - component.min_x + 1;
    let height = component.max_y - component.min_y + 1;
    (2..=6).contains(&width) && (18..=21).contains(&height) && component.pixels >= 14
}

fn digits_before_parenthesis<'a>(
    candidates: &'a [DigitCandidate],
    parenthesis: &Component,
) -> Option<&'a [DigitCandidate]> {
    let end = candidates.partition_point(|candidate| candidate.end_x < parenthesis.min_x);
    let before = &candidates[..end];
    let last = before.last()?;
    if !(7..=15).contains(&parenthesis.min_x.saturating_sub(last.end_x)) {
        return None;
    }
    let mut start = before.len() - 1;
    while start > 0 && digit_gap(&before[start - 1], &before[start]) <= 8 {
        start -= 1;
    }
    let group = &before[start..];
    valid_group(group).then_some(group)
}

fn digits_between_parentheses<'a>(
    candidates: &'a [DigitCandidate],
    left: &Component,
    right: &Component,
) -> Option<&'a [DigitCandidate]> {
    let start = candidates.partition_point(|candidate| candidate.x <= left.max_x);
    let end = candidates.partition_point(|candidate| candidate.end_x < right.min_x);
    let group = &candidates[start..end];
    if !valid_group(group)
        || !(1..=8).contains(&group[0].x.saturating_sub(left.max_x))
        || !(1..=8).contains(&right.min_x.saturating_sub(group.last()?.end_x))
        || !digits_are_contiguous(group)
    {
        return None;
    }
    Some(group)
}

fn scaling_digits<'a>(
    components: &[Component],
    candidates: &'a [DigitCandidate],
    left: &Component,
    right: &Component,
) -> Option<&'a [DigitCandidate]> {
    let start = candidates.partition_point(|candidate| candidate.x <= left.max_x);
    let inside_end = candidates.partition_point(|candidate| candidate.end_x < right.min_x);
    let inside = &candidates[start..inside_end];
    let mut valid = Vec::new();
    for length in 1..=inside.len().min(3) {
        let group = &inside[..length];
        let last = group.last()?;
        if !valid_group(group)
            || !(1..=8).contains(&group[0].x.saturating_sub(left.max_x))
            || !digits_are_contiguous(group)
            || !(14..=25).contains(&right.min_x.saturating_sub(last.end_x))
            || !has_percent_evidence(components, last.end_x, right.min_x)
        {
            continue;
        }
        valid.push(group);
    }
    (valid.len() == 1).then(|| valid[0])
}

fn has_percent_evidence(components: &[Component], digit_end: usize, right_x: usize) -> bool {
    let evidence = components.iter().filter(|component| {
        component.min_x > digit_end
            && component.max_x < right_x
            && !is_parenthesis(component)
            && component.pixels >= 5
    });
    let (pixels, rightmost) = evidence.fold((0usize, 0usize), |(pixels, rightmost), component| {
        (
            pixels.saturating_add(component.pixels),
            rightmost.max(component.max_x),
        )
    });
    pixels >= 20 && right_x.saturating_sub(rightmost) <= 8
}

fn valid_group(group: &[DigitCandidate]) -> bool {
    !group.is_empty() && group.len() <= 5
}

fn digits_are_contiguous(group: &[DigitCandidate]) -> bool {
    group
        .windows(2)
        .all(|pair| (1..=8).contains(&digit_gap(&pair[0], &pair[1])))
}

fn digit_gap(left: &DigitCandidate, right: &DigitCandidate) -> usize {
    right.x.saturating_sub(left.end_x)
}

fn parse_digit_group(group: &[DigitCandidate]) -> (u32, u32) {
    group.iter().fold((0u32, 0u32), |(value, score), digit| {
        (
            value.saturating_mul(10).saturating_add(digit.digit),
            score.saturating_add(digit.score),
        )
    })
}

fn read_attribute(rgba: &[u8], width: usize, origin: (usize, usize)) -> Option<AttributeRead> {
    for threshold in [210u8, 180u8] {
        let mut observed = [0u32; ATTRIBUTE_HEIGHT];
        let mut white_count = 0u32;
        for (y, row) in observed.iter_mut().enumerate() {
            for x in 0..ATTRIBUTE_WIDTH {
                if gray(rgba, width, origin.0 + x, origin.1 + y) > threshold {
                    *row |= 1 << x;
                    white_count += 1;
                }
            }
        }
        if !(60..=280).contains(&white_count) {
            continue;
        }
        let mut scores = ATTRIBUTE_TEMPLATES
            .iter()
            .map(|(attribute, template)| (*attribute, attribute_distance(&observed, template)))
            .collect::<Vec<_>>();
        scores.sort_by_key(|(_, score)| *score);
        let (value, score) = scores[0];
        let margin = scores[1].1.saturating_sub(score);
        let max_score = if threshold == 210 { 90 } else { 100 };
        if score <= max_score && margin >= 20 {
            return Some(AttributeRead {
                value,
                score,
                margin,
            });
        }
    }
    None
}

fn attribute_distance(observed: &[u32; ATTRIBUTE_HEIGHT], template: &[u32; 20]) -> u32 {
    let mut best = u32::MAX;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            let mut score = 0u32;
            for (y, observed_row) in observed.iter().enumerate() {
                let source_y = y as i32 - dy;
                let template_row = if (0..ATTRIBUTE_HEIGHT as i32).contains(&source_y) {
                    template[source_y as usize]
                } else {
                    0
                };
                let shifted = if dx < 0 {
                    template_row >> (-dx as u32)
                } else {
                    template_row << (dx as u32)
                };
                score += (observed_row ^ shifted).count_ones();
            }
            best = best.min(score);
        }
    }
    best
}

fn gray(rgba: &[u8], width: usize, x: usize, y: usize) -> u8 {
    let index = (y * width + x) * 4;
    if index + 2 >= rgba.len() {
        return 0;
    }
    rgba[index].min(rgba[index + 1]).min(rgba[index + 2])
}

fn empty_inspection() -> AttackInfoFrameInspection {
    AttackInfoFrameInspection {
        p1: None,
        p2: None,
        rois: FULL_ROIS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 数値行の走査は、読めない入力に対して必ず何も返さない。ここが値を
    /// 返してしまうと、上位の妥当性検査（`numeric_values_are_plausible`）が
    /// 存在しない数字を受け取ることになる。
    #[test]
    fn an_unreadable_numeric_row_yields_nothing() {
        let width = BASE_WIDTH;
        let blank = vec![0u8; width * 64 * 4];
        let read =
            |y1, y2| read_numeric_row(&blank, width, (100, 0), y1, y2, 210, NumericRowKind::Combo);

        // 白画素が一つも無い行。
        assert_eq!(read(0, 25), None);
        // 高さの無い行。
        assert_eq!(read(10, 10), None);
        // 走査バッファに収まらない高さ。
        assert_eq!(read(0, ROW_SCAN_HEIGHT + 1), None);
    }

    /// 括弧で挟まれた数値行を読み切れることを確かめる。
    ///
    /// 桁は連結成分の外接矩形から 1px 外側を起点に照合するので、明るい画素が
    /// テンプレートの縁から 1px 内側に収まる数字でないと素直に組み立てられ
    /// ない。その条件を満たすのは 7 だけ（input-vision 側で固定してある）。
    #[test]
    fn a_parenthesised_numeric_row_reads_both_groups() {
        use input_vision::test_support::paint_digit;

        let width = BASE_WIDTH;
        let origin = (100usize, 5usize);
        let mut rgba = vec![0u8; width * 64 * 4];

        // 走査窓を基準に配置する。数字はテンプレート左上、括弧は塗り潰し。
        let mut paint = |x: usize, y: usize, digit: usize| {
            paint_digit(&mut rgba, width, origin.0 + x, origin.1 + y, digit);
        };
        for x in [0, 11] {
            paint(x, 2, 7);
        }
        for x in [34, 45, 56] {
            paint(x, 2, 7);
        }

        let mut bar = |x0: usize| {
            for y in 2..=20 {
                for x in x0..x0 + 3 {
                    let index = ((origin.1 + y) * width + origin.0 + x) * 4;
                    rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        };
        bar(28);
        bar(68);

        let read = read_numeric_row(&rgba, width, origin, 0, 25, 210, NumericRowKind::Combo);

        let (first, second, score) = read.expect("括弧が揃った行は読める");
        assert_eq!((first, second), (77, 777));
        assert_eq!(score, 0, "テンプレートそのものなので誤差は出ない");
    }

    /// 括弧のアンカーが無い行は、数字が並んでいても読み取らない。
    #[test]
    fn digits_without_parentheses_yield_nothing() {
        let width = BASE_WIDTH;
        let mut rgba = vec![0u8; width * 64 * 4];
        // 数字くらいの大きさの白い塊を等間隔に置く。括弧に見える細長い
        // 塊は無いので、アンカーが取れず読み取りは成立しない。
        for block in 0..3 {
            for y in 4..20 {
                for x in 0..8 {
                    let index = (y * width + 100 + block * 15 + x) * 4;
                    rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        }

        let read = read_numeric_row(&rgba, width, (100, 0), 0, 25, 210, NumericRowKind::Combo);

        assert_eq!(read, None);
    }

    fn component(x: usize, width: usize, y: usize, height: usize, pixels: usize) -> Component {
        Component {
            min_x: x,
            min_y: y,
            max_x: x + width - 1,
            max_y: y + height - 1,
            pixels,
        }
    }

    fn digit(x: usize, value: u32) -> DigitCandidate {
        DigitCandidate {
            x,
            end_x: x + 8,
            digit: value,
            score: 5,
        }
    }

    #[test]
    fn attribute_templates_are_separated_with_large_margin() {
        for (expected, template) in ATTRIBUTE_TEMPLATES {
            let mut scores = ATTRIBUTE_TEMPLATES
                .iter()
                .map(|(attribute, candidate)| {
                    (*attribute, attribute_distance(&template, candidate))
                })
                .collect::<Vec<_>>();
            scores.sort_by_key(|(_, score)| *score);
            assert_eq!(scores[0], (expected, 0));
            assert!(
                scores[1].1 >= 40,
                "{expected:?} margin was only {}",
                scores[1].1
            );
        }
    }

    #[test]
    fn parentheses_anchor_both_numeric_groups() {
        let components = [
            component(40, 4, 2, 20, 30),
            component(72, 10, 3, 17, 35),
            component(85, 4, 2, 20, 30),
        ];
        let candidates = [digit(10, 1), digit(21, 6), digit(48, 5), digit(59, 5)];
        assert_eq!(
            parse_anchored_row(&components, &candidates, NumericRowKind::Damage),
            Some((16, 55, 20))
        );

        let combo_candidates = [
            digit(10, 1),
            digit(21, 6),
            digit(48, 2),
            digit(59, 6),
            digit(70, 6),
            // 括弧外の背景は数字に見えても最大値へ混ぜない。
            digit(92, 1),
        ];
        assert_eq!(
            parse_anchored_row(&components, &combo_candidates, NumericRowKind::Combo),
            Some((16, 266, 25))
        );
    }

    #[test]
    fn missing_edge_digits_are_not_shorter_valid_numbers() {
        let components = [
            component(40, 4, 2, 20, 30),
            component(72, 10, 3, 17, 35),
            component(85, 4, 2, 20, 30),
        ];
        assert_eq!(
            parse_anchored_row(
                &components,
                &[digit(10, 1), digit(48, 5), digit(59, 5)],
                NumericRowKind::Damage,
            ),
            None
        );
        assert_eq!(
            parse_anchored_row(
                &components,
                &[digit(10, 1), digit(21, 6), digit(48, 5)],
                NumericRowKind::Damage,
            ),
            None
        );
    }

    #[test]
    fn numeric_relationships_reject_partial_animation_reads() {
        assert!(numeric_values_are_plausible(480, 80, 1152, 2660));
        assert!(numeric_values_are_plausible(6000, 100, 11_000, 11_000));
        assert!(!numeric_values_are_plausible(480, 80, 115, 2660));
        assert!(!numeric_values_are_plausible(0, 80, 1152, 2660));
        assert!(!numeric_values_are_plausible(600, 101, 600, 600));
        assert!(!numeric_values_are_plausible(600, 100, 600, 20_001));
    }
}
