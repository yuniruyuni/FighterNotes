use wasm_bindgen::prelude::*;

use super::Analyzer;

#[wasm_bindgen]
impl Analyzer {
    /// Sets the user's and opponent's character names using the legacy mapping.
    pub fn set_characters(&mut self, own_char: &str, opponent_char: &str) {
        self.analysis_context
            .set_characters(own_char, opponent_char);
    }

    /// Sets player-normalized analysis metadata from JSON.
    pub fn set_analysis_context(&mut self, context_json: &str) -> Result<(), JsValue> {
        let mut context: video_analyzer::AnalysisContext = serde_json::from_str(context_json)
            .map_err(|error| JsValue::from_str(&format!("invalid analysis context: {error}")))?;
        context.normalize_for_side(&self.own_side);
        self.analysis_context = context;
        // The browser pipeline supplies the central FIGHT image. Legacy Rust
        // callers that only use set_characters keep the HP-based compatibility path.
        self.require_fight_markers = true;
        Ok(())
    }
}
