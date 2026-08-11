//! FrameFeatures struct と HP バー・ドライブゲージ検出。
//!
//! Python `sf6_video.py` の FrameFeatures dataclass および
//! `hp_bar_score` / `hp_fill_ratio` / `bar_fill_ratio` 関数を移植。

/// HUD ストリップの開始 Y 座標（1920x1080 基準）。HP バー・ドライブゲージを含む。
pub const HUD_STRIP_Y: u32 = 64;
/// HUD ストリップの高さ（HP bar y=64-96, Drive gauge y=112-134 を含む最小行数）
pub const HUD_STRIP_H: u32 = 70;

/// HP バー ROI（1920×1080 基準）: (x1, x2, y1, y2)
///
/// x2/x1 は HP ゲージ白枠の外縁（上端行）に合わせた値。
/// スキャンは平行四辺形境界（classify_hp_col が画像バウンドのみ使用）のため、
/// 斜め列の底部ピクセルは x2/x1 を超えて正しくスキャンされる。
pub const HP_ROI_P1: (u32, u32, u32, u32) = (172, 853, 64, 95);
pub const HP_ROI_P2: (u32, u32, u32, u32) = (1067, 1748, 64, 95);
/// 1 フレーム分の解析特徴量。
///
/// Python 版の `FrameFeatures` の簡略版。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FrameFeatures {
    pub frame_index: u32,
    pub fps: f32,
    /// 自分の HP 充填率（0.0–1.0）— finish() 後は遡及補正済み
    pub own_hp: f32,
    /// 相手の HP 充填率（0.0–1.0）— finish() 後は遡及補正済み
    pub opponent_hp: f32,
    /// 試合画面かどうか
    pub is_match_screen: bool,
    /// 自分のメーター状態（meter-tracker から取得）
    pub own_meter_state: Option<String>,
    /// 相手のメーター状態（meter-tracker から取得）
    pub opponent_meter_state: Option<String>,
    /// 左 HP バー検出スコア（sat>45 & val>80 の画素割合）
    pub left_hp_score: f32,
    /// 右 HP バー検出スコア
    pub right_hp_score: f32,
    /// 左ドライブゲージ充填率（0.0–1.0 = 0〜6 セルを正規化）
    pub left_drive_ratio: f32,
    /// 右ドライブゲージ充填率（0.0–1.0 = 0〜6 セルを正規化）
    pub right_drive_ratio: f32,
    /// 左プレイヤーがバーンアウト中か（true の間 left_drive_ratio は回復進捗 0.0–1.0）
    #[serde(default)]
    pub left_burnout: bool,
    /// 右プレイヤーがバーンアウト中か
    #[serde(default)]
    pub right_burnout: bool,
    /// 左ドライブ読み取りが不確実（遮蔽・状態遷移フラッシュ）
    #[serde(default)]
    pub left_drive_uncertain: bool,
    /// 右ドライブ読み取りが不確実
    #[serde(default)]
    pub right_drive_uncertain: bool,
    /// 左 SA ゲージ（0.0〜3.0、部分ストックを含む）
    #[serde(default)]
    pub left_super_value: f32,
    /// 右 SA ゲージ（0.0〜3.0、部分ストックを含む）
    #[serde(default)]
    pub right_super_value: f32,
    /// 左 SA ラベルを単フレームで確定できない
    #[serde(default = "default_true")]
    pub left_super_uncertain: bool,
    /// 右 SA ラベルを単フレームで確定できない
    #[serde(default = "default_true")]
    pub right_super_uncertain: bool,
    /// 左プレイヤーが CA 使用可能表示
    #[serde(default)]
    pub left_ca_ready: bool,
    /// 右プレイヤーが CA 使用可能表示
    #[serde(default)]
    pub right_ca_ready: bool,
    /// 左 HP 生値（単調制約・遡及補正前の hp_fill_ratio 直値）
    pub left_hp_raw: f32,
    /// 右 HP 生値（単調制約・遡及補正前の hp_fill_ratio 直値）
    pub right_hp_raw: f32,
    /// 左 HP 品質（0.0=確実, 1.0=疑問: アイランド検出）
    /// HP 領域境界の外側に HP カラー列が存在するとき 1.0 をセット。
    /// キャラクタースプライトが空き領域に重なったフレームを示す。
    #[serde(default)]
    pub left_hp_raw_quality: f32,
    /// 右 HP 品質（0.0=確実, 1.0=疑問: アイランド検出）
    #[serde(default)]
    pub right_hp_raw_quality: f32,
}

