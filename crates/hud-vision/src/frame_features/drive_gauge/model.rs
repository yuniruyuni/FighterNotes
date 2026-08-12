/// ドライブゲージ斜め列 1 本の色カテゴリ。
///
/// 空セル領域は半透明でステージ背景が透けるため「空 = 暗い」は成立しない。
/// 信頼できるのは点灯セルの超高彩度と、不透明なバーンアウト回復バーのみ。
/// それ以外の低情報列はすべて Rest に落とす。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DriveColClass {
    Lit,     // 点灯セル（黄〜緑グラデ + 橙警告: H 15-60, S>120, V>120）
    Gray,    // バーンアウト回復バー（S<60, 120<V<210）
    Foreign, // ゲージ外の高彩度色（スプライト遮蔽: S>120, V>120, H が 15-60 外）
    Rest,    // その他（空セル・背景透け・白フラッシュ・暗部）
    Outside, // ROI に収まる行が足りず、バーの測定になっていない列
}

/// ドライブゲージ読み取り結果。
#[derive(Debug, Clone, Copy)]
pub struct DriveGaugeRead {
    /// ゲージ値 0.0〜6.0（部分セルも連続値で反映）。バーンアウト中は 0.0
    pub value: f32,
    /// バーンアウト中か
    pub burnout: bool,
    /// バーンアウト回復進捗 0.0〜1.0（burnout=true のときのみ有効）
    pub recovery: f32,
    /// 読み取り不確実（遮蔽・状態遷移フラッシュ・HUD 消失）
    pub uncertain: bool,
}
