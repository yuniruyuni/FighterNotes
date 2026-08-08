//! 入力履歴欄の画素読み取りと、その系列の補修。
//!
//! 画面右端の入力履歴は方向記号・ボタン記号・フレーム数の桁で構成される。
//! ここでは 1 フレーム分の読み取り（`input_history`）と、読み落としを
//! 前後の行から埋め戻す処理（`input_tracker`）だけを扱う。
//!
//! モジュール名は移設前と同じにしてある。`video-analyzer` 側が
//! `crate::input_history` として再輸出するため、呼び出し側の経路は変わらない。

pub mod input_history;
pub mod input_tracker;

#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
