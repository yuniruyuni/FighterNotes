use super::{
    super_gauge_read, Patch, FULL_BAR_LEFT, FULL_BAR_RIGHT, FULL_LABEL_LEFT, FULL_LABEL_RIGHT,
};

/// SA ゲージの単フレーム認識値と読み取り ROI をデバッグ表示向けに返す。
pub fn super_gauge_debug_json(rgba: &[u8], width: u32, height: u32, side: &str) -> String {
    let is_left = side == "left";
    let read = super_gauge_read(rgba, width, height, side);
    let (label, bar) = if is_left {
        (FULL_LABEL_LEFT, FULL_BAR_LEFT)
    } else {
        (FULL_LABEL_RIGHT, FULL_BAR_RIGHT)
    };
    serde_json::json!({
        "value": read.value,
        "displayed_level": read.displayed_level,
        "critical_art": read.critical_art,
        "uncertain": read.uncertain,
        "label_roi": patch_json(label),
        "bar_roi": patch_json(bar),
    })
    .to_string()
}

fn patch_json(patch: Patch) -> serde_json::Value {
    serde_json::json!({
        "x1": patch.x,
        "x2": patch.x + patch.width,
        "y1": patch.y,
        "y2": patch.y + patch.height,
    })
}
