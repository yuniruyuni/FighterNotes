use wasm_bindgen::prelude::*;

/// Serializes both players' frame-meter observations for one RGBA frame.
#[wasm_bindgen]
pub fn inspect_frame(rgba: &[u8], width: u32, height: u32) -> String {
    let (left, right) = frame_meter::extract_row_obs(rgba, width, height);
    serde_json::json!({ "left": left, "right": right }).to_string()
}
