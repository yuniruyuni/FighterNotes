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
const METER_STRIP_BYTES: usize = BASE_WIDTH * NUMERIC_HEIGHT * 4;

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
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
    if width as usize != BASE_WIDTH {
        return empty_inspection();
    }
    if rgba.len() < METER_STRIP_BYTES {
        return empty_inspection();
    }
    read_frame(
        rgba,
        width as usize,
        NUMERIC_HEIGHT,
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
    if !window_fits(
        numeric_origin,
        (NUMERIC_WIDTH, NUMERIC_HEIGHT),
        width,
        height,
    ) {
        return None;
    }
    if !window_fits(
        attribute_origin,
        (ATTRIBUTE_WIDTH, ATTRIBUTE_HEIGHT),
        width,
        height,
    ) {
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

fn window_fits(origin: (usize, usize), size: (usize, usize), width: usize, height: usize) -> bool {
    width
        .checked_sub(size.0)
        .is_some_and(|last_x| origin.0 <= last_x)
        && height
            .checked_sub(size.1)
            .is_some_and(|last_y| origin.1 <= last_y)
}

fn read_numeric(rgba: &[u8], width: usize, origin: (usize, usize)) -> Option<NumericRead> {
    [210, 180].into_iter().find_map(|threshold| {
        let damage = read_numeric_row(
            rgba,
            width,
            origin,
            0,
            25,
            threshold,
            NumericRowKind::Damage,
        )?;
        let combo = read_numeric_row(
            rgba,
            width,
            origin,
            27,
            NUMERIC_HEIGHT,
            threshold,
            NumericRowKind::Combo,
        )?;
        combine_numeric_rows(damage, combo)
    })
}

fn combine_numeric_rows(damage: (u32, u32, u32), combo: (u32, u32, u32)) -> Option<NumericRead> {
    numeric_values_are_plausible(damage.0, damage.1, combo.0, combo.1).then(|| NumericRead {
        last_damage: damage.0,
        scaling_percent: damage.1,
        combo_damage: combo.0,
        max_combo_damage: combo.1,
        score: damage.2.saturating_add(combo.2),
    })
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
    let mut white = [0u8; NUMERIC_WIDTH * ROW_SCAN_HEIGHT];
    for y in 0..row_height {
        for x in 0..NUMERIC_WIDTH {
            white[y * NUMERIC_WIDTH + x] = u8::from(
                gray(rgba, width, origin.0 + x, origin.1 + row_y1 + y)
                    .cmp(&threshold)
                    .is_gt(),
            );
        }
    }

    let components = connected_components(&white, row_height);

    let frame = Frame::new(rgba, width, 0, threshold);
    let mut candidates = Vec::new();
    for component in &components {
        if !looks_like_a_digit(component) {
            continue;
        }
        let x0 = (origin.0 + component.min_x).saturating_sub(1);
        let y0 = (origin.1 + row_y1 + component.min_y).saturating_sub(1);
        let (digit, score, margin) = match_digit_gray(&frame, x0, y0);
        if digit_is_confident(score, margin) {
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

fn connected_components(
    white: &[u8; NUMERIC_WIDTH * ROW_SCAN_HEIGHT],
    row_height: usize,
) -> Vec<Component> {
    const NEIGHBOURS: [(isize, isize); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    let mut visited = [false; NUMERIC_WIDTH * ROW_SCAN_HEIGHT];
    let mut components = Vec::new();
    for start in 0..row_height * NUMERIC_WIDTH {
        if white[start] == 0 || visited[start] {
            continue;
        }
        let mut component = Component {
            min_x: NUMERIC_WIDTH,
            max_x: 0,
            min_y: row_height,
            max_y: 0,
            pixels: 0,
        };
        let mut stack = vec![start];
        visited[start] = true;
        // A component can contain at most `white.len()` distinct pixels. The
        // bound also prevents corrupted visit bookkeeping from cycling.
        for _ in white {
            if let Some(current) = stack.pop() {
                let x = current % NUMERIC_WIDTH;
                let y = current / NUMERIC_WIDTH;
                component.min_x = component.min_x.min(x);
                component.max_x = component.max_x.max(x);
                component.min_y = component.min_y.min(y);
                component.max_y = component.max_y.max(y);
                component.pixels += 1;
                for (dx, dy) in NEIGHBOURS {
                    let (Some(nx), Some(ny)) = (x.checked_add_signed(dx), y.checked_add_signed(dy))
                    else {
                        continue;
                    };
                    if nx >= NUMERIC_WIDTH || ny >= row_height {
                        continue;
                    }
                    let next = ny * NUMERIC_WIDTH + nx;
                    if white[next] != 0 && !visited[next] {
                        visited[next] = true;
                        stack.push(next);
                    }
                }
            }
        }
        components.push(component);
    }
    components
}

/// 連結成分が数字の大きさか。
///
/// 数字の高さは 16〜18px。括弧は 19〜20px、`%` の各部は 9px 以下なので、
/// 高さで句読記号を先に落とすと、細い `1` も安全に残せる。
fn looks_like_a_digit(component: &Component) -> bool {
    let width = component.max_x - component.min_x + 1;
    let height = component.max_y - component.min_y + 1;
    (3..=11).contains(&width) && (14..=18).contains(&height) && component.pixels >= 12
}

/// テンプレート照合の結果を桁として採るか。
///
/// スコアが十分に良ければそのまま採る。少し悪くても、二番目と大きく
/// 離れていれば採る。いずれの場合も、二番目と紛れている桁は採らない。
fn digit_is_confident(score: u32, margin: u32) -> bool {
    const CLEAR_SCORE: u32 = 28;
    const MARGIN_SCORE: u32 = 40;
    const STRONG_MARGIN: u32 = 15;
    const AMBIGUOUS_MARGIN: u32 = 3;

    let matched = score <= CLEAR_SCORE || (score <= MARGIN_SCORE && margin >= STRONG_MARGIN);
    matched && margin >= AMBIGUOUS_MARGIN
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
    let mut remaining = parentheses.as_slice();
    while let Some((left, rights)) = remaining.split_first() {
        for right in rights {
            if let Some(read) = parse_anchored_pair(components, candidates, kind, left, right) {
                reads.push(read);
            }
        }
        remaining = rights;
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

fn parse_anchored_pair(
    components: &[Component],
    candidates: &[DigitCandidate],
    kind: NumericRowKind,
    left: &Component,
    right: &Component,
) -> Option<(u32, u32, u32)> {
    let inner_width = right.min_x.saturating_sub(left.max_x);
    if !(12..=62).contains(&inner_width) {
        return None;
    }
    let first_group = digits_before_parenthesis(candidates, left)?;
    let second_group = match kind {
        NumericRowKind::Damage => scaling_digits(components, candidates, left, right),
        NumericRowKind::Combo => digits_between_parentheses(candidates, left, right),
    }?;
    let (first, first_score) = parse_digit_group(first_group);
    let (second, second_score) = parse_digit_group(second_group);
    Some((first, second, first_score.saturating_add(second_score)))
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
        if !(1..=8).contains(&group[0].x.saturating_sub(left.max_x))
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
    for (threshold, max_score) in [(210u8, 90u32), (180u8, 100u32)] {
        let mut observed = [0u32; ATTRIBUTE_HEIGHT];
        for (y, row) in observed.iter_mut().enumerate() {
            *row = (0..ATTRIBUTE_WIDTH)
                .map(|x| {
                    u32::from(
                        gray(rgba, width, origin.0 + x, origin.1 + y)
                            .cmp(&threshold)
                            .is_gt(),
                    ) << x
                })
                .sum();
        }
        let white_count = observed.iter().map(|row| row.count_ones()).sum();
        if !attribute_white_count_is_plausible(white_count) {
            continue;
        }
        let mut scores = ATTRIBUTE_TEMPLATES
            .iter()
            .map(|(attribute, template)| (*attribute, attribute_distance(&observed, template)))
            .collect::<Vec<_>>();
        scores.sort_by_key(|(_, score)| *score);
        let (value, score) = scores[0];
        let margin = scores[1].1.saturating_sub(score);
        if attribute_match_is_confident(score, margin, max_score) {
            return Some(AttributeRead {
                value,
                score,
                margin,
            });
        }
    }
    None
}

fn attribute_white_count_is_plausible(count: u32) -> bool {
    (60..=280).contains(&count)
}

fn attribute_match_is_confident(score: u32, margin: u32, max_score: u32) -> bool {
    score <= max_score && margin >= 20
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
    let Some([red, green, blue]) = rgba.get(index..).and_then(|tail| tail.first_chunk::<3>())
    else {
        return 0;
    };
    (*red).min(*green).min(*blue)
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

    #[test]
    fn numeric_rows_are_combined_only_when_their_values_agree() {
        let read = combine_numeric_rows((77, 80, 11), (777, 999, 13)).expect("整合する");

        assert_eq!(read.last_damage, 77);
        assert_eq!(read.scaling_percent, 80);
        assert_eq!(read.combo_damage, 777);
        assert_eq!(read.max_combo_damage, 999);
        assert_eq!(read.score, 24);
        assert!(combine_numeric_rows((800, 80, 3), (700, 999, 5)).is_none());
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
        let origin = (300usize, 5usize);
        let row_y1 = 30usize;
        let mut rgba = vec![0u8; width * 80 * 4];

        // 走査窓を基準に配置する。数字はテンプレート左上、括弧は塗り潰し。
        let mut paint = |x: usize, y: usize, digit: usize| {
            paint_digit(&mut rgba, width, origin.0 + x, origin.1 + row_y1 + y, digit);
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
                    let index = ((origin.1 + row_y1 + y) * width + origin.0 + x) * 4;
                    rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        };
        bar(28);
        bar(68);

        let read = read_numeric_row(
            &rgba,
            width,
            origin,
            row_y1,
            row_y1 + 25,
            210,
            NumericRowKind::Combo,
        );

        let (first, second, score) = read.expect("括弧が揃った行は読める");
        assert_eq!((first, second), (77, 777));
        assert_eq!(score, 0, "テンプレートそのものなので誤差は出ない");

        assert_eq!(
            read_numeric_row(
                &rgba,
                width,
                origin,
                row_y1,
                row_y1 + ROW_SCAN_HEIGHT,
                210,
                NumericRowKind::Combo,
            ),
            Some((77, 777, 0)),
            "最大走査高は境界を含む"
        );
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

    #[test]
    fn connected_components_join_diagonals_and_respect_the_row_edges() {
        let mut white = [0u8; NUMERIC_WIDTH * ROW_SCAN_HEIGHT];
        for (x, y) in [
            (0, 0),
            (0, 1),
            (1, 1),
            (2, 2),
            (10, 0),
            (10, 1),
            (189, 0),
            (188, 1),
        ] {
            white[y * NUMERIC_WIDTH + x] = 1;
        }
        // 走査高の直後にある画素は別の行なので含めない。
        white[3 * NUMERIC_WIDTH + 2] = 1;

        assert_eq!(
            connected_components(&white, 3),
            vec![
                component(0, 3, 0, 3, 4),
                component(10, 1, 0, 2, 2),
                component(188, 2, 0, 2, 2),
            ]
        );
    }

    #[test]
    fn the_right_edge_does_not_wrap_into_the_next_row() {
        let mut white = [0u8; NUMERIC_WIDTH * ROW_SCAN_HEIGHT];
        white[NUMERIC_WIDTH - 1] = 1;
        white[NUMERIC_WIDTH] = 1;

        let components = connected_components(&white, 2);

        assert_eq!(components.len(), 2);
        assert_eq!(components[0], component(189, 1, 0, 1, 1));
        assert_eq!(components[1], component(0, 1, 1, 1, 1));
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
        assert!(numeric_values_are_plausible(600, 100, 600, 20_000));
        assert!(!numeric_values_are_plausible(480, 80, 115, 2660));
        assert!(!numeric_values_are_plausible(0, 80, 1152, 2660));
        assert!(!numeric_values_are_plausible(600, 101, 600, 600));
        assert!(!numeric_values_are_plausible(600, 100, 600, 20_001));
    }
    // ── 形からの絞り込み ─────────────────────────────────────────────────

    /// 数字の大きさ。画面の中央表示は解像度が決まっているので、数字の
    /// 幅と高さもほぼ決まる。ここを緩めると `%` の丸や括弧まで桁になる。
    #[test]
    fn only_blobs_the_size_of_a_digit_are_considered() {
        assert!(looks_like_a_digit(&component(0, 5, 0, 16, 30)));

        assert!(!looks_like_a_digit(&component(0, 2, 0, 16, 30)), "細すぎる");
        assert!(
            !looks_like_a_digit(&component(0, 12, 0, 16, 30)),
            "太すぎる"
        );
        assert!(!looks_like_a_digit(&component(0, 5, 0, 13, 30)), "低すぎる");
        assert!(!looks_like_a_digit(&component(0, 5, 0, 19, 30)), "高すぎる");
        assert!(!looks_like_a_digit(&component(0, 5, 0, 16, 11)), "薄すぎる");
    }

    /// 大きさの境目はちょうどの値まで含める。
    #[test]
    fn the_size_limits_include_their_own_edges() {
        assert!(looks_like_a_digit(&component(0, 3, 0, 14, 12)));
        assert!(looks_like_a_digit(&component(0, 11, 0, 18, 12)));
    }

    /// 括弧は数字より細く、少し高い。
    #[test]
    fn a_parenthesis_is_thinner_and_taller_than_a_digit() {
        assert!(is_parenthesis(&&component(0, 4, 0, 20, 30)));

        assert!(!is_parenthesis(&&component(0, 1, 0, 20, 30)), "細すぎる");
        assert!(!is_parenthesis(&&component(0, 7, 0, 20, 30)), "太すぎる");
        assert!(!is_parenthesis(&&component(0, 4, 0, 17, 30)), "低すぎる");
        assert!(!is_parenthesis(&&component(0, 4, 0, 22, 30)), "高すぎる");
        assert!(!is_parenthesis(&&component(0, 4, 0, 20, 13)), "薄すぎる");
    }

    #[test]
    fn parenthesis_size_limits_include_every_edge() {
        assert!(is_parenthesis(&&component(0, 2, 0, 18, 14)));
        assert!(is_parenthesis(&&component(0, 6, 0, 21, 14)));
    }

    /// 細くて高さ 18px の塊は、数字にも括弧にも見える。どちらとして
    /// 扱うかは、桁の絞り込みと括弧の探索が別々に判断する。ここが
    /// 重なっていること自体は、読み取りの前提として知っておきたい。
    #[test]
    fn a_thin_eighteen_pixel_blob_looks_like_both() {
        let ambiguous = component(0, 4, 0, 18, 30);

        assert!(looks_like_a_digit(&ambiguous));
        assert!(is_parenthesis(&&ambiguous));
    }

    /// 重なるのはその 1px だけ。17px 以下は数字、19px 以上は括弧。
    #[test]
    fn outside_that_one_height_the_two_shapes_do_not_overlap() {
        for width in 1..=13 {
            for height in (12..=17).chain(19..=23) {
                let blob = component(0, width, 0, height, 40);
                assert!(
                    !(looks_like_a_digit(&blob) && is_parenthesis(&&blob)),
                    "{width}x{height} が数字にも括弧にも見える"
                );
            }
        }
    }

    // ── 照合結果を採るかどうか ───────────────────────────────────────────

    /// 十分に一致していれば採る。
    #[test]
    fn a_clear_match_is_taken() {
        assert!(digit_is_confident(28, 3));
        assert!(!digit_is_confident(29, 3), "一致が悪いのに採っている");
    }

    /// 一致が少し悪くても、二番目と大きく離れていれば採る。
    #[test]
    fn a_weaker_match_is_taken_when_it_stands_well_clear() {
        assert!(digit_is_confident(40, 15));
        assert!(!digit_is_confident(41, 15), "悪すぎる一致まで採っている");
        assert!(!digit_is_confident(40, 14), "差が足りないのに採っている");
    }

    /// 二番目と紛れている桁は、どれだけ一致が良くても採らない。
    #[test]
    fn a_digit_tangled_with_the_runner_up_is_never_taken() {
        assert!(digit_is_confident(0, 3));
        assert!(!digit_is_confident(0, 2), "紛れている桁を採っている");
    }

    // ── 桁の並び ─────────────────────────────────────────────────────────

    /// 桁は 1 個から 5 個まで。
    #[test]
    fn a_group_holds_between_one_and_five_digits() {
        let digits: Vec<DigitCandidate> = (0..6).map(|k| digit(k * 11, 1)).collect();

        assert!(!valid_group(&[]), "空の並びを桁として認めている");
        assert!(valid_group(&digits[..1]));
        assert!(valid_group(&digits[..5]));
        assert!(!valid_group(&digits[..6]), "6 桁まで認めている");
    }

    /// 隣り合う桁の間隔には決まった幅がある。空きすぎていれば別の数。
    #[test]
    fn digits_of_one_number_sit_close_together() {
        let pair = |gap: usize| [digit(0, 1), digit(8 + gap, 2)];

        assert!(digits_are_contiguous(&pair(1)));
        assert!(digits_are_contiguous(&pair(8)));
        assert!(
            !digits_are_contiguous(&pair(0)),
            "重なった桁を並びにしている"
        );
        assert!(!digits_are_contiguous(&pair(9)), "離れた桁を並びにしている");
    }

    /// 桁の並びは左から順に十進で組み立てる。
    #[test]
    fn a_group_of_digits_becomes_a_decimal_number() {
        let group = [digit(0, 1), digit(11, 2), digit(22, 3)];

        assert_eq!(parse_digit_group(&group), (123, 15));
        assert_eq!(parse_digit_group(&[]), (0, 0));
    }

    // ── `%` の裏付け ─────────────────────────────────────────────────────

    /// 補正値の右には `%` がある。桁と閉じ括弧の間に何も無ければ、
    /// それは補正値ではない。
    #[test]
    fn the_scaling_digits_must_be_followed_by_a_percent_sign() {
        let percent = [component(20, 4, 0, 4, 12), component(26, 4, 6, 4, 12)];

        assert!(has_percent_evidence(&percent, 15, 32));
        assert!(!has_percent_evidence(&[], 15, 32), "何も無いのに認めている");
    }

    /// `%` は閉じ括弧のすぐ手前にある。遠く離れた模様は `%` ではない。
    #[test]
    fn a_mark_far_from_the_closing_parenthesis_is_not_a_percent_sign() {
        let near = [component(20, 4, 0, 4, 24)];

        assert!(has_percent_evidence(&near, 15, 31));
        assert!(
            !has_percent_evidence(&near, 15, 32),
            "離れた模様を認めている"
        );
    }

    /// 点が薄すぎれば `%` とは言えない。
    #[test]
    fn a_faint_mark_is_not_enough_for_a_percent_sign() {
        assert!(has_percent_evidence(&[component(20, 4, 0, 4, 20)], 15, 27));
        assert!(!has_percent_evidence(&[component(20, 4, 0, 4, 19)], 15, 27));
    }

    /// 括弧そのものは `%` の証拠に数えない。
    #[test]
    fn the_parentheses_themselves_are_not_percent_evidence() {
        let parenthesis = [component(20, 4, 0, 20, 30)];

        assert!(!has_percent_evidence(&parenthesis, 15, 32));
    }

    // ── 画素の読み ───────────────────────────────────────────────────────

    /// 明るさは 3 チャンネルの最小値。どれか一つでも暗ければ暗い。
    #[test]
    fn the_brightness_of_a_pixel_is_its_darkest_channel() {
        let rgba = [200u8, 40, 255, 255];

        assert_eq!(gray(&rgba, 1, 0, 0), 40);
    }

    /// 画面の外は黒として読む。範囲外で落ちない。
    #[test]
    fn a_pixel_outside_the_frame_reads_as_black() {
        let rgba = vec![255u8; 8];

        assert_eq!(gray(&rgba, 1, 0, 0), 255);
        assert_eq!(gray(&rgba, 1, 0, 1), 255);
        assert_eq!(gray(&rgba, 1, 0, 2), 0, "画面の外を明るいと読んでいる");
        assert_eq!(gray(&rgba, 1, 5, 5), 0);
        assert_eq!(gray(&[255, 255], 1, 0, 0), 0, "RGB が揃わない画素");
    }

    // ── 属性の読み ───────────────────────────────────────────────────────

    /// 属性の手本を描けば、その属性として読み返せる。
    #[test]
    fn every_attribute_glyph_reads_back_from_itself() {
        for (expected, template) in ATTRIBUTE_TEMPLATES {
            let width = BASE_WIDTH;
            let mut rgba = vec![0u8; width * 64 * 4];
            for (y, bits) in template.iter().enumerate() {
                for x in 0..ATTRIBUTE_WIDTH {
                    if bits & (1 << x) != 0 {
                        let index = ((y + 5) * width + 100 + x) * 4;
                        rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
                    }
                }
            }

            let read = read_attribute(&rgba, width, (100, 5)).expect("手本は読める");

            assert_eq!(read.value, expected);
            assert_eq!(read.score, 0);
            assert!(read.margin >= 20);
        }
    }

    /// 白がほとんど無い枠は属性ではない。表示が出ていないだけ。
    #[test]
    fn an_almost_blank_attribute_box_is_not_read() {
        let width = BASE_WIDTH;
        let rgba = vec![0u8; width * 64 * 4];

        assert!(read_attribute(&rgba, width, (100, 5)).is_none());
    }

    /// 一面が白い枠も属性ではない。演出の白飛び。
    #[test]
    fn a_washed_out_attribute_box_is_not_read() {
        let width = BASE_WIDTH;
        let rgba = vec![255u8; width * 64 * 4];

        assert!(read_attribute(&rgba, width, (100, 5)).is_none());
    }

    // ── 走査窓 ───────────────────────────────────────────────────────────

    /// 走査窓が画面からはみ出す位置では何も読まない。
    #[test]
    fn a_side_whose_windows_fall_outside_the_frame_is_not_read() {
        let width = BASE_WIDTH;
        let rgba = vec![0u8; width * 64 * 4];

        // 数値の窓が右へはみ出す。
        assert!(read_side(&rgba, width, 64, (width - 1, 0), (100, 0)).is_none());
        // 数値の窓が下へはみ出す。
        assert!(read_side(&rgba, width, 64, (100, 63), (100, 0)).is_none());
        // 属性の窓が右へはみ出す。
        assert!(read_side(&rgba, width, 64, (100, 0), (width - 1, 0)).is_none());
        // 属性の窓が下へはみ出す。
        assert!(read_side(&rgba, width, 64, (100, 0), (100, 63)).is_none());
    }

    #[test]
    fn window_fit_checks_each_axis_and_includes_the_exact_edge() {
        assert!(window_fits((10, 20), (30, 40), 40, 60));
        assert!(!window_fits((11, 20), (30, 40), 40, 60));
        assert!(!window_fits((10, 21), (30, 40), 40, 60));
        assert!(!window_fits((0, 0), (41, 40), 40, 60));
        assert!(!window_fits((0, 0), (30, 61), 40, 60));
    }

    /// 想定と違う幅の映像は読まない。中央表示の位置は 1920px 基準で
    /// 決め打ちしているので、幅が違えば別の場所を見ることになる。
    #[test]
    fn a_frame_of_the_wrong_width_is_not_read() {
        let rgba = vec![0u8; 1280 * 720 * 4];
        let inspection = read_attack_info(&rgba, 1280, 720);

        assert!(inspection.p1.is_none());
        assert!(inspection.p2.is_none());
    }

    /// 画素が足りない映像も読まない。
    #[test]
    fn a_frame_shorter_than_it_claims_is_not_read() {
        let rgba = vec![0u8; BASE_WIDTH * 4];

        assert!(read_attack_info(&rgba, BASE_WIDTH as u32, 1080)
            .p1
            .is_none());
    }
    // ── 片側をまるごと読む ───────────────────────────────────────────────

    /// 中央表示の片側をひとそろい描く。
    ///
    /// 上の行は「与ダメージ(補正%)」、下の行は「コンボダメージ(最大)」。
    /// 数字はテンプレートの縁が 1px 内側に収まる `7` だけを使う（他の桁は
    /// 外接矩形の取り方で 1px ずれる）。括弧は塗り潰しの縦棒で代用する。
    fn paint_side(
        rgba: &mut [u8],
        width: usize,
        numeric: (usize, usize),
        attribute: (usize, usize),
    ) {
        use input_vision::test_support::paint_digit;

        let mut bar = |x0: usize, y0: usize| {
            for y in y0 + 2..=y0 + 20 {
                for x in x0..x0 + 3 {
                    let index = (y * width + x) * 4;
                    rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        };
        // 上の行: 77(77%)
        bar(numeric.0 + 28, numeric.1);
        bar(numeric.0 + 70, numeric.1);
        // 下の行: 77(777)
        bar(numeric.0 + 28, numeric.1 + 27);
        bar(numeric.0 + 68, numeric.1 + 27);

        for (x, y) in [
            (0, 2),
            (11, 2),
            (34, 2),
            (45, 2),
            (0, 29),
            (11, 29),
            (34, 29),
            (45, 29),
            (56, 29),
        ] {
            paint_digit(rgba, width, numeric.0 + x, numeric.1 + y, 7);
        }

        // 補正値の右の `%`。二つの小さな塊で、閉じ括弧のすぐ手前に置く。
        for (x0, y0) in [(56usize, 4usize), (63, 12)] {
            for y in y0..y0 + 4 {
                for x in x0..x0 + 5 {
                    let index = ((numeric.1 + y) * width + numeric.0 + x) * 4;
                    rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        }

        // 攻撃属性のグリフ。
        let (_, template) = ATTRIBUTE_TEMPLATES[0];
        for (y, bits) in template.iter().enumerate() {
            for x in 0..ATTRIBUTE_WIDTH {
                if bits & (1 << x) != 0 {
                    let index = ((attribute.1 + y) * width + attribute.0 + x) * 4;
                    rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        }
    }

    fn painted_frame(numeric: (usize, usize), attribute: (usize, usize)) -> Vec<u8> {
        let mut rgba = vec![0u8; BASE_WIDTH * 1080 * 4];
        paint_side(&mut rgba, BASE_WIDTH, numeric, attribute);
        rgba
    }

    /// ひとそろい描けば、数値も属性も読める。
    #[test]
    fn a_fully_painted_side_reads_both_rows_and_the_attribute() {
        let rgba = painted_frame((100, 10), (400, 10));

        let read = read_side(&rgba, BASE_WIDTH, 1080, (100, 10), (400, 10)).expect("読める");

        assert_eq!(read.value.last_damage, 77);
        assert_eq!(read.value.scaling_percent, 77);
        assert_eq!(read.value.combo_damage, 77);
        assert_eq!(read.value.max_combo_damage, 777);
        assert_eq!(read.value.attribute, ATTRIBUTE_TEMPLATES[0].0);
        assert_eq!(read.numeric_score, 0, "手本そのものなので誤差は出ない");
    }

    #[test]
    fn dim_numeric_rows_are_retried_with_the_lower_threshold() {
        let numeric = (100, 10);
        let attribute = (400, 10);
        let mut rgba = painted_frame(numeric, attribute);
        for y in numeric.1..numeric.1 + NUMERIC_HEIGHT {
            for x in numeric.0..numeric.0 + NUMERIC_WIDTH {
                let index = (y * BASE_WIDTH + x) * 4;
                if rgba[index] == 255 {
                    rgba[index..index + 3].fill(200);
                }
            }
        }

        let read = read_side(&rgba, BASE_WIDTH, 1080, numeric, attribute).expect("暗くても読める");

        assert_eq!(read.value.last_damage, 77);
        assert_eq!(read.value.combo_damage, 77);
    }

    /// 走査窓が画面からはみ出す位置では読まない。読める絵が描いてあっても、
    /// 窓の外を読みに行かない。
    #[test]
    fn each_window_must_fit_inside_the_frame() {
        let rgba = painted_frame((100, 10), (400, 10));
        let read = |numeric, attribute| read_side(&rgba, BASE_WIDTH, 1080, numeric, attribute);

        assert!(read((100, 10), (400, 10)).is_some(), "収まっていれば読める");
        assert!(
            read((BASE_WIDTH - NUMERIC_WIDTH + 1, 10), (400, 10)).is_none(),
            "数値の窓が右へはみ出している"
        );
        assert!(
            read((100, 1080 - NUMERIC_HEIGHT + 1), (400, 10)).is_none(),
            "数値の窓が下へはみ出している"
        );
        assert!(
            read((100, 10), (BASE_WIDTH - ATTRIBUTE_WIDTH + 1, 10)).is_none(),
            "属性の窓が右へはみ出している"
        );
        assert!(
            read((100, 10), (400, 1080 - ATTRIBUTE_HEIGHT + 1)).is_none(),
            "属性の窓が下へはみ出している"
        );
    }

    /// 窓がぴったり収まる位置は読む。1px の余裕を取り違えない。
    #[test]
    fn a_window_that_fits_exactly_is_still_read() {
        let numeric = (BASE_WIDTH - NUMERIC_WIDTH, 1080 - NUMERIC_HEIGHT);
        let attribute = (0, 0);
        let mut rgba = vec![0u8; BASE_WIDTH * 1080 * 4];
        paint_side(&mut rgba, BASE_WIDTH, numeric, attribute);

        assert!(read_side(&rgba, BASE_WIDTH, 1080, numeric, attribute).is_some());
    }

    /// 属性が読めなければ、数値が読めても片側全体を諦める。どちらか
    /// 片方だけの読みは、後段が使えない。
    #[test]
    fn a_side_without_a_readable_attribute_is_dropped() {
        let mut rgba = painted_frame((100, 10), (400, 10));
        for y in 10..30 {
            for x in 400..432 {
                let index = (y * BASE_WIDTH + x) * 4;
                rgba[index..index + 4].copy_from_slice(&[0, 0, 0, 255]);
            }
        }

        assert!(read_side(&rgba, BASE_WIDTH, 1080, (100, 10), (400, 10)).is_none());
    }

    /// 数値が読めなければ、属性が読めても片側全体を諦める。
    #[test]
    fn a_side_without_readable_numbers_is_dropped() {
        let mut rgba = painted_frame((100, 10), (400, 10));
        for y in 10..66 {
            for x in 100..290 {
                let index = (y * BASE_WIDTH + x) * 4;
                rgba[index..index + 4].copy_from_slice(&[0, 0, 0, 255]);
            }
        }

        assert!(read_side(&rgba, BASE_WIDTH, 1080, (100, 10), (400, 10)).is_none());
    }

    // ── 二つの入口 ───────────────────────────────────────────────────────

    /// フル画面の入口は、決め打ちの位置から両側を読む。
    #[test]
    fn the_full_frame_entry_point_reads_both_sides_from_their_fixed_places() {
        let mut rgba = vec![0u8; BASE_WIDTH * 1080 * 4];
        paint_side(
            &mut rgba,
            BASE_WIDTH,
            P1_NUMERIC_SOURCE,
            P1_ATTRIBUTE_SOURCE,
        );
        paint_side(
            &mut rgba,
            BASE_WIDTH,
            P2_NUMERIC_SOURCE,
            P2_ATTRIBUTE_SOURCE,
        );

        let inspection = read_attack_info(&rgba, BASE_WIDTH as u32, 1080);

        assert_eq!(
            inspection.p1.map(|side| side.value.max_combo_damage),
            Some(777)
        );
        assert_eq!(
            inspection.p2.map(|side| side.value.max_combo_damage),
            Some(777)
        );
        assert_eq!(inspection.rois, FULL_ROIS, "ROI は常に元画面の座標で返す");
    }

    /// 切り出した帯の入口は、詰めた位置から読む。フル画面の座標を
    /// そのまま使うと、帯の外を読みに行く。
    #[test]
    fn the_packed_strip_entry_point_reads_from_the_packed_places() {
        let height = NUMERIC_HEIGHT;
        let mut rgba = vec![0u8; BASE_WIDTH * height * 4];
        paint_side(
            &mut rgba,
            BASE_WIDTH,
            P1_NUMERIC_PACKED,
            P1_ATTRIBUTE_PACKED,
        );
        paint_side(
            &mut rgba,
            BASE_WIDTH,
            P2_NUMERIC_PACKED,
            P2_ATTRIBUTE_PACKED,
        );

        let inspection = read_attack_info_from_meter_strip(&rgba, BASE_WIDTH as u32);

        assert_eq!(
            inspection.p1.map(|side| side.value.last_damage),
            Some(77),
            "詰めた位置から読めていない"
        );
        assert_eq!(inspection.p2.map(|side| side.value.last_damage), Some(77));
    }

    /// 帯が数値行の高さに足りなければ読まない。
    #[test]
    fn a_strip_shorter_than_one_numeric_row_is_not_read() {
        let short = vec![0u8; BASE_WIDTH * (NUMERIC_HEIGHT - 1) * 4];

        let inspection = read_attack_info_from_meter_strip(&short, BASE_WIDTH as u32);

        assert!(inspection.p1.is_none());
        assert!(inspection.p2.is_none());
    }

    /// 想定と違う幅の帯も読まない。
    #[test]
    fn a_strip_of_the_wrong_width_is_not_read() {
        let strip = vec![0u8; 1280 * NUMERIC_HEIGHT * 4];

        assert!(read_attack_info_from_meter_strip(&strip, 1280).p1.is_none());
    }

    /// 高さの申告に対して画素が足りない映像は読まない。読み込みが
    /// 途中で切れた場面で、範囲外を読みに行かないための門。
    #[test]
    fn a_frame_with_fewer_pixels_than_declared_is_not_read() {
        let mut rgba = vec![0u8; BASE_WIDTH * 1080 * 4];
        paint_side(
            &mut rgba,
            BASE_WIDTH,
            P1_NUMERIC_SOURCE,
            P1_ATTRIBUTE_SOURCE,
        );

        assert!(read_attack_info(&rgba, BASE_WIDTH as u32, 1080)
            .p1
            .is_some());

        rgba.truncate(BASE_WIDTH * 1079 * 4);
        assert!(read_attack_info(&rgba, BASE_WIDTH as u32, 1080)
            .p1
            .is_none());
    }
    // ── 括弧を手がかりに桁を切り分ける ───────────────────────────────────

    /// 括弧の左の数は、括弧との間が決まった幅だけ空いている。
    #[test]
    fn the_number_left_of_a_parenthesis_sits_a_set_distance_away() {
        let parenthesis = component(40, 4, 2, 20, 30);
        let group = |end_gap: usize| {
            let x = 40 - end_gap - 8;
            digits_before_parenthesis(&[digit(x, 5)], &parenthesis).map(parse_digit_group)
        };

        assert_eq!(group(7), Some((5, 5)));
        assert_eq!(group(15), Some((5, 5)));
        assert_eq!(group(6), None, "近すぎる数まで拾っている");
        assert_eq!(group(16), None, "離れた数まで拾っている");
    }

    /// 左の数は、隣り合う桁が続く限りさかのぼって集める。
    #[test]
    fn the_number_left_of_a_parenthesis_gathers_its_touching_digits() {
        let parenthesis = component(50, 4, 2, 20, 30);
        // 末尾の桁は括弧から 9px。その左に 3px 間隔で桁が続く。
        let candidates = [digit(11, 1), digit(22, 2), digit(33, 3)];

        let group = digits_before_parenthesis(&candidates, &parenthesis).expect("読める");

        assert_eq!(parse_digit_group(group), (123, 15));

        let boundary = [digit(3, 1), digit(19, 2), digit(35, 3)];
        let group = digits_before_parenthesis(&boundary, &parenthesis).expect("境界も読める");
        assert_eq!(parse_digit_group(group), (123, 15));
    }

    /// 大きく離れた桁は別の数。さかのぼりはそこで止まる。
    #[test]
    fn the_gathering_stops_at_a_wide_gap() {
        let parenthesis = component(55, 4, 2, 20, 30);
        // 1 桁目と 2 桁目の間が 9px 空いている。
        let candidates = [digit(10, 1), digit(27, 2), digit(38, 3)];

        let group = digits_before_parenthesis(&candidates, &parenthesis).expect("読める");

        assert_eq!(parse_digit_group(group), (23, 10), "離れた桁まで繋げている");
    }

    /// 括弧の左に桁が無ければ読めない。
    #[test]
    fn nothing_left_of_the_parenthesis_means_nothing_to_read() {
        let parenthesis = component(40, 4, 2, 20, 30);

        assert!(digits_before_parenthesis(&[], &parenthesis).is_none());
        assert!(digits_before_parenthesis(&[digit(60, 5)], &parenthesis).is_none());
    }

    /// 括弧に挟まれた数は、両側の括弧からすぐ近くに始まって終わる。
    #[test]
    fn the_number_between_parentheses_touches_both_of_them() {
        // 左の括弧は x=10..13、桁は 9px 幅。
        let left = component(10, 4, 2, 20, 30);
        let group = |x: usize, right_x: usize| {
            let right = component(right_x, 4, 2, 20, 30);
            digits_between_parentheses(&[digit(x, 5)], &left, &right).map(parse_digit_group)
        };

        assert_eq!(group(14, 23), Some((5, 5)), "両側にすぐ接している");
        assert_eq!(group(14, 30), Some((5, 5)), "右へ 8px までは読む");
        assert_eq!(group(21, 30), Some((5, 5)), "左から 8px までは読む");
        assert_eq!(group(13, 23), None, "左の括弧に食い込んでいる");
        assert_eq!(group(22, 31), None, "左の括弧から離れすぎている");
        assert_eq!(group(14, 22), None, "右の括弧に食い込んでいる");
        assert_eq!(group(14, 31), None, "右の括弧から離れすぎている");
    }

    /// 挟まれた数の桁同士も隣り合っている。
    #[test]
    fn the_digits_between_parentheses_must_touch_each_other() {
        let left = component(10, 4, 2, 20, 30);
        let right = component(50, 4, 2, 20, 30);

        assert!(digits_between_parentheses(
            &[digit(15, 1), digit(26, 2), digit(37, 3)],
            &left,
            &right
        )
        .is_some());
        assert!(
            digits_between_parentheses(&[digit(15, 1), digit(36, 2), digit(47, 3)], &left, &right)
                .is_none(),
            "離れた桁を一つの数にしている"
        );
    }

    /// 補正値は括弧の左端から始まり、`%` を挟んで閉じ括弧へ届く。
    #[test]
    fn the_scaling_number_runs_from_the_open_parenthesis_to_the_percent_sign() {
        let left = component(10, 4, 2, 20, 30);
        let right = component(50, 4, 2, 20, 30);
        let percent = [
            left,
            right,
            component(38, 5, 2, 4, 12),
            component(44, 5, 8, 4, 12),
        ];

        let candidates = [digit(15, 8), digit(26, 0)];
        let group = scaling_digits(&percent, &candidates, &left, &right).expect("読める");

        assert_eq!(parse_digit_group(group), (80, 10));
    }

    #[test]
    fn a_one_digit_scaling_value_can_touch_both_edges() {
        let left = component(10, 4, 2, 20, 30);
        let right = component(45, 4, 2, 20, 30);
        let components = [left, right, component(33, 5, 2, 4, 20)];
        for x in [14, 21] {
            let candidates = [digit(x, 8)];
            let group = scaling_digits(&components, &candidates, &left, &right).expect("読める");
            assert_eq!(parse_digit_group(group), (8, 5));
        }

        assert!(
            scaling_digits(&components, &[digit(22, 8)], &left, &right).is_none(),
            "左括弧から9px離れた桁を採っている"
        );
    }

    #[test]
    fn percent_evidence_must_begin_after_the_last_digit() {
        let crossing_mark = [component(10, 12, 0, 4, 20)];

        assert!(!has_percent_evidence(&crossing_mark, 15, 27));
    }

    /// `%` の証拠が無ければ補正値ではない。括弧の中の数を何でも
    /// 補正値として読むと、コンボダメージが補正値の欄へ流れ込む。
    #[test]
    fn a_number_without_a_percent_sign_is_not_a_scaling_value() {
        let left = component(10, 4, 2, 20, 30);
        let right = component(50, 4, 2, 20, 30);
        let candidates = [digit(15, 8), digit(26, 0)];

        assert!(scaling_digits(&[left, right], &candidates, &left, &right).is_none());
    }

    /// 補正値の候補が二通り取れるときは読まない。どちらが正しいか
    /// 決められない。
    #[test]
    fn an_ambiguous_scaling_read_is_dropped() {
        let left = component(10, 4, 2, 20, 30);
        let right = component(48, 4, 2, 20, 30);
        // 1 桁で読んでも 2 桁で読んでも `%` の証拠が付いてしまう並び。
        let components = [
            left,
            right,
            component(36, 5, 2, 4, 12),
            component(42, 5, 2, 4, 20),
        ];
        let candidates = [digit(15, 8), digit(26, 0)];

        assert!(scaling_digits(&components, &candidates, &left, &right).is_none());
    }

    /// 括弧の組が複数取れて、読みが食い違えば読まない。
    #[test]
    fn parenthesis_pairs_that_disagree_are_dropped() {
        let components = [
            component(40, 4, 2, 20, 30),
            component(90, 4, 2, 20, 30),
            component(113, 4, 2, 20, 30),
        ];
        // 左の 2 つの括弧では (16, 559)、右の 2 つでは (559, 1) と読める。
        let candidates = [
            digit(10, 1),
            digit(21, 6),
            digit(48, 5),
            digit(59, 5),
            digit(75, 9),
            digit(100, 1),
        ];

        assert_eq!(
            parse_anchored_row(&components, &candidates, NumericRowKind::Combo),
            None,
            "食い違う読みのどちらかを採っている"
        );
    }

    /// 括弧が一つしか無ければ、挟む場所が決まらない。
    #[test]
    fn a_single_parenthesis_anchors_nothing() {
        let components = [component(40, 4, 2, 20, 30)];

        assert_eq!(
            parse_anchored_row(
                &components,
                &[digit(10, 1), digit(21, 6)],
                NumericRowKind::Combo
            ),
            None
        );
    }

    /// 括弧が近すぎる組は、間に数が入らないので使わない。
    #[test]
    fn parentheses_too_close_together_are_not_a_pair() {
        let row = |right_x: usize| {
            let components = [
                component(40, 4, 2, 20, 30),
                component(right_x, 4, 2, 20, 30),
            ];
            let candidates = [digit(10, 1), digit(21, 6), digit(45, 5)];
            parse_anchored_row(&components, &candidates, NumericRowKind::Combo).is_some()
        };

        assert!(row(55), "ちょうどの間隔を落としている");
        assert!(!row(54), "狭すぎる組を使っている");
    }

    #[test]
    fn the_largest_parenthesis_pair_span_is_included() {
        let left = component(40, 4, 2, 20, 30);
        let right = component(105, 4, 2, 20, 30);
        let components = [left, right, component(96, 5, 2, 4, 20)];
        let candidates = [digit(25, 1), digit(51, 8), digit(67, 0), digit(83, 0)];

        assert_eq!(
            parse_anchored_pair(
                &components,
                &candidates,
                NumericRowKind::Damage,
                &left,
                &right,
            ),
            Some((1, 800, 20))
        );
    }

    // ── 属性を読む門 ─────────────────────────────────────────────────────

    /// 属性の枠に散らばる白の量には上下の限りがある。少なすぎれば
    /// 表示が無く、多すぎれば演出の白飛び。
    #[test]
    fn the_amount_of_white_in_the_attribute_box_has_both_limits() {
        let read_with_white = |count: usize| {
            let width = BASE_WIDTH;
            let mut rgba = vec![0u8; width * 64 * 4];
            for index in 0..count {
                let x = index % ATTRIBUTE_WIDTH;
                let y = index / ATTRIBUTE_WIDTH;
                let offset = ((5 + y) * width + 100 + x) * 4;
                rgba[offset..offset + 4].copy_from_slice(&[255, 255, 255, 255]);
            }
            read_attribute(&rgba, width, (100, 5))
        };

        assert!(
            read_with_white(59).is_none(),
            "白が少なすぎる枠を読んでいる"
        );
        assert!(read_with_white(281).is_none(), "白が多すぎる枠を読んでいる");

        assert!(attribute_white_count_is_plausible(60));
        assert!(attribute_white_count_is_plausible(280));
        assert!(!attribute_white_count_is_plausible(59));
        assert!(!attribute_white_count_is_plausible(281));
        assert!(attribute_match_is_confident(90, 20, 90));
        assert!(!attribute_match_is_confident(91, 20, 90));
        assert!(!attribute_match_is_confident(90, 19, 90));
    }

    #[test]
    fn pixels_exactly_at_the_dim_threshold_are_still_dark() {
        let width = BASE_WIDTH;
        let (_, template) = ATTRIBUTE_TEMPLATES[0];
        let paint_at = |brightness: u8| {
            let mut rgba = vec![0u8; width * 64 * 4];
            for (y, bits) in template.iter().enumerate() {
                for x in 0..ATTRIBUTE_WIDTH {
                    if bits & (1 << x) != 0 {
                        let index = ((y + 5) * width + 100 + x) * 4;
                        rgba[index..index + 4]
                            .copy_from_slice(&[brightness, brightness, brightness, 255]);
                    }
                }
            }
            read_attribute(&rgba, width, (100, 5))
        };

        assert!(paint_at(180).is_none());
        assert!(paint_at(181).is_some());
    }

    /// 二番目の候補と紛れている形は読まない。
    #[test]
    fn an_attribute_tangled_with_the_runner_up_is_not_read() {
        let width = BASE_WIDTH;
        let mut rgba = vec![0u8; width * 64 * 4];
        // 二つの手本を重ねる。どちらとも決まらない。
        for (_, template) in ATTRIBUTE_TEMPLATES.iter().take(2) {
            for (y, bits) in template.iter().enumerate() {
                for x in 0..ATTRIBUTE_WIDTH {
                    if bits & (1 << x) != 0 {
                        let index = ((y + 5) * width + 100 + x) * 4;
                        rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
                    }
                }
            }
        }

        assert!(read_attribute(&rgba, width, (100, 5)).is_none());
    }

    /// 暗い表示は、閾値を下げて読み直す。画面暗転中もグリフは残る。
    #[test]
    fn a_dim_attribute_glyph_is_read_on_the_second_pass() {
        let width = BASE_WIDTH;
        let (expected, template) = ATTRIBUTE_TEMPLATES[0];
        let mut rgba = vec![0u8; width * 64 * 4];
        for (y, bits) in template.iter().enumerate() {
            for x in 0..ATTRIBUTE_WIDTH {
                if bits & (1 << x) != 0 {
                    let index = ((y + 5) * width + 100 + x) * 4;
                    // 210 は超えないが 180 は超える明るさ。
                    rgba[index..index + 4].copy_from_slice(&[200, 200, 200, 255]);
                }
            }
        }

        let read = read_attribute(&rgba, width, (100, 5)).expect("暗くても読める");

        assert_eq!(read.value, expected);
    }
}
