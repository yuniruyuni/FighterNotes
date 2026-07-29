use wasm_bindgen::prelude::*;

use crate::serialization::tracked_to_json;

use super::Analyzer;

#[derive(serde::Deserialize)]
struct ImportedMeterTimeline {
    left: meter_tracker::MeterTimeline,
    right: meter_tracker::MeterTimeline,
}

impl Analyzer {
    fn ensure_events(&mut self) {
        if self.events.is_some() {
            return;
        }
        if self.imported_meter.is_none() {
            self.tracker.finish();
        }
        video_analyzer::finalize_features(&mut self.features);

        let events = if self.input_rows.len() == self.features.len() && !self.input_rows.is_empty()
        {
            let p1_rows: Vec<_> = self.input_rows.iter().map(|(p1, _)| p1.clone()).collect();
            let p2_rows: Vec<_> = self.input_rows.iter().map(|(_, p2)| p2.clone()).collect();
            let p1_tracked = video_analyzer::repair_row0_sequence(&p1_rows);
            let p2_tracked = video_analyzer::repair_row0_sequence(&p2_rows);
            self.tracked_json = Some(format!(
                r#"{{"p1":[{}],"p2":[{}]}}"#,
                tracked_to_json(&p1_tracked),
                tracked_to_json(&p2_tracked),
            ));
            let meter = self
                .imported_meter
                .as_ref()
                .map(|(left, right)| (left, right))
                .unwrap_or((&self.tracker.left, &self.tracker.right));
            video_analyzer::build_match_events_with_context(
                &self.features,
                &p1_tracked,
                &p2_tracked,
                Some(meter),
                &self.analysis_context,
            )
        } else {
            video_analyzer::build_match_events_with_context(
                &self.features,
                &[],
                &[],
                None,
                &self.analysis_context,
            )
        };
        self.input_rows.clear();
        self.input_rows.shrink_to_fit();
        self.events = Some(events);
    }

    fn report_json(&self) -> String {
        let report = video_analyzer::advice::build_report_with_context(
            &self.features,
            self.events.as_ref().expect("finalized events"),
            &self.analysis_context,
        );
        serde_json::to_string(&report).unwrap_or_else(|error| format!(r#"{{"error":"{error}"}}"#))
    }

    fn tracker_timeline_json(&self) -> String {
        serde_json::json!({
            "left": &self.tracker.left,
            "right": &self.tracker.right,
            "video_map": &self.tracker.video_map,
        })
        .to_string()
    }
}

#[wasm_bindgen]
impl Analyzer {
    pub fn get_features_json(&self) -> String {
        serde_json::to_string(&self.features)
            .unwrap_or_else(|error| format!(r#"[{{"error":"{error}"}}]"#))
    }

    pub fn finish(&mut self) -> String {
        self.ensure_events();
        self.report_json()
    }

    pub fn finish_meter_timeline(&mut self) -> String {
        self.tracker.finish();
        self.tracker_timeline_json()
    }

    pub fn set_meter_timeline(&mut self, timeline_json: &str) -> Result<(), JsValue> {
        if self.events.is_some() {
            return Err(JsValue::from_str(
                "meter timeline cannot be changed after finalization",
            ));
        }
        let timeline: ImportedMeterTimeline = serde_json::from_str(timeline_json)
            .map_err(|error| JsValue::from_str(&format!("invalid meter timeline: {error}")))?;
        self.imported_meter = Some((timeline.left, timeline.right));
        self.imported_timeline_json = Some(timeline_json.to_string());
        Ok(())
    }

    pub fn get_spatial_windows_json(&mut self) -> String {
        self.ensure_events();
        serde_json::to_string(&video_analyzer::spatial_candidate_windows(
            self.events.as_ref().expect("finalized events"),
        ))
        .unwrap_or_else(|error| format!(r#"[{{"error":"{error}"}}]"#))
    }

    pub fn refine_with_spatial(&mut self, observations_json: &str) -> Result<String, JsValue> {
        let observations: Vec<video_analyzer::SpatialObservation> =
            serde_json::from_str(observations_json).map_err(|error| {
                JsValue::from_str(&format!("invalid spatial observations: {error}"))
            })?;
        self.ensure_events();
        video_analyzer::refine_match_events_with_spatial(
            self.events.as_mut().expect("finalized events"),
            &observations,
            &self.analysis_context,
        );
        Ok(self.report_json())
    }

    pub fn get_tracked_inputs(&self) -> String {
        self.tracked_json
            .clone()
            .unwrap_or_else(|| "null".to_string())
    }

    pub fn get_timeline(&self) -> String {
        self.imported_timeline_json
            .clone()
            .unwrap_or_else(|| self.tracker_timeline_json())
    }
}
