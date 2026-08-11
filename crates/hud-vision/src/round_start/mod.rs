//! SF6 のラウンド開始時に中央へ表示される `FIGHT` 画像の検出。
//!
//! browser pipeline は固定位置の中央 ROI を 128x52 へ縮小し、HUD strip 内で
//! 既存解析が使わない中央領域へ格納する。ここでは色そのものではなく、実動画の
//! 複数ステージから集計した平均画像の強い輝度勾配だけを照合する。

use std::sync::OnceLock;

/// Browser が中央 ROI を縮小する幅。
pub const FIGHT_PATCH_WIDTH: usize = 128;
/// Browser が中央 ROI を縮小する高さ。
pub const FIGHT_PATCH_HEIGHT: usize = 52;
/// HUD strip 内に埋め込む左端。
pub const FIGHT_PATCH_X: usize = 896;
/// HUD strip 内に埋め込む上端。
pub const FIGHT_PATCH_Y: usize = 9;
/// 60fps 映像から画像照合する間隔。
pub const FIGHT_SAMPLE_INTERVAL: u32 = 4;

const TEMPLATE_SIZE: usize = FIGHT_PATCH_WIDTH * FIGHT_PATCH_HEIGHT;
const TEMPLATE_EDGE_THRESHOLD: i16 = 55;
const MAX_SHIFT_X: i16 = 2;
const MAX_SHIFT_Y: i16 = 1;
const FIGHT_SCORE_THRESHOLD: f32 = 0.45;
const FIGHT_PEAK_THRESHOLD: f32 = 0.60;
const FIGHT_MAX_HIT_GAP: u32 = 24;
const FIGHT_MIN_HITS: usize = 3;

// 実ゲームを撮影した複数ステージ・複数ラウンドから生成した平均輝度モデル。
// 元動画、frame、screenshot、crop は含まず、128x52 の画素平均だけを保持する。
static FIGHT_TEMPLATE: &[u8] = include_bytes!("fight_template.bin");

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FightObservation {
    pub frame: u32,
    pub score: f32,
}

/// ひとつのラウンド開始演出に対応する `FIGHT` の安定表示区間。
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FightMarker {
    /// 安定した `FIGHT` 画像を最初に確認した video frame。
    pub first_frame: u32,
    /// 安定表示を最後に確認した位置。戦闘解析の開始アンカーに使う。
    pub last_frame: u32,
    pub peak_frame: u32,
    pub peak_score: f32,
}

#[derive(Debug)]
struct TemplateEdge {
    x: i16,
    y: i16,
    gx: i16,
    gy: i16,
}

#[derive(Debug)]
struct FightTemplateModel {
    edges: Vec<TemplateEdge>,
    energy: f64,
}

/// HUD strip 中央へ埋め込まれた縮小 ROI と `FIGHT` 平均画像の輪郭相関。
///
/// 1px 程度の browser resize 差を吸収するため、狭い平行移動範囲で最大値を取る。
/// 入力不正・モデル不正時は検出側へ倒さず 0.0 を返す。
pub fn fight_score_from_hud_strip(hud_strip: &[u8], strip_width: usize) -> f32 {
    let Some(model) = fight_template_model() else {
        return 0.0;
    };
    let required = strip_width
        .checked_mul(FIGHT_PATCH_Y + FIGHT_PATCH_HEIGHT)
        .and_then(|pixels| pixels.checked_mul(4));
    if required.is_none_or(|required| hud_strip.len() < required)
        || FIGHT_PATCH_X + FIGHT_PATCH_WIDTH > strip_width
    {
        return 0.0;
    }

    let mut best = -1.0f32;
    for shift_y in -MAX_SHIFT_Y..=MAX_SHIFT_Y {
        for shift_x in -MAX_SHIFT_X..=MAX_SHIFT_X {
            let mut dot = 0.0f64;
            let mut sample_energy = 0.0f64;
            for edge in &model.edges {
                let source_x = edge.x.saturating_add(shift_x);
                let source_y = edge.y.saturating_add(shift_y);
                let gx = patch_luma(hud_strip, strip_width, source_x + 1, source_y)
                    - patch_luma(hud_strip, strip_width, source_x - 1, source_y);
                let gy = patch_luma(hud_strip, strip_width, source_x, source_y + 1)
                    - patch_luma(hud_strip, strip_width, source_x, source_y - 1);
                dot += f64::from(edge.gx) * f64::from(gx) + f64::from(edge.gy) * f64::from(gy);
                sample_energy += f64::from(gx) * f64::from(gx) + f64::from(gy) * f64::from(gy);
            }
            best = best.max((dot / (model.energy * sample_energy).sqrt()) as f32);
        }
    }
    best.max(0.0)
}

