pub(super) use super::super::*;

// ── ヘルパー ──────────────────────────────────────────────────────────────

pub(super) fn make_frame(left_hp_raw: f32, left_uncertain: bool, is_match: bool) -> FrameFeatures {
    FrameFeatures {
        frame_index: 0,
        fps: 60.0,
        own_hp: left_hp_raw,
        opponent_hp: 0.5,
        is_match_screen: is_match,
        own_meter_state: None,
        opponent_meter_state: None,
        left_hp_score: 0.0,
        right_hp_score: 0.0,
        left_drive_ratio: 1.0,
        right_drive_ratio: 1.0,
        left_burnout: false,
        right_burnout: false,
        left_drive_uncertain: false,
        right_drive_uncertain: false,
        left_hp_raw,
        right_hp_raw: 0.5,
        left_hp_raw_quality: if left_uncertain { 1.0 } else { 0.0 },
        right_hp_raw_quality: 0.0,
    }
}

pub(super) fn hud_strip_from_frame(rgba: &[u8]) -> Vec<u8> {
    let y1 = HUD_STRIP_Y as usize;
    let y2 = y1 + HUD_STRIP_H as usize;
    let mut strip = Vec::with_capacity(1920 * (y2 - y1) * 4);
    for y in y1..y2 {
        let start = y * 1920 * 4;
        strip.extend_from_slice(&rgba[start..start + 1920 * 4]);
    }
    strip
}

pub(super) fn paint_full_left_drive_gauge(rgba: &mut [u8]) {
    let (x1, x2, y1, y2) = DRIVE_ROI_LEFT;
    for cy in 0..(x2 - x1) as usize {
        for ry in 0..(y2 - y1) as usize {
            let offset = (ry as f32 * DRIVE_BAR_SLOPE).round() as usize;
            let x = x1 as usize + cy + offset;
            let y = y1 as usize + ry;
            let index = (y * 1920 + x) * 4;
            rgba[index..index + 4].copy_from_slice(&[240, 200, 0, 255]);
        }
    }
}

// HP バー ROI (P1): x=172-853, y=64-95, 実効行 y=69-91 (22 行)
// P1 は右端に HP が残存し、左側から空きになる仕様。
/// P1 HP バーのテスト用 RGBA バッファを生成する。
///
/// HP バーは平行四辺形（slope=0.75）で描画し、3 辺ふちどりに白ピクセルを配置する:
/// - 右辺ふちどり: 斜め列 cy=679 (= roi_w-2)
/// - 上辺ふちどり: row=0..HP_COL_ROW_SKIP_TOP, x=ROI 右端付近
/// - 下辺ふちどり: row=row_end..roi_h, x=ROI 右端付近
pub(super) fn make_rgba_p1_bar(fill_ratio: f32) -> Vec<u8> {
    let mut rgba = vec![0u8; 1920 * 1080 * 4];
    let x1 = 172usize;
    let roi_w = 681usize; // x=172..853
    let roi_h = 31usize; // y=64..95
    let y1 = 64usize;
    let row_start = HP_COL_ROW_SKIP_TOP;
    let row_end = roi_h - HP_COL_ROW_SKIP_BOTTOM; // 31-4=27
    let slope = HP_BAR_SLOPE;

    let filled = (roi_w as f32 * fill_ratio) as usize;
    let cx_start = roi_w - filled; // HP 左辺の斜め列インデックス

    // 右端 cap 列（cy=679 = roi_w-2）
    let border_cy = roi_w - 2; // = 679

    // 平行四辺形 HP バーを赤で描画（白 cap より左のみ）
    for ry in row_start..row_end {
        let x_offset = ((ry - row_start) as f32 * slope).round() as usize;
        let cap_x = x1 + border_cy + x_offset;
        let hp_x_start = (x1 + cx_start + x_offset).min(cap_x);
        let hp_x_end = cap_x.min(x1 + roi_w);
        let y = y1 + ry;
        for x in hp_x_start..hp_x_end {
            let idx = (y * 1920 + x) * 4;
            rgba[idx] = 220;
            rgba[idx + 1] = 0;
            rgba[idx + 2] = 0;
            rgba[idx + 3] = 255;
        }
    }

    // 右辺ふちどり（cy=679）: 斜め列に白を配置
    // 平行四辺形スキャンでは cap 列の下部行が x1+roi_w を超えるため、
    // 画像バウンド（1920）でクリップして classify_hp_col と同じ範囲をカバーする。
    for ry in row_start..row_end {
        let x_offset = ((ry - row_start) as f32 * slope).round() as i32;
        let gx = x1 as i32 + border_cy as i32 + x_offset;
        if (0..1920).contains(&gx) {
            let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
            rgba[idx] = 255;
            rgba[idx + 1] = 255;
            rgba[idx + 2] = 255;
            rgba[idx + 3] = 255;
        }
    }

    // fill edge ふちどり（cy=cx_start-2, cx_start-1）: HP 充填左端を示す白 2 列
    // HP=100% (cx_start=0,1) はふちどりなし（fill が端まで埋まるため不要）
    if cx_start >= 2 {
        for offset in 0..2 {
            let fe_cy = cx_start - 2 + offset;
            for ry in row_start..row_end {
                let x_off = ((ry - row_start) as f32 * slope).round() as i32;
                let gx = x1 as i32 + fe_cy as i32 + x_off;
                if gx >= x1 as i32 && gx < (x1 + roi_w) as i32 {
                    let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
                    rgba[idx] = 255;
                    rgba[idx + 1] = 255;
                    rgba[idx + 2] = 255;
                    rgba[idx + 3] = 255;
                }
            }
        }
    }

    // 上辺ふちどり（row=0..row_start）: P1 アンカー右端付近に白を配置
    let anchor_x = x1 + roi_w - 15; // ROI 右端から 15px 内側
    for ry in 0..row_start {
        let idx = ((y1 + ry) * 1920 + anchor_x) * 4;
        rgba[idx] = 255;
        rgba[idx + 1] = 255;
        rgba[idx + 2] = 255;
        rgba[idx + 3] = 255;
    }

    // 下辺ふちどり（row=row_end..roi_h）: 同じアンカー位置に白を配置
    for ry in row_end..roi_h {
        let idx = ((y1 + ry) * 1920 + anchor_x) * 4;
        rgba[idx] = 255;
        rgba[idx + 1] = 255;
        rgba[idx + 2] = 255;
        rgba[idx + 3] = 255;
    }

    rgba
}

