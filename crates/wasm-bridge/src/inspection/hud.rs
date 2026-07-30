use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn hp_parallelogram_json() -> String {
    let format_point = |x: i32, y: i32| format!("{{\"x\":{x},\"y\":{y}}}");
    let format_parallelogram = |geometry: video_analyzer::HpParallelogram| {
        format!(
            "{{\"top_left\":{},\"top_right\":{},\"bottom_right\":{},\"bottom_left\":{}}}",
            format_point(geometry.top_left.0, geometry.top_left.1),
            format_point(geometry.top_right.0, geometry.top_right.1),
            format_point(geometry.bottom_right.0, geometry.bottom_right.1),
            format_point(geometry.bottom_left.0, geometry.bottom_left.1),
        )
    };
    format!(
        "{{\"p1\":{},\"p2\":{}}}",
        format_parallelogram(video_analyzer::hp_parallelogram("p1")),
        format_parallelogram(video_analyzer::hp_parallelogram("p2")),
    )
}

#[wasm_bindgen]
pub fn inspect_hp(rgba: &[u8], width: u32, height: u32) -> String {
    let left_col_active = video_analyzer::hp_col_active(rgba, width, height, "p1");
    let right_col_active = video_analyzer::hp_col_active(rgba, width, height, "p2");
    let left_col_orange = video_analyzer::hp_col_orange(rgba, width, height, "p1");
    let right_col_orange = video_analyzer::hp_col_orange(rgba, width, height, "p2");
    let left_col_yellow = video_analyzer::hp_col_yellow(rgba, width, height, "p1");
    let right_col_yellow = video_analyzer::hp_col_yellow(rgba, width, height, "p2");
    let left_score = video_analyzer::hp_bar_score(rgba, width, height, "p1");
    let right_score = video_analyzer::hp_bar_score(rgba, width, height, "p2");
    let left_fill = video_analyzer::hp_fill_ratio(rgba, width, height, "p1");
    let right_fill = video_analyzer::hp_fill_ratio(rgba, width, height, "p2");
    let left_drive = video_analyzer::drive_fill_ratio(rgba, width, height, "left");
    let right_drive = video_analyzer::drive_fill_ratio(rgba, width, height, "right");
    let left_orange_fill = video_analyzer::hp_damage_fill(rgba, width, height, "p1");
    let right_orange_fill = video_analyzer::hp_damage_fill(rgba, width, height, "p2");
    let left_yellow_fill = true_fraction(&left_col_yellow);
    let right_yellow_fill = true_fraction(&right_col_yellow);
    serde_json::json!({
        "left_score": left_score,
        "right_score": right_score,
        "left_fill": left_fill,
        "right_fill": right_fill,
        "left_drive": left_drive,
        "right_drive": right_drive,
        "left_col_active": left_col_active,
        "right_col_active": right_col_active,
        "left_col_orange": left_col_orange,
        "right_col_orange": right_col_orange,
        "left_col_yellow": left_col_yellow,
        "right_col_yellow": right_col_yellow,
        "left_orange_fill": left_orange_fill,
        "right_orange_fill": right_orange_fill,
        "left_yellow_fill": left_yellow_fill,
        "right_yellow_fill": right_yellow_fill,
    })
    .to_string()
}

#[wasm_bindgen]
pub fn inspect_drive(rgba: &[u8], width: u32, height: u32) -> String {
    let left = video_analyzer::drive_bar_debug_json(rgba, width, height, "left");
    let right = video_analyzer::drive_bar_debug_json(rgba, width, height, "right");
    format!(r#"{{"left":{left},"right":{right}}}"#)
}

#[wasm_bindgen]
pub fn inspect_super(rgba: &[u8], width: u32, height: u32) -> String {
    let left = video_analyzer::super_gauge_debug_json(rgba, width, height, "left");
    let right = video_analyzer::super_gauge_debug_json(rgba, width, height, "right");
    format!(r#"{{"left":{left},"right":{right}}}"#)
}

#[wasm_bindgen]
pub fn inspect_input(rgba: &[u8], width: u32, height: u32) -> String {
    let p1 = video_analyzer::input_history_debug_json(rgba, width, height, "p1");
    let p2 = video_analyzer::input_history_debug_json(rgba, width, height, "p2");
    format!(r#"{{"p1":{p1},"p2":{p2}}}"#)
}

fn true_fraction(values: &[bool]) -> f32 {
    values.iter().filter(|&&value| value).count() as f32 / values.len().max(1) as f32
}
