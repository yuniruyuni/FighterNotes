use wasm_bindgen::prelude::*;

/// Spatial analyzer for short, non-contiguous candidate windows.
#[wasm_bindgen]
pub struct SpatialWindowAnalyzer {
    extractor: video_analyzer::SpatialExtractor,
    rgba_buf: Vec<u8>,
    width: u32,
    height: u32,
    observations: Vec<video_analyzer::SpatialObservation>,
}

#[wasm_bindgen]
impl SpatialWindowAnalyzer {
    #[wasm_bindgen(constructor)]
    pub fn new(
        width: u32,
        height: u32,
        training_overlay: bool,
    ) -> Result<SpatialWindowAnalyzer, JsValue> {
        let length = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(4))
            .filter(|_| width > 0 && height > 0)
            .ok_or_else(|| JsValue::from_str("invalid spatial frame dimensions"))?;
        let config = if training_overlay {
            video_analyzer::SpatialConfig::sf6_training_overlay()
        } else {
            video_analyzer::SpatialConfig::default()
        };
        Ok(Self {
            extractor: video_analyzer::SpatialExtractor::new(config),
            rgba_buf: vec![0; length],
            width,
            height,
            observations: Vec::new(),
        })
    }

    pub fn rgba_buf_ptr(&self) -> u32 {
        self.rgba_buf.as_ptr() as u32
    }

    pub fn rgba_buf_len(&self) -> u32 {
        self.rgba_buf.len() as u32
    }

    pub fn reset_window(&mut self) {
        self.extractor.reset_window();
    }

    // フレームごとのヒントは JS 境界では素朴な bool 引数が最も読める。
    #[allow(clippy::too_many_arguments)]
    pub fn observe_inplace(
        &mut self,
        frame_index: u32,
        p1_teleport: bool,
        p2_teleport: bool,
        p1_airborne: bool,
        p2_airborne: bool,
        contact: bool,
        sides_certain: bool,
    ) -> Result<(), JsValue> {
        let observation = self
            .extractor
            .observe_rgba(
                frame_index,
                &self.rgba_buf,
                self.width,
                self.height,
                video_analyzer::SpatialHints {
                    p1: video_analyzer::ActorHint {
                        anchor: None,
                        allow_discontinuity: p1_teleport,
                        allow_airborne: p1_airborne,
                    },
                    p2: video_analyzer::ActorHint {
                        anchor: None,
                        allow_discontinuity: p2_teleport,
                        allow_airborne: p2_airborne,
                    },
                    contact_effect: contact,
                    sides_certain,
                },
            )
            .map_err(|error| JsValue::from_str(&error.to_string()))?;
        self.observations.push(observation);
        Ok(())
    }

    pub fn get_observations_json(&self) -> String {
        serde_json::to_string(&self.observations)
            .unwrap_or_else(|error| format!(r#"[{{"error":"{error}"}}]"#))
    }
}

#[cfg(test)]
mod tests {
    use super::SpatialWindowAnalyzer;

    #[test]
    fn reuses_buffer_across_windows() {
        let mut analyzer = SpatialWindowAnalyzer::new(16, 9, false).unwrap();
        assert_eq!(analyzer.rgba_buf_len(), 16 * 9 * 4);
        let frame = vec![0u8; 16 * 9 * 4];
        analyzer.rgba_buf.copy_from_slice(&frame);
        analyzer
            .observe_inplace(10, false, false, false, false, false, false)
            .unwrap();
        analyzer.reset_window();
        analyzer.rgba_buf.copy_from_slice(&frame);
        analyzer
            .observe_inplace(20, false, true, false, true, false, false)
            .unwrap();

        let observations: Vec<video_analyzer::SpatialObservation> =
            serde_json::from_str(&analyzer.get_observations_json()).unwrap();
        assert_eq!(
            observations
                .iter()
                .map(|observation| observation.frame_index)
                .collect::<Vec<_>>(),
            [10, 20]
        );
    }
}
