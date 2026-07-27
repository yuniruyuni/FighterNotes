use super::super::*;

#[test]
fn digit_templates_are_separable() {
    for (a, (_, means)) in DIGIT_NCC.iter().enumerate() {
        let mut rgba = vec![0u8; DIGIT_W * DIGIT_H * 4];
        for (y, row) in means.iter().enumerate() {
            for (x, &value) in row.iter().enumerate() {
                let index = (y * DIGIT_W + x) * 4;
                rgba[index] = value;
                rgba[index + 1] = value;
                rgba[index + 2] = value;
                rgba[index + 3] = 255;
            }
        }
        let frame = Frame {
            rgba: &rgba,
            w: DIGIT_W,
            y_off: 0,
            white_th: 210,
        };
        let (digit, score, margin) = match_digit_gray(&frame, 0, 0);
        assert_eq!(digit, a as u32, "digit {a} was classified as {digit}");
        assert!(score <= 5, "digit {a} self-score was {score}");
        assert!(
            margin >= DIGIT_AMBIG_MARGIN,
            "digit {a} was classified as ambiguous"
        );
    }
}

#[test]
fn direction_templates_are_separable() {
    for a in 0..9 {
        for b in (a + 1)..9 {
            let ca = mask_centroid(&DIR_TEMPLATES[a]).unwrap();
            let cb = mask_centroid(&DIR_TEMPLATES[b]).unwrap();
            let dx = (ca.0 - cb.0).round() as i32;
            let dy = (ca.1 - cb.1).round() as i32;
            let shifted = shift_mask(&DIR_TEMPLATES[b], dx, dy);
            let distance = glyph_distance(&DIR_TEMPLATES[a], &shifted, DIR_W as u32);
            assert!(
                distance > DIR_MIN_MARGIN,
                "directions {:?} and {:?} are only {distance} apart",
                DIR_ORDER[a],
                DIR_ORDER[b]
            );
        }
    }
}

#[test]
fn ncc_models_have_enough_pixels() {
    for (digit, (mask, _)) in DIGIT_NCC.iter().enumerate() {
        let pixels: u32 = mask.iter().map(|row| row.count_ones()).sum();
        assert!(
            pixels >= 40,
            "digit {digit} has only {pixels} stable pixels"
        );
    }
}
