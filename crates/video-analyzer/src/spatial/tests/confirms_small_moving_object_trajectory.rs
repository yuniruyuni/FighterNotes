use super::*;

#[test]
fn confirms_small_moving_object_trajectory() {
    let mut extractor = SpatialExtractor::new(test_config());
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    extractor
        .observe_rgba(200, &first, WIDTH, HEIGHT, hints())
        .unwrap();

    let mut second = first.clone();
    rect(&mut second, 190, 96, 12, 8, [240, 90, 20]);
    let first_object = extractor
        .observe_rgba(201, &second, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    assert!(first_object
        .projectile_candidates
        .iter()
        .any(|candidate| !candidate.trajectory_confirmed));
    assert!(
        first_object
            .motion_regions
            .iter()
            .any(|region| region.effect_color_fraction >= 0.5),
        "{:?}",
        first_object.motion_regions
    );

    let mut third = first.clone();
    rect(&mut third, 174, 96, 12, 8, [240, 90, 20]);
    let second_object = extractor
        .observe_rgba(202, &third, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    let projectile = second_object
        .projectile_candidates
        .iter()
        .find(|candidate| candidate.trajectory_confirmed)
        .expect("two observations confirm a moving-object trajectory");
    assert_eq!(projectile.motion, HorizontalMotion::Left);
    assert!(projectile.velocity_x.unwrap() < 0.0);
    assert!(projectile.center.x > 0.45 && projectile.center.x < 0.65);
}
