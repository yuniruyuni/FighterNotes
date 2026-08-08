use super::support::*;

#[test]
fn hp_zone_segmentation_preserves_empty_single_and_transition_boundaries() {
    use HpColColor::*;

    assert!(segment_zones(&[]).is_empty());

    let single = segment_zones(&[Fill, Fill, Fill]);
    assert_eq!(single.len(), 1);
    assert_eq!(single[0].color, Fill);
    assert_eq!((single[0].start, single[0].end), (0, 2));

    let zones = segment_zones(&[White, White, Fill, Dark, Dark]);
    assert_eq!(zones.len(), 3);
    assert_eq!(
        (zones[0].color, zones[0].start, zones[0].end),
        (White, 0, 1)
    );
    assert_eq!((zones[1].color, zones[1].start, zones[1].end), (Fill, 2, 2));
    assert_eq!((zones[2].color, zones[2].start, zones[2].end), (Dark, 3, 4));
}
