//! アドバイスレポート生成（フェーズ 2: 弱点検出器）。
//!
//! イベント層（match_events）の出力から「指摘カード」を生成する。
//! 各カードは頻度・重大度・証拠フレーム・SF6 上の根拠・練習メニューを持つ。
//!
//! 検出器の設計方針（ユーザー指定）:
//!   - DI 反応一般は指摘しない。通常技の実行まで確認できたDI被弾だけ、
//!     技を置く距離・タイミングを確認する場面として提示する
//!   - 投げ被弾は「読み合いの混合が崩れている」とき（同一結果の連続）のみ指摘
//!   - 暴れは「被圧状態でのボタン → 大被弾」のみ指摘（差し合い負けと区別）
//!   - 対空率・バーンアウト管理・入力習慣統計・ラウンド構造は常に分析
//!   - 確反のような確定状況は 1 件から診断できるが、読み合いを含む行動は
//!     単発の負けを指摘しない。同一回答の反復、選択率の偏り、複数回の損失が
//!     揃った場合だけ原因診断とする。結果だけの場面一覧は Observation にする。
//!   - 原因診断・事実確認・統計の順、同種では証拠確度とダメージ総量
//!     （severity）の順で提示。

use crate::frame_data;
use crate::frame_features::FrameFeatures;
#[cfg(test)]
use crate::match_events::{
    DefenseResponseKind, DpReachability, PunishOutcome, PunishReachability, TeleportContext,
    ThreatOutcome,
};
use crate::match_events::{
    DefensiveActionKind, EventConfidence, JumpDirection, JumpOutcome, MatchEvents,
    MinusPressOutcome,
};
mod builder;
mod cards;
mod coverage;
mod damage_origins;
mod detectors;
mod model;
mod parameters;
mod stats;
mod summaries;

pub use builder::{build_report, build_report_with_context};
pub use model::*;
pub use parameters::RULESET_VERSION;

pub(crate) use cards::build_advice_cards;
pub(crate) use coverage::build_coverage;
pub(crate) use detectors::*;
pub(crate) use parameters::*;
pub(crate) use stats::*;
pub(crate) use summaries::*;

#[cfg(test)]
mod tests;
