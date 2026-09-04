//! 抽出器の出力と正解データの突き合わせ。

use spatial_refine::spatial::{DistanceBand, SpatialObservation};

use crate::sim::{GtFrame, WIDTH};

/// 評価用の意味的な距離バンド。world 距離をおおよその技レンジへ写像した
/// もので、抽出器の screen 座標閾値とは独立に定義する。
fn expected_band(separation: f32) -> DistanceBand {
    if separation <= 0.75 {
        DistanceBand::Overlap
    } else if separation <= 2.0 {
        DistanceBand::Close
    } else if separation <= 4.0 {
        DistanceBand::Mid
    } else {
        DistanceBand::Far
    }
}

#[derive(Default)]
struct ActorAccuracy {
    frames: u32,
    tracked: u32,
    observed: u32,
    error_sum: f32,
    error_max: f32,
    air_true_positive: u32,
    air_false_positive: u32,
    air_false_negative: u32,
    air_frames: u32,
}

impl ActorAccuracy {
    fn add(&mut self, gt_anchor: (f32, f32), gt_air: bool, actor: Option<(f32, bool, bool)>) {
        self.frames += 1;
        if gt_air {
            self.air_frames += 1;
        }
        let Some((anchor_x, observed, ground_anchor)) = actor else {
            if gt_air {
                self.air_false_negative += 1;
            }
            return;
        };
        self.tracked += 1;
        if observed {
            self.observed += 1;
        }
        let error = (anchor_x - gt_anchor.0).abs();
        self.error_sum += error;
        self.error_max = self.error_max.max(error);
        let predicted_air = !ground_anchor;
        match (gt_air, predicted_air) {
            (true, true) => self.air_true_positive += 1,
            (false, true) => self.air_false_positive += 1,
            (true, false) => self.air_false_negative += 1,
            (false, false) => {}
        }
    }

    fn report(&self, label: &str) -> String {
        if self.frames == 0 {
            return format!("  {label}: 計測フレームなし");
        }
        let tracked_rate = 100.0 * self.tracked as f32 / self.frames as f32;
        let observed_rate = 100.0 * self.observed as f32 / self.frames as f32;
        if self.tracked == 0 {
            return format!("  {label}: 追跡 0.0% — 全フレームで喪失");
        }
        let mean = self.error_sum / self.tracked as f32;
        format!(
            "  {label}: 追跡 {tracked_rate:5.1}% (実観測 {observed_rate:5.1}%) | anchor誤差 mean {mean:.4} max {:.4} (={:.0}px/{:.0}px @1920)",
            self.error_max,
            mean * 1920.0,
            self.error_max * 1920.0,
        )
    }
}

pub struct ScenarioMetrics {
    p1: ActorAccuracy,
    p2: ActorAccuracy,
    distance_pairs: Vec<(f32, f32)>,
    band_total: u32,
    band_matched: u32,
    contact_frames: u32,
    contact_detected: u32,
    contact_error_sum: f32,
    previous_gt: Option<GtFrame>,
    cumulative_zoom: f32,
    corrected_pairs: Vec<(f32, f32)>,
    camera_frames: u32,
    camera_estimated: u32,
    pan_error_px_sum: f32,
    zoom_error_sum: f32,
    identity_frames: u32,
    identity_swapped: u32,
}

impl ScenarioMetrics {
    pub fn new() -> Self {
        Self {
            p1: ActorAccuracy::default(),
            p2: ActorAccuracy::default(),
            distance_pairs: Vec::new(),
            band_total: 0,
            band_matched: 0,
            contact_frames: 0,
            contact_detected: 0,
            contact_error_sum: 0.0,
            previous_gt: None,
            cumulative_zoom: 1.0,
            corrected_pairs: Vec::new(),
            camera_frames: 0,
            camera_estimated: 0,
            pan_error_px_sum: 0.0,
            zoom_error_sum: 0.0,
            identity_frames: 0,
            identity_swapped: 0,
        }
    }

