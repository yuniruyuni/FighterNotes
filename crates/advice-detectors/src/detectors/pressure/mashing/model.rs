use crate::match_events::{DamageEvent, InputSegment};

pub struct MashHit {
    pub press_frame: u32,
    pub damage_end_frame: u32,
    pub round_no: u32,
    pub drop: f32,
    pub meter_confirmed: bool,
    pub input: String,
}

impl MashHit {
    pub fn new(press: &InputSegment, damage: &DamageEvent, meter_confirmed: bool) -> Self {
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
