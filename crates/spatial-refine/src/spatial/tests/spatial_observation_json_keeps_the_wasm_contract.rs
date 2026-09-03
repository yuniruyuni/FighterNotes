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
        contact: Some(ContactObservation {
            center: SpatialPoint::new(0.5, 0.6),
            bounds: SpatialRect::new(0.45, 0.55, 0.55, 0.65),
            effect_cells: 9,
            confidence: 0.7,
        }),
        camera: Some(CameraMotion {
            pan_dx: -0.002,
            zoom_ratio: 1.001,
            confidence: 0.75,
        }),
    };

    let json = serde_json::to_value(&observation).unwrap();
    assert_eq!(json["frame_index"], 7);
    assert_eq!(json["distance_band"], "mid");
    assert_eq!(json["horizontal_order"], "p1_left");
    assert!(json["motion_regions"].is_array());
    assert_eq!(json["contact"]["effect_cells"], 9);
    assert_eq!(json["camera"]["confidence"], 0.75);
    assert_eq!(
        serde_json::from_value::<SpatialObservation>(json).unwrap(),
        observation
    );

    // 旧 JSON(contact なし)も読めることを固定する。
    let mut legacy = serde_json::to_value(&observation).unwrap();
    legacy.as_object_mut().unwrap().remove("contact");
    let parsed = serde_json::from_value::<SpatialObservation>(legacy).unwrap();
    assert_eq!(parsed.contact, None);
}
