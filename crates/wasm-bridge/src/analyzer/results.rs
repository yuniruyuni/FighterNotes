use wasm_bindgen::prelude::*;

use crate::serialization::tracked_to_json;

use super::Analyzer;

#[derive(serde::Deserialize)]
struct ImportedMeterTimeline {
    left: meter_tracker::MeterTimeline,
    right: meter_tracker::MeterTimeline,
    #[serde(default)]
    attack_info: Vec<video_analyzer::AttackInfoObservation>,
}

/// ラウンド開始演出の数として辻褄が合うか。
///
/// 1 試合は 2 本先取なので、`FIGHT` は 2 回か 3 回出る。それ以外の数は、
/// 動画が途中から始まっているか、中央が隠れて読めていないか、別の何かを
/// 誤検出している。どれにせよラウンド境界には使えない。
fn marker_count_is_valid(count: usize) -> bool {
    (2..=3).contains(&count)
}

/// ラウンド開始演出が見つからなかったときに、利用者へ返す説明。
fn marker_count_error(count: usize) -> String {
    format!(
        "FIGHT のラウンド開始演出を 2〜3 回検出する必要があります（検出: {count} 回）。対戦開始前からの未編集動画で、中央画面が隠れていないことを確認してください。"
    )
}

impl Analyzer {
    /// 入力欄の読みを解析へ使えるか。
    ///
    /// フレームごとに 1 行ずつ揃っていなければ、どの入力がどのフレームの
    /// ものか決まらない。数が合わないまま使うと、入力と場面が全部ずれる。
    fn input_rows_are_usable(&self) -> bool {
        self.input_rows.len() == self.features.len() && !self.input_rows.is_empty()
    }

    fn ensure_fight_markers(&mut self) {
        if self.fight_markers.is_none() {
            self.fight_markers = Some(video_analyzer::detect_fight_markers(
                &self.fight_observations,
            ));
        }
    }

