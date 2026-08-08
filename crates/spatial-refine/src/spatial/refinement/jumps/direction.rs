use super::super::super::HorizontalOrder;
use crate::match_events::JumpDirection;

pub(super) fn resolve(side: u8, input_dir: &str, order: Option<HorizontalOrder>) -> JumpDirection {
    if input_dir == "U" {
        return JumpDirection::Neutral;
    }
    let input_right = match input_dir {
        "UR" => true,
        "UL" => false,
        _ => return JumpDirection::Unknown,
    };
    let forward_is_right = match (side, order) {
        (1, Some(HorizontalOrder::P1Left)) | (2, Some(HorizontalOrder::P1Right)) => true,
        (1, Some(HorizontalOrder::P1Right)) | (2, Some(HorizontalOrder::P1Left)) => false,
        _ => return JumpDirection::Unknown,
    };
    if input_right == forward_is_right {
        JumpDirection::Forward
    } else {
        JumpDirection::Backward
    }
}
