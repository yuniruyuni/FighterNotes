mod advantage;
mod common;
mod mashing;
mod minus;

pub(crate) use advantage::detect_advantage_abandoned;
pub(crate) use mashing::detect_mashing;
pub(crate) use minus::{detect_press_while_minus, detect_throw_while_minus};
