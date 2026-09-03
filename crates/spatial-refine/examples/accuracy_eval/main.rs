//! 空間解析の精度計測ハーネス。
//!
//! 正解データ付きの合成シーンを production 構成の抽出器へ流し、
//! anchor 誤差・追跡維持率・空中判定・距離指標を数値化する。
//! 手法の追加や閾値変更の前後で同じシナリオを流し、相対比較に使う。
//!
//! ```text
//! cargo run -p spatial-refine --example accuracy_eval --release
//! ```

mod metrics;
mod scenarios;
mod sim;

use spatial_refine::spatial::{SpatialConfig, SpatialExtractor};

use metrics::ScenarioMetrics;
use sim::{HEIGHT, WIDTH};

fn main() {
    println!("spatial accuracy eval — 合成シーンによる相対比較\n");
    for scenario in scenarios::all() {
        let mut extractor = SpatialExtractor::new(SpatialConfig::sf6_training_overlay());
        let mut collected = ScenarioMetrics::new();
        let mut frame_index = 0u32;
        (scenario.run)(&mut |gt, frame, hints| {
            let observation = extractor
                .observe_rgba(frame_index, &frame, WIDTH, HEIGHT, hints)
                .expect("合成フレームの観測に失敗");
            let contact_estimate = observation
                .contact
                .as_ref()
                .map(|contact| (contact.center.x, contact.center.y));
            if std::env::var("EVAL_DEBUG_SCENARIO").as_deref() == Ok(scenario.name) && gt.measure {
                println!(
                    "f{frame_index} gt=({:.3},{:.3}) est=({:?},{:?}) regions={:?}",
                    gt.p1_anchor.0,
                    gt.p2_anchor.0,
                    observation.p1.as_ref().map(|a| (a.anchor.x, a.observed)),
                    observation.p2.as_ref().map(|a| (a.anchor.x, a.observed)),
                    observation
                        .motion_regions
                        .iter()
                        .map(|r| (r.bounds.left, r.bounds.right, r.bounds.bottom))
                        .collect::<Vec<_>>()
                );
            }
            if std::env::var("EVAL_DEBUG").is_ok() && gt.contact.is_some() {
                println!(
                    "f{frame_index} gt={:?} est={contact_estimate:?} p1={:?} p2={:?} regions={:?}",
                    gt.contact,
                    observation.p1.as_ref().map(|a| (a.anchor.x, a.observed)),
                    observation.p2.as_ref().map(|a| (a.anchor.x, a.observed)),
                    observation
                        .motion_regions
                        .iter()
                        .map(|r| (
                            r.bounds.left,
                            r.bounds.right,
                            r.changed_cells,
                            r.effect_color_fraction
                        ))
                        .collect::<Vec<_>>()
                );
            }
            collected.add(&gt, &observation, contact_estimate);
            frame_index += 1;
        });
        collected.print(scenario.name, scenario.description);
    }
}