pub(super) fn make_rgba_p1_bar_dithered(fill_ratio: f32) -> Vec<u8> {
    // ディザリングパターン: 4 列ごとに vivid red, 残りは黒。平行四辺形で描画。
    let mut rgba = vec![0u8; 1920 * 1080 * 4];
    let x1 = 172usize;
    let roi_w = 681usize;
    let roi_h = 31usize;
    let y1 = 64usize;
    let row_start = HP_COL_ROW_SKIP_TOP;
    let row_end = roi_h - HP_COL_ROW_SKIP_BOTTOM;
    let slope = HP_BAR_SLOPE;

    let filled = (roi_w as f32 * fill_ratio) as usize;
    let cx_start = roi_w - filled;

    // 右端 cap 列（cy=679 = roi_w-2）
    let border_cy = roi_w - 2;

    // ディザリング fill を描画（白 cap より左のみ）
    for ry in row_start..row_end {
        let x_offset = ((ry - row_start) as f32 * slope).round() as usize;
        let cap_x = x1 + border_cy + x_offset;
        for xi in 0..filled {
            if xi % 4 == 0 {
                let x = x1 + cx_start + xi + x_offset;
                if x < cap_x.min(x1 + roi_w) {
                    let idx = ((y1 + ry) * 1920 + x) * 4;
                    rgba[idx] = 220;
                    rgba[idx + 3] = 255;
                }
            }
        }
    }

    // 右辺ふちどり（cy=679）— 平行四辺形スキャンに合わせ画像バウンドでクリップ
    for ry in row_start..row_end {
        let x_offset = ((ry - row_start) as f32 * slope).round() as i32;
        let gx = x1 as i32 + border_cy as i32 + x_offset;
        if (0..1920).contains(&gx) {
            let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
            rgba[idx] = 255;
            rgba[idx + 1] = 255;
            rgba[idx + 2] = 255;
            rgba[idx + 3] = 255;
        }
    }

    // fill edge ふちどり（cx_start-2, cx_start-1）
    if cx_start >= 2 {
        for offset in 0..2 {
            let fe_cy = cx_start - 2 + offset;
            for ry in row_start..row_end {
                let x_off = ((ry - row_start) as f32 * slope).round() as i32;
                let gx = x1 as i32 + fe_cy as i32 + x_off;
                if gx >= x1 as i32 && gx < (x1 + roi_w) as i32 {
                    let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
                    rgba[idx] = 255;
                    rgba[idx + 1] = 255;
                    rgba[idx + 2] = 255;
                    rgba[idx + 3] = 255;
                }
            }
        }
    }

    // 上辺・下辺ふちどり
    let anchor_x = x1 + roi_w - 15;
    for ry in 0..row_start {
        let idx = ((y1 + ry) * 1920 + anchor_x) * 4;
        rgba[idx] = 255;
        rgba[idx + 1] = 255;
        rgba[idx + 2] = 255;
        rgba[idx + 3] = 255;
    }
    for ry in row_end..roi_h {
        let idx = ((y1 + ry) * 1920 + anchor_x) * 4;
        rgba[idx] = 255;
        rgba[idx + 1] = 255;
        rgba[idx + 2] = 255;
        rgba[idx + 3] = 255;
    }

    rgba
}

