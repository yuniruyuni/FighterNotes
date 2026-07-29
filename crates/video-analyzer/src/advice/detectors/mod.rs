//! 指摘カードを、検出対象と判定責務ごとに構成する。

mod big_hits;
mod burnout;
mod defense;
mod direction;
mod jumps;
mod pressure;
mod punishment;
mod rounds;
mod throw_loop;
mod throw_whiff;

pub(crate) use big_hits::detect_big_hits;
pub(crate) use burnout::detect_burnout;
pub(crate) use defense::{detect_guard_break, detect_layered_defense, detect_teleport_defense};
pub(crate) use direction::dir_arrow;
pub(crate) use jumps::{detect_anti_air, detect_own_jumps};
pub(crate) use pressure::{detect_mashing, detect_press_while_minus, detect_throw_while_minus};
pub(crate) use punishment::{
    detect_low_conversion, detect_punish_fail, detect_punish_missed, detect_reversal_punished,
};
pub(crate) use rounds::{detect_early_hits, detect_lead_loss};
pub(crate) use throw_loop::detect_throw_loop;
pub(crate) use throw_whiff::detect_throw_whiff_punished;

#[cfg(test)]
mod tests;
