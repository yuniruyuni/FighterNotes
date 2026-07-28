mod label;
mod lifecycle;
mod observation;
mod reading;
mod recording;
mod replay;
mod slab;
mod update;

use std::sync::Arc;

use frame_meter::{BrightClass, CellState, RowObs, CELL_COUNT};

use super::MeterTracker;

fn tracker_at(absolute: i64) -> MeterTracker {
    let mut tracker = MeterTracker::new();
    tracker.absolute_frame = Some(absolute);
    tracker
}

fn observation_with_state(state: CellState) -> RowObs {
    let mut observation = RowObs::empty();
    observation.states.fill(state);
    observation
}

fn digit_correlations() -> Vec<[f32; 10]> {
    vec![[-1.0; 10]; CELL_COUNT]
}

fn lit_observation(edge: i32) -> RowObs {
    let mut observation = RowObs::empty();
    observation.v.fill(100.0);
    observation.bright.fill(BrightClass::Fresh);
    observation.fresh_edge = edge;
    observation
}

fn shared(observation: RowObs) -> Arc<RowObs> {
    Arc::new(observation)
}

fn shared_pair(left: RowObs, right: RowObs) -> (Arc<RowObs>, Arc<RowObs>) {
    (shared(left), shared(right))
}

fn insert_read(
    tracker: &mut MeterTracker,
    side: &str,
    absolute: i64,
    state: &str,
    confidence: f64,
    covered: bool,
) {
    tracker
        .reads
        .get_mut(side)
        .unwrap()
        .insert(absolute, (state.to_string(), confidence, covered));
}