/// fill 域中央に暗色ブロックを挿入してスプライト遮蔽を模倣する。
pub(super) fn make_rgba_p1_bar_with_mid_occlusion(
    fill_ratio: f32,
    dark_start_cy: usize,
    dark_width: usize,
) -> Vec<u8> {
    let mut rgba = make_rgba_p1_bar(fill_ratio);
    let x1: i32 = 172;
    let y1 = 64usize;
    let roi_h = 31usize;
    let row_start = HP_COL_ROW_SKIP_TOP;
    let row_end = roi_h - HP_COL_ROW_SKIP_BOTTOM;
    let slope = HP_BAR_SLOPE;
    for cy in dark_start_cy..(dark_start_cy + dark_width) {
        for ry in row_start..row_end {
            let x_off = ((ry - row_start) as f32 * slope).round() as i32;
            let gx = x1 + cy as i32 + x_off;
            if (0..1920).contains(&gx) {
                let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 255;
            }
        }
    }
    rgba
}

/// 低 HP 状態の黄色バーを模倣する（fill pixels を R=255,G=237,B=0 に置換）。
/// HSV: H≈28(OpenCV 0-180), S≈255, V≈255 → is_fill 第2条件 (h 22-35, s>120, v>200) に合致。
pub(super) fn make_rgba_p1_bar_yellow(fill_ratio: f32) -> Vec<u8> {
    let mut rgba = make_rgba_p1_bar(fill_ratio);
    for px in rgba.chunks_exact_mut(4) {
        if px[0] == 220 && px[1] == 0 && px[2] == 0 {
            px[0] = 255;
            px[1] = 237;
            px[2] = 0;
        }
    }
    rgba
}

/// 黄色 HP バーに橙色ダメージゾーンを追加した合成データ。
pub(super) fn make_rgba_p1_bar_yellow_with_orange(
    fill_ratio: f32,
    damage_start_cy: usize,
    damage_width: usize,
) -> Vec<u8> {
    let mut rgba = make_rgba_p1_bar_yellow(fill_ratio);
    let x1: i32 = 172;
    let y1 = 64usize;
    let roi_h = 31usize;
    let row_start = HP_COL_ROW_SKIP_TOP;
    let row_end = roi_h - HP_COL_ROW_SKIP_BOTTOM;
    let slope = HP_BAR_SLOPE;
    for cy in damage_start_cy..(damage_start_cy + damage_width) {
        for ry in row_start..row_end {
            let x_off = ((ry - row_start) as f32 * slope).round() as i32;
            let gx = x1 + cy as i32 + x_off;
            if (0..1920).contains(&gx) {
                let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
                rgba[idx] = 160;
                rgba[idx + 1] = 135;
                rgba[idx + 2] = 35;
                rgba[idx + 3] = 255;
            }
        }
    }
    rgba
}

/// 高輝度橙色で fill edge を上書きする回帰データ。
pub(super) fn make_rgba_p1_bar_yellow_with_bright_orange_overwriting_fill_edge(
    fill_ratio: f32,
    damage_width: usize,
) -> Vec<u8> {
    let mut rgba = make_rgba_p1_bar_yellow(fill_ratio);
    let x1: i32 = 172;
    let y1 = 64usize;
    let roi_h = 31usize;
    let roi_w = 681usize;
    let row_start = HP_COL_ROW_SKIP_TOP;
    let row_end = roi_h - HP_COL_ROW_SKIP_BOTTOM;
    let slope = HP_BAR_SLOPE;

    let filled = (roi_w as f32 * fill_ratio) as usize;
    let cx_start = roi_w - filled;
    // damage_end = cx_start（fill_edge_white を含む fill の左端まで上書き）
    let damage_end = cx_start;
    let damage_start = damage_end.saturating_sub(damage_width + 2);

    // damage_left_white: damage 帯の左端に 2 列の白を配置
    for dl_offset in 0..2usize {
        let dl_cy = damage_start.saturating_sub(2) + dl_offset;
        for ry in row_start..row_end {
            let x_off = ((ry - row_start) as f32 * slope).round() as i32;
            let gx = x1 + dl_cy as i32 + x_off;
            if gx >= 0 && (gx as usize) < 1920 {
                let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
                rgba[idx] = 255;
                rgba[idx + 1] = 255;
                rgba[idx + 2] = 255;
                rgba[idx + 3] = 255;
            }
        }
    }

    // 高輝度橙色（fill_edge_white を上書き）: R=220, G=161, B=0
    // H=22 (OpenCV), V=220>200, G/R=161/220=0.73<0.80 → 修正後は Orange
    for cy in damage_start..damage_end {
        for ry in row_start..row_end {
            let x_off = ((ry - row_start) as f32 * slope).round() as i32;
            let gx = x1 + cy as i32 + x_off;
            if gx >= 0 && (gx as usize) < 1920 {
                let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
                rgba[idx] = 220;
                rgba[idx + 1] = 161;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 255;
            }
        }
    }

    rgba
}

