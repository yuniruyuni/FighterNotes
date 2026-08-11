use super::super::Frame;

#[test]
fn frame_coordinates_are_relative_to_the_strip_origin() {
    let rgba = [11, 22, 33, 255];
    let frame = Frame::new(&rgba, 1, 7, 210);

    assert_eq!(frame.px(0, 7), Some((11, 22, 33)));
    assert_eq!(frame.px(0, 6), None);
    assert_eq!(frame.px(0, 8), None);
}

#[test]
fn white_threshold_and_incomplete_pixels_have_exact_boundaries() {
    let rgba = [100, 100, 100, 255];
    let frame = Frame::new(&rgba, 1, 0, 150);
    assert!(!frame.is_white(0, 0));

    let incomplete = Frame::new(&rgba[..2], 1, 0, 0);
    assert_eq!(incomplete.px(0, 0), None);
}
