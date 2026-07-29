//! 入力履歴表示の読み取り。
//!
//! SF6 の画面左右端に表示される入力履歴（コントローラ入力 + 継続フレーム数）を
//! 行単位で構造化して読み取る。
//!
//! 表示仕様（1920x1080の実ゲーム撮影動画で実測・較正）:
//!   - P1: 画面左端 / P2: 画面右端（左右ミラー配置）
//!   - 先頭行（最新入力）y=236、行ピッチ 34px、可視 約18 行
//!   - P1 行内: カウント数字（右揃え、右端 x=79）| 方向グリフ（x=86-126）|
//!     ボタンバッジ（色付き円 / AUTO 白ボックス、x=122-186）
//!   - P2 は完全ミラー（バッジ | 方向 | 数字。数字右端 x=1865）
//!   - 先頭行のカウントは 1 動画フレームごとに +1（60fps 表示 = ゲームフレーム 1:1）
//!   - 入力変化で全行が 1 行下にシフトし count=1 の新行が先頭に入る
//!   - パネルは半透明でステージ背景が透ける。コンボカウンター（"n HITS"）が
//!     P1 側の行 5-7 付近に重なることがある

include!("templates.rs");

// ── ジオメトリ（1920x1080 基準） ─────────────────────────────────────────────

/// 先頭行の上端 y
const ROW0_Y: u32 = 236;
/// 行ピッチ
const ROW_PITCH: u32 = 34;
/// 読み取る行数
pub const INPUT_ROWS: usize = 18;

/// 入力ストリップ（先頭行のみ）の切り出し範囲。
/// 行 0 の構成要素: 方向グリフ y=234-260、数字/バッジ y=236-254。
/// 上下に 2px マージンを取る
pub const INPUT_STRIP_Y: u32 = 232;
pub const INPUT_STRIP_H: u32 = 36;

/// 数字ボックス: 幅 11、高さ 18。ones 桁の左端 x（P1）。桁は左へ 11px ずつ
const DIGIT_W: usize = 11;
const DIGIT_H: usize = 18;
const P1_ONES_X: u32 = 68;
const P2_ONES_X: u32 = 1856;
/// 読む最大桁数（実測最大 3 桁 = 999 フレーム ≈ 16 秒）
const MAX_DIGITS: usize = 3;

/// 方向グリフボックス: 幅 40、高さ 26（行上端の 2px 上から）
const DIR_W: usize = 40;
const DIR_H: usize = 26;
const P1_DIR_X: u32 = 86;
const P2_DIR_X: u32 = 1794;
const DIR_Y_OFF: i32 = -2;

/// バッジ帯（色付き円 / 文字付き色箱）。全ボタン同時押し（最大 8 個
/// ≈ 28px ピッチ × 8 = 224px）まで収容できる幅を取る
const P1_BADGE_X: (u32, u32) = (122, 350);
const P2_BADGE_X: (u32, u32) = (1568, 1796);
/// AUTO 箱・投げ円が現れる近傍帯（方向グリフ寄りの先頭スロット群）。
/// 無彩色検出は絶対量ベースのため、広帯だと背景ノイズが蓄積する
const P1_MONO_X: (u32, u32) = (122, 186);
const P2_MONO_X: (u32, u32) = (1730, 1796);

mod badges;
mod button_glyphs;
mod debug;
mod digits;
mod direction;
mod mask;
mod model;
mod pixel;
mod reader;

pub use button_glyphs::classic_throw;
pub use debug::input_history_debug_json;
pub use model::{BadgeColor, BadgeMark, BtnGlyph, InputDir, InputRow};
pub use reader::{read_input_row0_from_strip, read_input_rows};

use badges::read_badges;
use button_glyphs::classify_btn_glyph_in_span;
#[cfg(test)]
use button_glyphs::{classify_btn_glyph, BTN_GLYPH_W};
#[cfg(test)]
use digits::match_digit_gray;
use digits::read_count;
#[cfg(test)]
use digits::DIGIT_AMBIG_MARGIN;
use direction::read_dir;
#[cfg(test)]
use direction::{mask_centroid, shift_mask, DIR_MIN_MARGIN};
use mask::glyph_distance;
use model::DIR_ORDER;
use pixel::Frame;

#[cfg(test)]
mod tests;