    /// 解析を組み立てる。断る理由は素の文字列で返す。`JsValue` は wasm の
    /// 外では作れないため、呼び手が境界で包む。
    fn ensure_events(&mut self) -> Result<(), String> {
        if self.events.is_some() {
            return Ok(());
        }
        if self.imported_meter.is_none() {
            self.tracker.finish();
        }
        self.ensure_fight_markers();
        let fight_markers = self
            .fight_markers
            .clone()
            .expect("fight markers initialized");
        let marker_count_is_valid = marker_count_is_valid(fight_markers.len());
        if self.require_fight_markers && !marker_count_is_valid {
            return Err(marker_count_error(fight_markers.len()));
        }
        if marker_count_is_valid {
            video_analyzer::finalize_features_with_fight_markers(
                &mut self.features,
                &fight_markers,
                self.analysis_context.own_side(),
            );
        } else {
            video_analyzer::finalize_features(&mut self.features);
        }
        let attack_info = self
            .imported_attack_info
            .as_deref()
            .unwrap_or(&self.attack_info_tracker.observations);

        let events = if self.input_rows_are_usable() {
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
            if marker_count_is_valid {
                video_analyzer::build_match_events_with_context_and_fight_markers_and_attack_info(
                    &self.features,
                    &p1_tracked,
                    &p2_tracked,
                    Some(meter),
                    &self.analysis_context,
                    &fight_markers,
                    attack_info,
                )
            } else {
                video_analyzer::build_match_events_with_context_and_attack_info(
                    &self.features,
                    &p1_tracked,
                    &p2_tracked,
                    Some(meter),
                    &self.analysis_context,
                    attack_info,
                )
            }
        } else {
            if marker_count_is_valid {
                video_analyzer::build_match_events_with_context_and_fight_markers_and_attack_info(
                    &self.features,
                    &[],
                    &[],
                    None,
                    &self.analysis_context,
                    &fight_markers,
                    attack_info,
                )
            } else {
                video_analyzer::build_match_events_with_context_and_attack_info(
                    &self.features,
                    &[],
                    &[],
                    None,
                    &self.analysis_context,
                    attack_info,
                )
            }
        };
        self.input_rows.clear();
        self.input_rows.shrink_to_fit();
        self.events = Some(events);
        Ok(())
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
            "attack_info": &self.attack_info_tracker.observations,
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

    pub fn get_fight_markers_json(&mut self) -> String {
        self.ensure_fight_markers();
        serde_json::to_string(
            self.fight_markers
                .as_deref()
                .expect("fight markers initialized"),
        )
        .unwrap_or_else(|error| format!(r#"[{{"error":"{error}"}}]"#))
    }

    pub fn finish(&mut self) -> Result<String, JsValue> {
        // GPU から届いた充填率は、解析を組み立てる前に入れておく。
        self.apply_hp_fills()
            .map_err(|error| JsValue::from_str(&error))?;
        self.ensure_events()
            .map_err(|error| JsValue::from_str(&error))?;
        Ok(self.report_json())
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
        self.imported_attack_info = Some(timeline.attack_info);
        self.imported_timeline_json = Some(timeline_json.to_string());
        Ok(())
    }

    pub fn get_spatial_windows_json(&mut self) -> Result<String, JsValue> {
        self.ensure_events()
            .map_err(|error| JsValue::from_str(&error))?;
        Ok(
            serde_json::to_string(&video_analyzer::spatial_candidate_windows(
                self.events.as_ref().expect("finalized events"),
            ))
            .unwrap_or_else(|error| format!(r#"[{{"error":"{error}"}}]"#)),
        )
    }

    pub fn refine_with_spatial(&mut self, observations_json: &str) -> Result<String, JsValue> {
        let observations: Vec<video_analyzer::SpatialObservation> =
            serde_json::from_str(observations_json).map_err(|error| {
                JsValue::from_str(&format!("invalid spatial observations: {error}"))
            })?;
        self.ensure_events()
            .map_err(|error| JsValue::from_str(&error))?;
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

    /// Takes the attack info observations read by another worker.
    ///
    /// meter のタイムラインとは別の経路で届くため、`set_meter_timeline` の
    /// 後に呼ぶ。タイムライン側が運ぶ観測を上書きする。
    pub fn set_attack_info_json(&mut self, observations_json: &str) -> Result<(), JsValue> {
        if self.events.is_some() {
            return Err(JsValue::from_str(
                "attack info cannot be changed after finalization",
            ));
        }
        let observations: Vec<video_analyzer::AttackInfoObservation> =
            serde_json::from_str(observations_json)
                .map_err(|error| JsValue::from_str(&format!("invalid attack info: {error}")))?;
        // 成果物のタイムラインは観測を同梱する形で決まっている。読み手が
        // 別ワーカーへ移っても、その形は変えない。
        if let Some(raw) = &self.imported_timeline_json {
            if let Ok(mut timeline) = serde_json::from_str::<serde_json::Value>(raw) {
                if let Ok(value) = serde_json::to_value(&observations) {
                    timeline["attack_info"] = value;
                    self.imported_timeline_json = Some(timeline.to_string());
                }
            }
        }
        self.imported_attack_info = Some(observations);
        Ok(())
    }

    pub fn get_attack_info_json(&self) -> String {
        serde_json::to_string(
            self.imported_attack_info
                .as_deref()
                .unwrap_or(&self.attack_info_tracker.observations),
        )
        .unwrap_or_else(|error| format!(r#"[{{"error":"{error}"}}]"#))
    }

    /// Returns the compact, semantic event set used by the local video
    /// regression runner. Frame-by-frame meter/input series intentionally stay
    /// in their existing diagnostics; this payload contains only events that
    /// can be annotated and matched one-to-one.
    pub fn get_regression_events_json(&mut self) -> Result<String, JsValue> {
        self.ensure_events()
            .map_err(|error| JsValue::from_str(&error))?;
        let events = self.events.as_ref().expect("finalized events");
        let attack_sequences: Vec<_> = events
            .attack_evidence
            .sequences
            .iter()
            .map(|sequence| {
                serde_json::json!({
                    "attacker": sequence.attacker,
                    "start_frame": sequence.start_frame,
                    "end_frame": sequence.end_frame,
                    "combo_damage": sequence.combo_damage,
                    "starter_attribute": sequence.starter_attribute,
                    "final_attribute": sequence.final_attribute,
                    "complete": sequence.complete,
                    "recovered_from_max": sequence.recovered_from_max,
                })
            })
            .collect();
        Ok(serde_json::json!({
            "rounds": &events.rounds,
            "damage": &events.damage,
            "super_arts": &events.super_arts,
            "attack_evidence": {
                "sequences": attack_sequences,
                "damage": &events.attack_evidence.damage,
                "super_arts": &events.attack_evidence.super_arts,
            },
        })
        .to_string())
    }

    pub fn get_timeline(&self) -> String {
        self.imported_timeline_json
            .clone()
            .unwrap_or_else(|| self.tracker_timeline_json())
    }
}

#[cfg(test)]
mod tests;
