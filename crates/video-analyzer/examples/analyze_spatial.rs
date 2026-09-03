//! Inspect the low-cost spatial extractor with a short PNG sequence.
//!
//! Example:
//! cargo run -p video-analyzer --example analyze_spatial -- \
//!   --training --p1-x 0.20 --p2-x 0.88 /tmp/window/*.png

use std::env;
use std::error::Error;

use video_analyzer::{ActorHint, SpatialConfig, SpatialExtractor, SpatialHints, SpatialPoint};

fn main() -> Result<(), Box<dyn Error>> {
    let mut training = false;
    let mut p1_x = None;
    let mut p2_x = None;
    let mut paths = Vec::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--training" => training = true,
            "--p1-x" => p1_x = Some(parse_x(args.next(), "--p1-x")?),
            "--p2-x" => p2_x = Some(parse_x(args.next(), "--p2-x")?),
            _ => paths.push(arg),
        }
    }
    if paths.len() < 2 {
        return Err("provide at least two PNG frames".into());
    }

    let config = if training {
        SpatialConfig::sf6_training_overlay()
    } else {
        SpatialConfig::default()
    };
    let mut extractor = SpatialExtractor::new(config);

    for (index, path) in paths.iter().enumerate() {
        let rgba = image::open(path)?.to_rgba8();
        let hints = if index == 0 {
            SpatialHints {
                p1: actor_hint(p1_x),
                p2: actor_hint(p2_x),
                contact_effect: false,
            }
        } else {
            SpatialHints::default()
        };
        let observation = extractor.observe_rgba(
            index as u32,
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            hints,
        )?;
        println!("{}", serde_json::to_string(&observation)?);
    }
    Ok(())
}

fn parse_x(value: Option<String>, flag: &str) -> Result<f32, Box<dyn Error>> {
    let value = value.ok_or_else(|| format!("{flag} requires a value"))?;
    let x: f32 = value.parse()?;
    if !(0.0..=1.0).contains(&x) {
        return Err(format!("{flag} must be within 0..1").into());
    }
    Ok(x)
}

fn actor_hint(x: Option<f32>) -> ActorHint {
    ActorHint {
        anchor: x.map(|x| SpatialPoint::new(x, 0.90)),
        allow_discontinuity: false,
        allow_airborne: false,
    }
}