fn default_true() -> bool {
    true
}

// -------------------------------------------------------------------------
// ユーティリティ
// -------------------------------------------------------------------------

/// 1920x1080 基準の ROI 座標を実フレーム解像度にスケーリングする。
pub(crate) fn scale_roi(
    x1_base: u32,
    x2_base: u32,
    y1_base: u32,
    y2_base: u32,
    width: u32,
    height: u32,
) -> (u32, u32, u32, u32) {
    let sx = width as f32 / 1920.0;
    let sy = height as f32 / 1080.0;
    let x1 = ((x1_base as f32 * sx) as u32).min(width.saturating_sub(1));
    let x2 = ((x2_base as f32 * sx) as u32).min(width);
    let y1 = ((y1_base as f32 * sy) as u32).min(height.saturating_sub(1));
    let y2 = ((y2_base as f32 * sy) as u32).min(height);
    (x1, x2, y1, y2)
}

/// RGBA フレーム上の斜めバー領域。
///
/// HP とドライブの列分類は同じ座標変換を使うため、画像と ROI の対応を
/// この値オブジェクトに集約する。
pub(crate) struct SlantedRoi<'a> {
    pub(crate) rgba: &'a [u8],
    pub(crate) frame_width: usize,
    pub(crate) x: std::ops::Range<usize>,
    pub(crate) y_start: usize,
    pub(crate) height: usize,
    pub(crate) strip_y: usize,
    pub(crate) slope: f32,
}

impl SlantedRoi<'_> {
    /// 列 `column` の行 `row` が、画面のどの x に当たるか。
    ///
    /// バーは平行四辺形なので、行が下がるごとに横へずれる。ずれた先が
    /// ROI の外へ出る行は、その列には属さない。
    pub(crate) fn column_x(&self, column: usize, row: usize, slope_origin: usize) -> Option<usize> {
        let relative_row = row.checked_sub(slope_origin)?;
        let x_offset = (relative_row as f32 * self.slope).round() as i32;
        let x = self.x.start as i32 + column as i32 + x_offset;
        (x >= self.x.start as i32 && (x as usize) < self.x.end).then_some(x as usize)
    }

    /// 列 `column` の行 `row` の画素。ROI の外へ出た行と、バッファの
    /// 終わりを越える行は None。
    pub(crate) fn rgb_at(
        &self,
        column: usize,
        row: usize,
        slope_origin: usize,
    ) -> Option<[f32; 3]> {
        let x = self.column_x(column, row, slope_origin)?;
        let y = self.y_start.checked_add(row)?.checked_sub(self.strip_y)?;
        let index = (y * self.frame_width + x) * 4;
        let pixel = self.rgba.get(index..)?.first_chunk::<3>()?;
        Some([pixel[0] as f32, pixel[1] as f32, pixel[2] as f32])
    }
}

/// 色相判定は入力履歴の読み取りと共有する。経路を変えずに済むよう、
/// ここからそのまま再輸出する。
pub(crate) use pixel_color::rgb_to_hsv;

// ── サブモジュール（frame_features.rs 3,019 行からの分割。公開 API 不変）──

mod debug_json;
mod drive_gauge;
mod hp_bar;
mod hp_correct;
mod super_gauge;

pub use debug_json::*;
pub use drive_gauge::*;
pub use hp_bar::*;
pub use hp_correct::*;
pub use super_gauge::*;

#[cfg(test)]
mod tests;
