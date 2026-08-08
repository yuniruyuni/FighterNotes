//! 解析の前提となる静的な情報。
//!
//! どちらも上流の観測に依存せず、解析の全段から参照される。
//!   - `context`: どちらが自分か、両者のキャラクターは何か
//!   - `frame_data`: 技のフレームデータ表（確反候補と攻撃属性）
//!
//! モジュール名は移設前と同じにしてある。`video-analyzer` 側が
//! `crate::context` / `crate::frame_data` として再輸出するため、
//! 呼び出し側の経路は変わらない。

pub mod context;
pub mod frame_data;
