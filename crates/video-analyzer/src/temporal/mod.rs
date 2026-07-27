//! フレームフィーチャ列の時間方向クリーニング（確定層）。
//!
//! wasm-bridge の finish() と CLI の e2e 検証が共用する。ここで確定した値が
//! viewer の表示とイベント層（match_events）の入力の唯一の源になる。

mod drive;
mod hp;

pub use drive::clean_drive_temporal;
pub use hp::{confirm_hp, FULL_HP, FULL_MIN_RUN};

#[cfg(test)]
mod tests;
