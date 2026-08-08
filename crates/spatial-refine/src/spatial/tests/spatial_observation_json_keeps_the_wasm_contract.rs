use super::*;

#[test]
fn spatial_observation_json_keeps_the_wasm_contract() {
    let observation = SpatialObservation {
        frame_index: 7,
        p1: None,
        p2: None,
        screen_distance: Some(0.25),
        distance_band: Some(DistanceBand::Mid),
        horizontal_order: Some(HorizontalOrder::P1Left),
        projectile_candidates: vec![],
        motion_regions: vec![],
    };

    let json = serde_json::to_value(&observation).unwrap();
    assert_eq!(json["frame_index"], 7);
    assert_eq!(json["distance_band"], "mid");
    assert_eq!(json["horizontal_order"], "p1_left");
    assert!(json["motion_regions"].is_array());
    assert_eq!(
        serde_json::from_value::<SpatialObservation>(json).unwrap(),
        observation
    );
}