    /// 1 フレーム分を突き合わせる。`contact_estimate` は衝突位置の推定で、
    /// 未実装の構成では常に `None` を渡す。
    pub fn add(
        &mut self,
        gt: &GtFrame,
        observation: &SpatialObservation,
        contact_estimate: Option<(f32, f32)>,
    ) {
        if let Some(camera) = observation.camera {
            self.cumulative_zoom *= camera.zoom_ratio;
        }
        if let Some(previous) = self.previous_gt {
            if gt.measure {
                self.camera_frames += 1;
                if let Some(camera) = observation.camera {
                    self.camera_estimated += 1;
                    let gt_pan_px = (previous.cam_center_x - gt.cam_center_x) * gt.cam_scale;
                    let gt_zoom = gt.cam_scale / previous.cam_scale;
                    self.pan_error_px_sum += (camera.pan_dx * WIDTH as f32 - gt_pan_px).abs();
                    self.zoom_error_sum += (camera.zoom_ratio - gt_zoom).abs();
                }
            }
        }
        self.previous_gt = Some(*gt);
        if !gt.measure {
            return;
        }
        let actor_tuple = |actor: &Option<spatial_refine::spatial::ActorObservation>| {
            actor
                .as_ref()
                .map(|a| (a.anchor.x, a.observed, a.ground_anchor))
        };
        self.p1
            .add(gt.p1_anchor, gt.p1_air, actor_tuple(&observation.p1));
        self.p2
            .add(gt.p2_anchor, gt.p2_air, actor_tuple(&observation.p2));
        // 同定: 両方を追跡できているフレームで、互いに逆の正解へ近ければ
        // P1/P2 を取り違えている。
        if let (Some(p1), Some(p2)) = (&observation.p1, &observation.p2) {
            self.identity_frames += 1;
            let p1_wrong =
                (p1.anchor.x - gt.p2_anchor.0).abs() < (p1.anchor.x - gt.p1_anchor.0).abs();
            let p2_wrong =
                (p2.anchor.x - gt.p1_anchor.0).abs() < (p2.anchor.x - gt.p2_anchor.0).abs();
            if p1_wrong && p2_wrong {
                self.identity_swapped += 1;
            }
        }

        if let Some(screen_distance) = observation.screen_distance {
            self.distance_pairs.push((screen_distance, gt.separation));
            self.corrected_pairs
                .push((screen_distance / self.cumulative_zoom, gt.separation));
        }
        if let Some(band) = observation.distance_band {
            self.band_total += 1;
            if band == expected_band(gt.separation) {
                self.band_matched += 1;
            }
        }
        if let Some(gt_contact) = gt.contact {
            self.contact_frames += 1;
            if let Some(estimate) = contact_estimate {
                self.contact_detected += 1;
                let dx = estimate.0 - gt_contact.0;
                let dy = estimate.1 - gt_contact.1;
                self.contact_error_sum += (dx * dx + dy * dy).sqrt();
            }
        }
    }

    pub fn print(&self, name: &str, description: &str) {
        println!("== {name} — {description}");
        println!("{}", self.p1.report("P1"));
        println!("{}", self.p2.report("P2"));
        let air = combine_air(&self.p1, &self.p2);
        println!("  {air}");
        println!("  {}", self.distance_report());
        println!("  {}", self.contact_report());
        println!("  {}", self.camera_report());
        if self.identity_frames > 0 {
            println!(
                "  同定: 取り違え {}f / {}f ({:.1}%)",
                self.identity_swapped,
                self.identity_frames,
                100.0 * self.identity_swapped as f32 / self.identity_frames as f32,
            );
        }
        println!();
    }

    fn distance_report(&self) -> String {
        if self.distance_pairs.len() < 3 {
            return "距離: 出力がほぼ無い".to_string();
        }
        let rho = spearman(&self.distance_pairs);
        let band = if self.band_total == 0 {
            "バンド出力なし".to_string()
        } else {
            format!(
                "バンド一致 {:.1}% (N={})",
                100.0 * self.band_matched as f32 / self.band_total as f32,
                self.band_total
            )
        };
        let corrected = spearman(&self.corrected_pairs);
        let linear_raw = pearson_pairs(&self.distance_pairs);
        let linear_corrected = pearson_pairs(&self.corrected_pairs);
        format!(
            "距離: ρ {rho:.3} → 補正後 {corrected:.3} | 線形r {linear_raw:.3} → 補正後 {linear_corrected:.3} (N={}) | {band}",
            self.distance_pairs.len()
        )
    }

