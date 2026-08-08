//! 空間再評価のテストで使う観測列の組み立て補助。
//!
//! 空間で詰めた結果が助言としてどう出るかまで見るテストは上位 crate に
//! 置くため、`test-support` feature で公開する。

mod events;
mod frames;

pub use events::*;
pub use frames::*;
