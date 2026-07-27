// ── マスク照合（膨張許容距離） ───────────────────────────────────────────────
//
// グリフの線の太さは背景輝度と AA で ±1px 変動する（P1/P2 間でも差がある）ため、
// 単純 XOR ではなく「相手の 1px 膨張に含まれないピクセル数」で照合する。
// 太さ 1px の差は消え、形状の違いだけが距離に残る。

/// 行 bitmask 配列の 3x3 膨張
pub(super) fn dilate_rows<const N: usize>(m: &[u64; N], width: u32) -> [u64; N] {
    let col_mask: u64 = if width >= 64 {
        u64::MAX
    } else {
        (1u64 << width) - 1
    };
    let mut h = [0u64; N];
    for (i, &row) in m.iter().enumerate() {
        h[i] = (row | (row << 1) | (row >> 1)) & col_mask;
    }
    let mut out = [0u64; N];
    for i in 0..N {
        let up = if i > 0 { h[i - 1] } else { 0 };
        let dn = if i + 1 < N { h[i + 1] } else { 0 };
        out[i] = h[i] | up | dn;
    }
    out
}

/// 背景不変のグリフ照合距離。
///
/// グリフは不透明（白グリフ + 黒縁取り）で、テンプレート外は透明部 =
/// 背景が透ける。したがって評価はテンプレートの不透明領域内に限定する:
///   - 欠け: テンプレートグリフのうちサンプル（1px 膨張）に無い画素
///   - 侵食: テンプレートの縁取り帯（dilate3\dilate1 = 不透明の黒縁）に
///     サンプル白がある画素（別グリフのストロークはここに落ちる）
///
/// テンプレート領域外のサンプル白（= 背景の明部）は一切数えない
pub(super) fn glyph_distance<const N: usize>(sample: &[u64; N], t: &[u64; N], width: u32) -> u32 {
    let ds = dilate_rows(sample, width);
    let d1 = dilate_rows(t, width);
    let d2 = dilate_rows(&d1, width);
    let mut score = 0u32;
    for i in 0..N {
        score += (t[i] & !ds[i]).count_ones(); // 欠け
                                               // 縁取り帯 = グリフの 1px 外側リング（確実に不透明な黒縁の内側）。
                                               // 2px 以遠は透明部に踏み出し背景を拾うため使わない
        score += (sample[i] & d2[i] & !d1[i]).count_ones();
    }
    score
}
