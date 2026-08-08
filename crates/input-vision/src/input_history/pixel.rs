pub struct Frame<'a> {
    pub(crate) rgba: &'a [u8],
    pub(crate) w: usize,
    pub(crate) y_off: usize, // ストリップ運用時の先頭行 y（現状 0 = フルフレーム）
    /// 白判定閾値。通常 210（背景透け除去）。画面暗転時は文字自体が
    /// 暗くなるため 180 でリトライする（暗転時は背景も暗く bleed しない）
    pub(crate) white_th: u8,
}

impl<'a> Frame<'a> {
    /// 走査対象のストリップを包む。
    ///
    /// `y_off` はストリップの先頭行がフレーム全体の何行目かを表す。
    /// `white_th` は白判定の閾値で、通常は 210、画面暗転時は 180 を使う。
    pub fn new(rgba: &'a [u8], w: usize, y_off: usize, white_th: u8) -> Self {
        Self {
            rgba,
            w,
            y_off,
            white_th,
        }
    }

    #[inline]
    pub(crate) fn px(&self, x: usize, y: usize) -> Option<(u8, u8, u8)> {
        let yy = y.checked_sub(self.y_off)?;
        let idx = (yy * self.w + x) * 4;
        if idx + 2 >= self.rgba.len() {
            return None;
        }
        Some((self.rgba[idx], self.rgba[idx + 1], self.rgba[idx + 2]))
    }

    /// 白（文字・グリフ）判定。グリフの純白は 240+、半透明パネル越しの
    /// 明るい背景は ≤210 のため通常閾値は 210
    #[inline]
    pub(crate) fn is_white(&self, x: usize, y: usize) -> bool {
        let th = self.white_th;
        matches!(self.px(x, y), Some((r, g, b)) if r > th && g > th && b > th)
    }

    /// グレースケール値（min チャンネル）。範囲外は 0
    #[inline]
    pub(crate) fn gray(&self, x: usize, y: usize) -> u8 {
        self.px(x, y).map_or(0, |(r, g, b)| r.min(g).min(b))
    }
}
