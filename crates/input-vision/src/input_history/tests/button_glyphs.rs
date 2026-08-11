use super::super::button_glyphs::{
    glyph_score_is_accepted, glyph_score_margin, has_glyph_light_hole, is_glyph_dark,
};
use super::super::*;

#[test]
fn glyph_pixel_and_score_gates_include_their_exact_edges() {
    assert!(is_glyph_dark(99, 180, 180));
    assert!(!is_glyph_dark(100, 180, 180));
    assert!(has_glyph_light_hole(100, 40));
    assert!(!has_glyph_light_hole(100, 41));
    assert!(glyph_score_is_accepted(45, 3));
    assert!(!glyph_score_is_accepted(46, 3));
    assert!(!glyph_score_is_accepted(45, 2));
    assert_eq!(glyph_score_margin(12, 20), 8);
    assert_eq!(glyph_score_margin(20, 12), 8);
}

#[test]
fn button_glyph_templates_classify() {
    for (template, expected) in [
        (&BTN_GLYPH_PUNCH, BtnGlyph::Punch),
        (&BTN_GLYPH_KICK, BtnGlyph::Kick),
    ] {
        let mut rgba = vec![160u8; BTN_GLYPH_W as usize * DIGIT_H * 4];
        for alpha in rgba[3..].iter_mut().step_by(4) {
            *alpha = 255;
        }
        for (y, bits) in template.iter().enumerate() {
            for x in 0..BTN_GLYPH_W as usize {
                if bits & (1 << x) != 0 {
                    let index = (y * BTN_GLYPH_W as usize + x) * 4;
                    rgba[index..index + 3].copy_from_slice(&[0, 180, 180]);
                }
            }
        }
        let frame = Frame {
            rgba: &rgba,
            w: BTN_GLYPH_W as usize,
            y_off: 0,
            white_th: 210,
        };
        assert_eq!(classify_btn_glyph(&frame, 0, 0), Some(expected));
    }
}

#[test]
fn button_glyph_alignment_recovers_a_circle_joined_to_background() {
    let span_w = BTN_GLYPH_W as usize + 13;
    let glyph_x = 5usize;
    let mut rgba = vec![160u8; span_w * DIGIT_H * 4];
    for alpha in rgba[3..].iter_mut().step_by(4) {
        *alpha = 255;
    }
    for (y, bits) in BTN_GLYPH_KICK.iter().enumerate() {
        for x in 0..BTN_GLYPH_W as usize {
            if bits & (1 << x) != 0 {
                let index = (y * span_w + glyph_x + x) * 4;
                rgba[index..index + 3].copy_from_slice(&[0, 180, 180]);
            }
        }
    }
    let frame = Frame {
        rgba: &rgba,
        w: span_w,
        y_off: 0,
        white_th: 210,
    };

    assert_eq!(
        classify_btn_glyph_in_span(&frame, 0, 0, span_w),
        Some(BtnGlyph::Kick)
    );
}

#[test]
fn button_glyph_reading_honors_both_coordinate_offsets() {
    let width = BTN_GLYPH_W as usize + 10;
    let height = DIGIT_H * 2 + 8;
    let (glyph_x, glyph_y) = (4, DIGIT_H + 3);
    let mut rgba = vec![160u8; width * height * 4];
    for alpha in rgba[3..].iter_mut().step_by(4) {
        *alpha = 255;
    }
    for (y, bits) in BTN_GLYPH_PUNCH.iter().enumerate() {
        for x in 0..BTN_GLYPH_W as usize {
            if bits & (1 << x) != 0 {
                let index = ((glyph_y + y) * width + glyph_x + x) * 4;
                rgba[index..index + 3].copy_from_slice(&[0, 180, 180]);
            }
        }
    }
    let frame = Frame::new(&rgba, width, 0, 210);

    assert_eq!(
        classify_btn_glyph(&frame, glyph_x, glyph_y),
        Some(BtnGlyph::Punch)
    );
    assert_eq!(
        classify_btn_glyph_in_span(&frame, glyph_x, glyph_y, BTN_GLYPH_W as usize),
        Some(BtnGlyph::Punch)
    );
}

#[test]
fn classic_button_labels() {
    use BtnGlyph::*;
    let mark = |color, glyph| BadgeMark {
        color,
        boxed: false,
        glyph: Some(glyph),
    };
    assert_eq!(mark(BadgeColor::Green, Punch).label(), "弱P");
    assert_eq!(mark(BadgeColor::Yellow, Punch).label(), "中P");
    assert_eq!(mark(BadgeColor::Red, Punch).label(), "強P");
    assert_eq!(mark(BadgeColor::Green, Kick).label(), "弱K");
    assert_eq!(mark(BadgeColor::Yellow, Kick).label(), "中K");
    assert_eq!(mark(BadgeColor::Red, Kick).label(), "強K");

    let plain = BadgeMark {
        color: BadgeColor::Green,
        boxed: false,
        glyph: None,
    };
    assert_eq!(plain.label(), "弱");
    let drive_impact = BadgeMark {
        color: BadgeColor::Green,
        boxed: true,
        glyph: None,
    };
    assert_eq!(drive_impact.label(), "DI");
}

#[test]
fn classic_throw_pair() {
    use BtnGlyph::*;
    let classic = |color, glyph| BadgeMark {
        color,
        boxed: false,
        glyph: Some(glyph),
    };
    assert!(classic_throw(&[
        classic(BadgeColor::Green, Punch),
        classic(BadgeColor::Green, Kick)
    ]));
    assert!(classic_throw(&[
        classic(BadgeColor::Green, Punch),
        classic(BadgeColor::Yellow, Punch),
        classic(BadgeColor::Green, Kick),
    ]));
    assert!(!classic_throw(&[classic(BadgeColor::Green, Punch)]));
    assert!(!classic_throw(&[
        classic(BadgeColor::Green, Punch),
        classic(BadgeColor::Yellow, Kick)
    ]));
    assert!(!classic_throw(&[
        classic(BadgeColor::Yellow, Punch),
        classic(BadgeColor::Yellow, Kick)
    ]));

    let modern = |color| BadgeMark {
        color,
        boxed: false,
        glyph: None,
    };
    assert!(!classic_throw(&[
        modern(BadgeColor::Green),
        modern(BadgeColor::Green)
    ]));
}

#[test]
fn button_glyph_templates_are_sane() {
    let mut punch_pixels = 0u32;
    let mut kick_pixels = 0u32;
    let mut differing_pixels = 0u32;
    for index in 0..18 {
        assert_eq!(BTN_GLYPH_PUNCH[index] & !BTN_GLYPH_INTERIOR[index], 0);
        assert_eq!(BTN_GLYPH_KICK[index] & !BTN_GLYPH_INTERIOR[index], 0);
        punch_pixels += BTN_GLYPH_PUNCH[index].count_ones();
        kick_pixels += BTN_GLYPH_KICK[index].count_ones();
        differing_pixels += (BTN_GLYPH_PUNCH[index] ^ BTN_GLYPH_KICK[index]).count_ones();
    }
    assert!(
        punch_pixels >= 80,
        "punch template has only {punch_pixels} pixels"
    );
    assert!(
        kick_pixels >= 80,
        "kick template has only {kick_pixels} pixels"
    );
    assert!(
        differing_pixels >= 60,
        "P/K templates differ by only {differing_pixels} pixels"
    );
}
