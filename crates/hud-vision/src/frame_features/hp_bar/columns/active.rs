use super::palette::{ratio, BarColour};
use super::scan::ColumnScan;

/// HP ROI 内の各列が HP 色かどうかを返す（デバッグ・充填率計算の共通ヘルパー）。
///
/// 戻り値の長さ = ROI 幅（スケール済み列数）。空 ROI は空 Vec を返す。
pub fn hp_col_active(rgba: &[u8], width: u32, height: u32, side: &str) -> Vec<bool> {
    // 帯だけを渡す入口は無い。列の判定は全画面からしか呼ばれない。
    let Some(scan) = ColumnScan::new(width, height, side, 0) else {
        return Vec::new();
    };
    // 残量は本来の色でも、危険域の黄でも成立する。黄は髪などのテクスチャと
    // 紛れるため、列のほとんどが黄であることを求める。
    let remaining_ratio = if side == "p1" {
        ratio::REMAINING_P1
    } else {
        ratio::REMAINING_P2
    };
    let by_colour = scan.columns_where(
        rgba,
        remaining_ratio,
        BarColour::RemainingHealth {
            first_player: side == "p1",
        },
    );
    let by_yellow = scan.columns_where(rgba, ratio::REMAINING_YELLOW, BarColour::LowHealthYellow);
    by_colour
        .into_iter()
        .zip(by_yellow)
        .map(|(colour, yellow)| colour || yellow)
        .collect()
}
