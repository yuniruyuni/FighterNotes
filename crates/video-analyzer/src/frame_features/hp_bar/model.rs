/// 斜め列 1 本の色カテゴリ。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HpColColor {
    White,       // 純白ふちどり: R>180, G>180, B>180 (右端 cap, fill edge)
    Fill,        // HP 充填色（赤 P1 / 青 P2 / 低 HP 黄、高輝度 V>200）
    Ghost,       // ダメージゴースト: コンボで失った HP の暗い残像（H=20–30, S>150, 100≤V<200）
    YellowWhite, // 黄白: damage zone 左端境界（R>165, G>150, B>100, 非 White）
    Orange,      // 受けダメージゾーン（H=10–27, S>60, 80<V<200）
    Dark,        // 空き・背景・その他
}

/// 同色の連続列区間。
#[derive(Clone, Copy)]
pub(crate) struct HpZone {
    pub(crate) color: HpColColor,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl HpZone {
    pub(crate) fn width(self) -> usize {
        self.end - self.start + 1
    }
}

/// HP バーデコード結果（ステートマシン出力）。
pub(crate) struct HpBarDecode {
    pub(crate) fill_ratio: f32,  // 充填率 0.0–1.0（fill_edge_cy から算出）
    pub(crate) orange_fill: f32, // damage zone の幅 / roi_w（境界ベース）
    pub(crate) uncertain: bool,
    pub(crate) fill_edge_cy: Option<usize>, // 充填端列インデックス（fill zone 境界, P1=左端, P2=右端）
    pub(crate) damage_left_cy: Option<usize>, // ダメージゾーン境界列（存在する場合のみ）
}

/// 斜め列 1 本を色カテゴリに分類する。
///
/// サイド固有の HP 一次充填色。
/// 黄色ピンチ fill・ゴースト・ダメージ橙は両サイド共通のため、
/// サイドごとに異なるのは通常時の充填色（P1 赤 / P2 青）のみ。
#[derive(Debug, Clone, Copy)]
pub(crate) enum HpFillHue {
    Red,  // P1
    Blue, // P2
}
