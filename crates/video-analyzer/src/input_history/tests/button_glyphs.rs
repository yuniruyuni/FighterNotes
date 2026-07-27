use super::super::*;

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
