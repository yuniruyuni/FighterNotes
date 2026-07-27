use super::super::super::HorizontalOrder;

pub(super) fn is_forward(side: u8, input_dir: &str, order: Option<HorizontalOrder>) -> bool {
    matches!(
        (side, input_dir, order),
        (1, "R", Some(HorizontalOrder::P1Left))
            | (1, "L", Some(HorizontalOrder::P1Right))
            | (2, "L", Some(HorizontalOrder::P1Left))
            | (2, "R", Some(HorizontalOrder::P1Right))
    )
}
