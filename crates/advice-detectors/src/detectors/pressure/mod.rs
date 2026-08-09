mod advantage;
mod common;
mod mashing;
mod minus;

pub use advantage::detect_advantage_abandoned;
pub use mashing::detect_mashing;
pub use minus::{detect_press_while_minus, detect_throw_while_minus};
