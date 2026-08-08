/// 方向入力（テンキー表記の 9 方向）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum InputDir {
    Neutral,
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
    /// グリフが判別できない（遮蔽・フラッシュ）
    Unknown,
}

impl InputDir {
    pub fn as_str(self) -> &'static str {
        match self {
            InputDir::Neutral => "N",
            InputDir::Up => "U",
            InputDir::UpRight => "UR",
            InputDir::Right => "R",
            InputDir::DownRight => "DR",
            InputDir::Down => "D",
            InputDir::DownLeft => "DL",
            InputDir::Left => "L",
            InputDir::UpLeft => "UL",
            InputDir::Unknown => "?",
        }
    }
}

/// DIR_TEMPLATES の並びに対応する方向
pub(super) const DIR_ORDER: [InputDir; 9] = [
    InputDir::Neutral,
    InputDir::Up,
    InputDir::UpRight,
    InputDir::Right,
    InputDir::DownRight,
    InputDir::Down,
    InputDir::DownLeft,
    InputDir::Left,
    InputDir::UpLeft,
];

/// ボタンバッジの色クラス（意味づけはせず色相で分類）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BadgeColor {
    Yellow, // H≈24-35（円）
    Orange, // H≈6-17（SP 箱）
    Green,  // H≈92-99（teal 円）
    Blue,   // H≈102-110（DP 箱）
    Red,    // H≈173-178（円）
}

impl BadgeColor {
    pub fn as_str(self) -> &'static str {
        match self {
            BadgeColor::Yellow => "Y",
            BadgeColor::Orange => "O",
            BadgeColor::Green => "G",
            BadgeColor::Blue => "B",
            BadgeColor::Red => "R",
        }
    }
}

/// クラシック操作の円内グリフ（拳 = パンチ / 足 = キック）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BtnGlyph {
    Punch,
    Kick,
}

/// バッジ 1 個（色 + 形状 + グリフ）。
///
/// - 円（boxed=false, glyph=None）: Modern の攻撃ボタン。
///   信号機カラーで 青緑=弱 / 黄=中 / 赤=強
/// - 円（boxed=false, glyph=Some）: クラシックの攻撃ボタン。
///   色 = 強度、グリフ = P/K で 6 ボタン（弱P/中P/強P/弱K/中K/強K）
/// - 文字付きボックス（boxed=true）: 特殊操作。橙=SP / 青=DP / teal=DI
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BadgeMark {
    pub color: BadgeColor,
    pub boxed: bool,
    pub glyph: Option<BtnGlyph>,
}

impl BadgeMark {
    /// 表示用ラベル。
    pub fn label(self) -> &'static str {
        use BtnGlyph::*;
        if let (false, Some(g)) = (self.boxed, self.glyph) {
            return match (self.color, g) {
                (BadgeColor::Green, Punch) => "弱P",
                (BadgeColor::Yellow, Punch) => "中P",
                (BadgeColor::Red, Punch) => "強P",
                (BadgeColor::Green, Kick) => "弱K",
                (BadgeColor::Yellow, Kick) => "中K",
                (BadgeColor::Red, Kick) => "強K",
                // 円は teal/黄/赤のみ（分類学的拒否で保証）
                _ => "?",
            };
        }
        match (self.boxed, self.color) {
            (false, BadgeColor::Green) => "弱",
            (false, BadgeColor::Yellow) => "中",
            (false, BadgeColor::Red) => "強",
            (true, BadgeColor::Orange) => "SP",
            (true, BadgeColor::Blue) => "DP",
            (true, BadgeColor::Green) => "DI",
            // 未観測の組み合わせ（色 + 形状をそのまま示す）
            (false, BadgeColor::Blue) => "円B",
            (false, BadgeColor::Orange) => "円O",
            (true, BadgeColor::Yellow) => "箱Y",
            (true, BadgeColor::Red) => "箱R",
        }
    }
}

/// 入力履歴 1 行の読み取り結果。
#[derive(Debug, Clone)]
pub struct InputRow {
    /// 継続フレーム数。読めない場合 None
    pub count: Option<u32>,
    /// 方向入力
    pub dir: InputDir,
    /// ボタンバッジ（画面 x 昇順）
    pub badges: Vec<BadgeMark>,
    /// AUTO バッジ（Modern のオートコンボ）
    pub auto: bool,
    /// 投げボタン（暗い円 + 白い手アイコン）
    pub throw: bool,
    /// 行が空（履歴がまだ無い / パネル外）
    pub empty: bool,
    /// 読み取り品質の悪化フラグ（遮蔽・フラッシュの疑い）
    pub uncertain: bool,
}
