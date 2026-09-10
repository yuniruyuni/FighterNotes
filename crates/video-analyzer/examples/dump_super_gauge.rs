//! 診断用: SA ゲージ帯 PNG 列(1920 幅、y=950 起点で切り出した帯)から
//! 左右の SA ゲージ読みを CSV で吐く。
//!
//! 抽出例:
//!   ffmpeg -i video.mp4 -vf "crop=1920:85:0:950" -start_number 0 band_%06d.png
//!
//! 使い方:
//!   cargo run --release --example dump_super_gauge -- /tmp/bands/band_*.png

const W: u32 = 1920;
const H: u32 = 1080;
const BAND_Y: usize = 950;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let events_mode = args.first().map(|s| s == "--events").unwrap_or(false);
    if events_mode {
        args.remove(0);
    }
    let mut canvas = vec![0u8; (W * H * 4) as usize];
    let mut features: Vec<video_analyzer::FrameFeatures> = Vec::new();
    if !events_mode {
        println!("frame,left_value,left_uncertain,left_ca,right_value,right_uncertain,right_ca");
    }
    for (frame, path) in args.iter().enumerate() {
        let image = image::open(path).expect("band png").to_rgba8();
        assert_eq!(image.width(), W, "band width");
        let band_h = image.height() as usize;
        for row in 0..band_h {
            let y = BAND_Y + row;
            let src = &image.as_raw()[row * W as usize * 4..(row + 1) * W as usize * 4];
            canvas[y * W as usize * 4..(y + 1) * W as usize * 4].copy_from_slice(src);
        }
        let left = video_analyzer::super_gauge_read(&canvas, W, H, "left");
        let right = video_analyzer::super_gauge_read(&canvas, W, H, "right");
        if events_mode {
            features.push(video_analyzer::FrameFeatures {
                frame_index: frame as u32,
                fps: 60.0,
                own_hp: 1.0,
                opponent_hp: 1.0,
                is_match_screen: true,
                own_meter_state: None,
                opponent_meter_state: None,
                left_hp_score: 1.0,
                right_hp_score: 1.0,
                left_drive_ratio: 0.0,
                right_drive_ratio: 0.0,
                left_burnout: false,
                right_burnout: false,
                left_drive_uncertain: true,
                right_drive_uncertain: true,
                left_super_value: left.value,
                right_super_value: right.value,
                left_super_uncertain: left.uncertain,
                right_super_uncertain: right.uncertain,
                left_ca_ready: left.critical_art,
                right_ca_ready: right.critical_art,
                left_hp_raw: 1.0,
                right_hp_raw: 1.0,
                left_hp_raw_quality: 0.0,
                right_hp_raw_quality: 0.0,
            });
        } else {
            println!(
                "{frame},{:.3},{},{},{:.3},{},{}",
                left.value,
                left.uncertain as u8,
                left.critical_art as u8,
                right.value,
                right.uncertain as u8,
                right.critical_art as u8
            );
        }
    }
    if events_mode {
        video_analyzer::clean_super_temporal(&mut features);
        if let Ok(range) = std::env::var("DUMP_CLEANED") {
            let (a, b) = range.split_once('-').unwrap();
            let (a, b): (u32, u32) = (a.parse().unwrap(), b.parse().unwrap());
            for feature in &features {
                if feature.frame_index >= a && feature.frame_index <= b {
                    eprintln!(
                        "f{} L {:.2} unc={} | R {:.2} unc={}",
                        feature.frame_index,
                        feature.left_super_value,
                        feature.left_super_uncertain as u8,
                        feature.right_super_value,
                        feature.right_super_uncertain as u8
                    );
                }
            }
        }
        for (side, name) in [(0usize, "left"), (1usize, "right")] {
            let mut previous: Option<f32> = None;
            for feature in &features {
                let (value, uncertain) = if side == 0 {
                    (feature.left_super_value, feature.left_super_uncertain)
                } else {
                    (feature.right_super_value, feature.right_super_uncertain)
                };
                if uncertain {
                    continue;
                }
                if let Some(previous_value) = previous {
                    let before = previous_value.floor() as i32;
                    let after = value.floor() as i32;
                    if after < before && previous_value - value >= 0.65
                    /* MIN_SUPER_SPEND_DROP */
                    {
                        println!(
                            "EVENT {name} f{} level {} ({:.2} -> {:.2})",
                            feature.frame_index,
                            (before - after).clamp(1, 3),
                            previous_value,
                            value
                        );
                    }
                }
                previous = Some(value);
            }
        }
    }
}
