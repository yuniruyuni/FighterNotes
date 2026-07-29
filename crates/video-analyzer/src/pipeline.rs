//! 知覚後のパイプライン一括実行（確定層 → イベント層 → レポート）。
//!
//! wasm-bridge の `finish()` と CLI（examples/analyze_stacked）が共用する
//! 唯一の入口。処理列をここに集約することで、確定層の順序や引数の
//! 食い違いによる「wasm と CLI で結果が違う」事故を防ぐ。

use crate::advice::{self, AdviceReport};
use crate::context::AnalysisContext;
use crate::frame_features::FrameFeatures;
use crate::input_tracker::TrackedInput;
use crate::match_events;
use crate::round_start::FightMarker;

/// 確定層: 知覚層の per-frame 読みを時間方向にクリーニングして確定させる。
///
/// ここで確定した値が viewer の表示とイベント層の入力の唯一の源になる。
/// 順序に意味がある: HP の確定（遮蔽埋め・急落棄却・単調化）を先に、
/// ドライブの短期偽値排除を後に行う。
pub fn finalize_features(features: &mut Vec<FrameFeatures>) {
    crate::temporal::confirm_hp(features);
    crate::temporal::clean_drive_temporal(features);
}

/// Browser の `FIGHT` 画像検出を境界の唯一の決定信号として確定層を実行する。
pub fn finalize_features_with_fight_markers(
    features: &mut Vec<FrameFeatures>,
    markers: &[FightMarker],
    own_side: &str,
) {
    crate::temporal::confirm_hp_with_fight_markers(features, markers, own_side);
    crate::temporal::clean_drive_temporal(features);
}

/// フレームフィーチャ列と入力トラッカー出力からアドバイスレポートを生成する。
///
/// `p1_inputs` / `p2_inputs` は `repair_row0_sequence` の出力（features と
/// 同数・同順）。入力読み取りが無いパイプラインでは空スライスでよい。
/// `meter` はフレームメーターの確定タイムライン（P1, P2）。None なら
/// コンタクト検出・stun ゲートは HP ベースの近似にフォールバックする。
/// `own_char` はユーザー入力の自キャラ名（確反提案の技名列挙に使用。不明なら None）。
pub fn analyze_match(
    features: &[FrameFeatures],
    p1_inputs: &[TrackedInput],
    p2_inputs: &[TrackedInput],
    meter: Option<(&meter_tracker::MeterTimeline, &meter_tracker::MeterTimeline)>,
    own_side: &str,
    own_char: Option<&str>,
) -> AdviceReport {
    let context = AnalysisContext::from_characters(own_side, own_char, None);
    analyze_match_with_context(features, p1_inputs, p2_inputs, meter, &context)
}

/// Context-aware entry point for character-specific event detection.
pub fn analyze_match_with_context(
    features: &[FrameFeatures],
    p1_inputs: &[TrackedInput],
    p2_inputs: &[TrackedInput],
    meter: Option<(&meter_tracker::MeterTimeline, &meter_tracker::MeterTimeline)>,
    context: &AnalysisContext,
) -> AdviceReport {
    let events = match_events::build_match_events_with_context(
        features, p1_inputs, p2_inputs, meter, context,
    );
    advice::build_report_with_context(features, &events, context)
}

/// フレームフィーチャ列のみからアドバイスレポートを生成する（互換）。
pub fn analyze_features(features: &[FrameFeatures], own_side: &str) -> AdviceReport {
    analyze_match(features, &[], &[], None, own_side, None)
}

/// フレームフィーチャ列のみを context 付きで解析する。
pub fn analyze_features_with_context(
    features: &[FrameFeatures],
    context: &AnalysisContext,
) -> AdviceReport {
    analyze_match_with_context(features, &[], &[], None, context)
}
