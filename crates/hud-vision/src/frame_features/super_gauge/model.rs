/// SA ゲージの単フレーム読み取り結果。
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct SuperGaugeRead {
    /// 0.0〜3.0。整数部は表示ラベル、少数部は次ストックまでのバー。
    pub value: f32,
    /// 画面に表示された整数ラベル。CA 表示は Some(3)。
    pub displayed_level: Option<u8>,
    /// 画面に CA ラベルが表示されている。
    pub critical_art: bool,
    /// HUD 消失・遮蔽等で整数ラベルを確定できない。
    pub uncertain: bool,
}

impl Default for SuperGaugeRead {
    fn default() -> Self {
        Self {
            value: 0.0,
            displayed_level: None,
            critical_art: false,
            uncertain: true,
        }
    }
}
