mod drive_filling;
mod drive_segments;
mod hp_crashes;
mod hp_monotonic;
mod hp_uncertainty;
mod support;

use super::{clean_drive_temporal, confirm_hp};

#[test]
fn empty_feature_sequence_is_a_noop() {
    let mut features = Vec::new();

    confirm_hp(&mut features);
    clean_drive_temporal(&mut features);

    assert!(features.is_empty());
}
