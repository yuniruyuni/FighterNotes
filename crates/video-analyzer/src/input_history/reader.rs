use super::*;

// ── 行読み取り本体 ───────────────────────────────────────────────────────────

/// 1 フレームの入力履歴を読み取る。side は "p1" / "p2"。
///
/// フルフレーム（1920x1080 想定、他解像度は scale_roi で対応）の RGBA を受け取る。
pub fn read_input_rows(rgba: &[u8], width: u32, height: u32, side: &str) -> Vec<InputRow> {
    read_input_rows_impl(rgba, width, height, side, 0, INPUT_ROWS)
}

/// 入力ストリップ（y=INPUT_STRIP_Y から INPUT_STRIP_H 行、フル幅）から
/// 先頭行（現在入力）のみを読み取る。wasm パイプライン用。
pub fn read_input_row0_from_strip(strip: &[u8], full_width: u32, side: &str) -> InputRow {
    read_input_rows_impl(strip, full_width, 1080, side, INPUT_STRIP_Y as usize, 1)
        .into_iter()
        .next()
        .unwrap_or(InputRow {
            count: None,
            dir: InputDir::Unknown,
            badges: Vec::new(),
            auto: false,
            throw: false,
            empty: true,
            uncertain: false,
        })
}

fn read_input_rows_impl(
    rgba: &[u8],
    width: u32,
    height: u32,
    side: &str,
    y_strip_start: usize,
    max_rows: usize,
) -> Vec<InputRow> {
    let is_p1 = side == "p1";
    let f = Frame {
        rgba,
        w: width as usize,
        y_off: y_strip_start,
        white_th: 210,
    };

    // 解像度スケール（1920x1080 以外にも対応）。x/y 独立スケール
    let sx = width as f32 / 1920.0;
    let sy = height as f32 / 1080.0;
    // ジオメトリが細かいため、現状は 1920x1080 のみ完全サポート。
    // 異解像度はスケールした座標で近似読み取り（テンプレートは非スケール）
    let scale_x = |x: u32| (x as f32 * sx).round() as usize;
    let scale_y = |y: u32| (y as f32 * sy).round() as usize;

    let ones_x = if is_p1 { P1_ONES_X } else { P2_ONES_X };
    let dir_x = if is_p1 { P1_DIR_X } else { P2_DIR_X };
    let badge_x = if is_p1 { P1_BADGE_X } else { P2_BADGE_X };

    // 暗転リトライ用の低閾値フレーム
    let f_dim = Frame { white_th: 180, ..f };

    let mut rows = Vec::with_capacity(max_rows);
    for ri in 0..max_rows {
        let y_top = ROW0_Y + ROW_PITCH * ri as u32;
        if scale_y(y_top) + DIGIT_H >= height as usize {
            break;
        }
        let y0 = scale_y(y_top);
        // 数字は正規化相関（輝度不変）のため単一パスで暗転にも対応
        let (count, count_unc, _) = read_count(&f, scale_x(ones_x) as u32, y0);
        let dy = (y0 as i32 + DIR_Y_OFF).max(0) as usize;
        let (d1, du1, ds1) = read_dir(&f, scale_x(dir_x), dy);
        let (d2, du2, ds2) = read_dir(&f_dim, scale_x(dir_x), dy);
        let (dir, dir_unc) = if ds2 < ds1 { (d2, du2) } else { (d1, du1) };
        let mono_x = if is_p1 { P1_MONO_X } else { P2_MONO_X };
        let (mut badges, mut auto, mut throw) = read_badges(
            &f,
            (scale_x(badge_x.0) as u32, scale_x(badge_x.1) as u32),
            (scale_x(mono_x.0) as u32, scale_x(mono_x.1) as u32),
            is_p1,
            y0,
        );

        let uncertain = count_unc || dir_unc;
        if uncertain {
            // 遮蔽・演出中の行はバッジ帯も汚染されている（"n HITS" の色文字が
            // 偽バッジを作る等）ため、バッジ出力を抑制する
            badges.clear();
            auto = false;
            throw = false;
        }
        let empty = count.is_none() && !count_unc && dir == InputDir::Unknown && !dir_unc;
        rows.push(InputRow {
            count,
            dir,
            badges,
            auto,
            throw,
            empty,
            uncertain,
        });
    }
    rows
}
