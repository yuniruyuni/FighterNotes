use crate::match_events::{DamageEvent, InputSegment};

pub(super) struct MashHit {
    pub(super) press_frame: u32,
    pub(super) damage_end_frame: u32,
    pub(super) round_no: u32,
    pub(super) drop: f32,
    pub(super) meter_confirmed: bool,
    pub(super) input: String,
}

impl MashHit {
    pub(super) fn new(press: &InputSegment, damage: &DamageEvent, meter_confirmed: bool) -> Self {
        let input = if press.badges.is_empty() {
            if press.auto { "AUTO" } else { "ボタン" }.to_string()
        } else {
            press.badges.join("+")
        };
        Self {
            press_frame: press.start_frame,
            damage_end_frame: damage.end_frame,
            round_no: damage.round_no,
            drop: damage.drop,
            meter_confirmed,
            input,
        }
    }
}