/// fill と fill edge の間に暗い orange ghost を挿入した合成バー。
pub(super) fn make_rgba_p1_bar_yellow_with_dim_orange_ghost(
    total_fill_ratio: f32,
    ghost_width: usize,
) -> Vec<u8> {
    let mut rgba = make_rgba_p1_bar_yellow(total_fill_ratio);
    let x1: i32 = 172;
    let y1 = 64usize;
    let roi_h = 31usize;
    let roi_w = 681usize;
    let row_start = HP_COL_ROW_SKIP_TOP;
    let row_end = roi_h - HP_COL_ROW_SKIP_BOTTOM;
    let slope = HP_BAR_SLOPE;

    // fill_edge の右端（= fill zone 左辺 cx_start）からゴースト幅分だけ dim orange を配置。
    // fill_edge White（cx_start-2, cx_start-1）はそのまま保持。
    let filled = (roi_w as f32 * total_fill_ratio) as usize;
    let cx_start = roi_w - filled;
    let ghost_end = (cx_start + ghost_width).min(roi_w - 3);

    for cy in cx_start..ghost_end {
        for ry in row_start..row_end {
            let x_off = ((ry - row_start) as f32 * slope).round() as i32;
            let gx = x1 + cy as i32 + x_off;
            if gx >= 0 && (gx as usize) < 1920 {
                let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
                rgba[idx] = 160;
                rgba[idx + 1] = 135; // G/R = 135/160 = 0.844 > 0.82
                rgba[idx + 2] = 35;
                rgba[idx + 3] = 255;
            }
        }
    }
    rgba
}

/// KO 直後の HP バー（明るい fill なし、ゴースト残像のみ）の合成データ。
/// cap 白枠は保持し、cap の内側 ghost_ratio 分をゴースト色 (160,135,35) で塗る。
pub(super) fn make_rgba_p1_bar_ghost_only(ghost_ratio: f32) -> Vec<u8> {
    let mut rgba = make_rgba_p1_bar_yellow(0.0);
    let x1: i32 = 172;
    let y1 = 64usize;
    let roi_h = 31usize;
    let roi_w = 681usize;
    let row_start = HP_COL_ROW_SKIP_TOP;
    let row_end = roi_h - HP_COL_ROW_SKIP_BOTTOM;
    let slope = HP_BAR_SLOPE;
    let ghost_w = (roi_w as f32 * ghost_ratio) as usize;
    let ghost_end = roi_w - 2; // cap (cy=679) の直前まで
    let ghost_start = ghost_end.saturating_sub(ghost_w);
    for cy in ghost_start..ghost_end {
        for ry in row_start..row_end {
            let x_off = ((ry - row_start) as f32 * slope).round() as i32;
            let gx = x1 + cy as i32 + x_off;
            if gx >= 0 && (gx as usize) < 1920 {
                let idx = ((y1 + ry) * 1920 + gx as usize) * 4;
                rgba[idx] = 160;
                rgba[idx + 1] = 135;
                rgba[idx + 2] = 35;
                rgba[idx + 3] = 255;
            }
        }
    }
    rgba
}

// ── decode_hp_zones（アンカー正規化済みゾーン列の直接テスト、サイド非依存） ──

/// (color, width) のリストから連続ゾーン列を構築する。
pub(super) fn zones_from(spec: &[(HpColColor, usize)]) -> Vec<HpZone> {
    let mut zones = Vec::new();
    let mut pos = 0usize;
    for &(color, w) in spec {
        zones.push(HpZone {
            color,
            start: pos,
            end: pos + w - 1,
        });
        pos += w;
    }
    zones
}

pub(super) fn drive_runs_from(
    spec: &[(DriveColClass, usize)],
) -> Vec<(DriveColClass, usize, usize)> {
    let mut runs = Vec::new();
    let mut pos = 0usize;
    for &(class, w) in spec {
        runs.push((class, pos, pos + w - 1));
        pos += w;
    }
    runs
}