    fn contact_report(&self) -> String {
        if self.contact_frames == 0 {
            return "接触位置: このシナリオに接触なし".to_string();
        }
        if self.contact_detected == 0 {
            return format!("接触位置: GT {}f / 検出 0f (未実装)", self.contact_frames);
        }
        format!(
            "接触位置: GT {}f / 検出 {}f | 誤差 mean {:.4} (={:.0}px @1920)",
            self.contact_frames,
            self.contact_detected,
            self.contact_error_sum / self.contact_detected as f32,
            self.contact_error_sum / self.contact_detected as f32 * 1920.0,
        )
    }
}

impl ScenarioMetrics {
    fn camera_report(&self) -> String {
        if self.camera_frames == 0 {
            return "カメラ: 計測フレームなし".to_string();
        }
        if self.camera_estimated == 0 {
            return "カメラ: 推定 0% (未実装または全フレーム棄却)".to_string();
        }
        format!(
            "カメラ: 推定 {:.1}% | pan誤差 mean {:.2}px@480 | zoom誤差 mean {:.4}",
            100.0 * self.camera_estimated as f32 / self.camera_frames as f32,
            self.pan_error_px_sum / self.camera_estimated as f32,
            self.zoom_error_sum / self.camera_estimated as f32,
        )
    }
}

fn combine_air(p1: &ActorAccuracy, p2: &ActorAccuracy) -> String {
    let air_frames = p1.air_frames + p2.air_frames;
    if air_frames == 0 {
        let false_positive = p1.air_false_positive + p2.air_false_positive;
        let grounded = p1.tracked + p2.tracked;
        if grounded == 0 {
            return "空中判定: 追跡なし".to_string();
        }
        return format!("空中判定: GT空中なし | 空中誤検出 {false_positive}f / 追跡 {grounded}f");
    }
    let true_positive = (p1.air_true_positive + p2.air_true_positive) as f32;
    let false_positive = (p1.air_false_positive + p2.air_false_positive) as f32;
    let false_negative = (p1.air_false_negative + p2.air_false_negative) as f32;
    let precision = if true_positive + false_positive > 0.0 {
        true_positive / (true_positive + false_positive)
    } else {
        0.0
    };
    let recall = true_positive / (true_positive + false_negative);
    format!("空中判定: precision {precision:.2} recall {recall:.2} (GT空中 {air_frames}f)")
}

/// Spearman の順位相関。screen 距離が world 距離の単調な指標として
/// どこまで信用できるかを測る。
fn spearman(pairs: &[(f32, f32)]) -> f32 {
    let xs: Vec<f32> = pairs.iter().map(|p| p.0).collect();
    let ys: Vec<f32> = pairs.iter().map(|p| p.1).collect();
    pearson(&ranks(&xs), &ranks(&ys))
}

fn pearson_pairs(pairs: &[(f32, f32)]) -> f32 {
    let xs: Vec<f32> = pairs.iter().map(|p| p.0).collect();
    let ys: Vec<f32> = pairs.iter().map(|p| p.1).collect();
    pearson(&xs, &ys)
}

fn ranks(values: &[f32]) -> Vec<f32> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut result = vec![0.0f32; values.len()];
    for (rank, &index) in order.iter().enumerate() {
        result[index] = rank as f32;
    }
    result
}

fn pearson(xs: &[f32], ys: &[f32]) -> f32 {
    let n = xs.len() as f32;
    let mean_x = xs.iter().sum::<f32>() / n;
    let mean_y = ys.iter().sum::<f32>() / n;
    let mut covariance = 0.0;
    let mut variance_x = 0.0;
    let mut variance_y = 0.0;
    for (x, y) in xs.iter().zip(ys) {
        covariance += (x - mean_x) * (y - mean_y);
        variance_x += (x - mean_x) * (x - mean_x);
        variance_y += (y - mean_y) * (y - mean_y);
    }
    if variance_x <= 0.0 || variance_y <= 0.0 {
        return 0.0;
    }
    covariance / (variance_x.sqrt() * variance_y.sqrt())
}
