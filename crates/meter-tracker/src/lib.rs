//! Tracks the SF6 frame-meter cursor and emits per-player timelines.

mod calibration;
mod model;
mod tracker;

pub use model::{MeterTimeline, TimelineEntry, TimelineSegment};
pub use tracker::MeterTracker;