/// 閾値を超えた低頻度観測を、ラウンド単位の `FIGHT` 表示へまとめる。
pub fn detect_fight_markers(observations: &[FightObservation]) -> Vec<FightMarker> {
    let mut markers = Vec::new();
    let mut run: Vec<FightObservation> = Vec::new();

    for &observation in observations
        .iter()
        .filter(|observation| observation.score >= FIGHT_SCORE_THRESHOLD)
    {
        if run
            .last()
            .is_some_and(|last| observation.frame > last.frame + FIGHT_MAX_HIT_GAP)
        {
            push_marker(&mut markers, &run);
            run.clear();
        }
        run.push(observation);
    }
    push_marker(&mut markers, &run);
    markers
}

fn push_marker(markers: &mut Vec<FightMarker>, run: &[FightObservation]) {
    if run.len() < FIGHT_MIN_HITS {
        return;
    }
    let Some(peak) = run
        .iter()
        .max_by(|left, right| left.score.total_cmp(&right.score))
    else {
        return;
    };
    if peak.score < FIGHT_PEAK_THRESHOLD {
        return;
    }
    markers.push(FightMarker {
        first_frame: run[0].frame,
        last_frame: run[run.len() - 1].frame,
        peak_frame: peak.frame,
        peak_score: peak.score,
    });
}

fn fight_template_model() -> Option<&'static FightTemplateModel> {
    static MODEL: OnceLock<Option<FightTemplateModel>> = OnceLock::new();
    MODEL
        .get_or_init(|| {
            if FIGHT_TEMPLATE.len() != TEMPLATE_SIZE {
                return None;
            }
            let mut edges = Vec::new();
            let mut energy = 0.0f64;
            let x_start = usize::from((MAX_SHIFT_X + 1) as u16);
            let x_end = FIGHT_PATCH_WIDTH - x_start;
            let y_start = usize::from((MAX_SHIFT_Y + 1) as u16);
            let y_end = FIGHT_PATCH_HEIGHT - y_start;
            for y in y_start..y_end {
                for x in x_start..x_end {
                    let index = y * FIGHT_PATCH_WIDTH + x;
                    let gx =
                        i16::from(FIGHT_TEMPLATE[index + 1]) - i16::from(FIGHT_TEMPLATE[index - 1]);
                    let gy = i16::from(FIGHT_TEMPLATE[index + FIGHT_PATCH_WIDTH])
                        - i16::from(FIGHT_TEMPLATE[index - FIGHT_PATCH_WIDTH]);
                    if gx.abs() + gy.abs() < TEMPLATE_EDGE_THRESHOLD {
                        continue;
                    }
                    edges.push(TemplateEdge {
                        x: x as i16,
                        y: y as i16,
                        gx,
                        gy,
                    });
                    energy += f64::from(gx) * f64::from(gx) + f64::from(gy) * f64::from(gy);
                }
            }
            (!edges.is_empty() && energy > 0.0).then_some(FightTemplateModel { edges, energy })
        })
        .as_ref()
}

/// 画素の明るさ。テンプレートは輝度の勾配だけを持つので、照合の前に
/// 色を明るさへ落とす。
///
/// 重みは目の感度に合わせてあり、緑が最も、青が最も効かない。等しく
/// 混ぜると、色の違う場面で `FIGHT` の輪郭が出たり消えたりする。
/// 三つの重みは 256 を成すので、無彩色はそのままの値で通る。
fn luma(red: u8, green: u8, blue: u8) -> i16 {
    let weighted = 77 * u32::from(red) + 150 * u32::from(green) + 29 * u32::from(blue);
    ((weighted + 128) >> 8) as i16
}

fn patch_luma(hud_strip: &[u8], strip_width: usize, x: i16, y: i16) -> i16 {
    let source_x = FIGHT_PATCH_X + x as usize;
    let source_y = FIGHT_PATCH_Y + y as usize;
    let index = (source_y * strip_width + source_x) * 4;
    luma(hud_strip[index], hud_strip[index + 1], hud_strip[index + 2])
}

#[cfg(test)]
mod tests;
