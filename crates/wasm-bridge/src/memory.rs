use wasm_bindgen::prelude::*;

/// Exposes WASM linear memory for direct `VideoFrame.copyTo()` writes.
#[wasm_bindgen]
pub fn wasm_memory() -> JsValue {
    wasm_bindgen::memory()
}
