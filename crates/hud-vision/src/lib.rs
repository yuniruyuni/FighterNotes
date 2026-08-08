//! HUD の画素読み取り。
//!
//! HP バー・ドライブゲージ・SA ゲージ・ラウンド開始の `FIGHT` 表示は
//! いずれも画面上の決まった位置の画素をそのまま読む処理で、上流の解析結果に
//! 依存しない。ここから先（時系列の確定、イベント化、アドバイス）は
//! `video-analyzer` が担う。
//!
//! モジュール名は移設前と同じにしてある。`video-analyzer` 側が
//! `crate::frame_features` として再輸出するため、呼び出し側の経路は変わらない。

pub mod frame_features;
pub mod round_start;
