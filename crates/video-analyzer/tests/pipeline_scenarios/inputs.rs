use std::ops::RangeInclusive;

use video_analyzer::input_history::BtnGlyph;
use video_analyzer::{BadgeColor, BadgeMark, InputDir, TrackedInput};

pub fn neutral_inputs(frame_count: usize) -> Vec<TrackedInput> {
    (0..frame_count)
        .map(|index| TrackedInput {
            count: Some(index as u32 + 1),
            dir: InputDir::Neutral,
            badges: Vec::new(),
            auto: false,
            throw: false,
            repaired: false,
            uncertain: false,
        })
        .collect()
}

pub fn classic_punch(color: BadgeColor) -> BadgeMark {
    BadgeMark {
        color,
        boxed: false,
        glyph: Some(BtnGlyph::Punch),
    }
}

pub fn set_input_run(
    inputs: &mut [TrackedInput],
    frames: RangeInclusive<u32>,
    dir: InputDir,
    badges: Vec<BadgeMark>,
) {
    let start = *frames.start();
    for frame in frames {
        inputs[frame as usize] = TrackedInput {
            count: Some(frame - start + 1),
            dir,
            badges: badges.clone(),
            auto: false,
            throw: false,
            repaired: false,
            uncertain: false,
        };
    }
}
