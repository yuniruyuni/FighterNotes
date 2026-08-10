//! アンカー正規化済みのラン列からドライブゲージを読む。左右非依存。
//!
//! 空き領域は半透明でステージ背景が透けるため、「空き = 暗い」が成り立たない。
//! 信頼できるのは点灯セルの高い彩度と、不透明な回復バーの灰色だけ。そのため
//! 厳密なゾーン文法ではなく、アンカー側から点灯を繋いだ遠端で読む。

use super::model::{DriveColClass, DriveGaugeRead};

/// アンカー縁で点灯していなくても許す幅。枠線の描画で数列暗くなる。
const MAX_EDGE_SKIP: usize = 8;
/// 排出中の部分セルは、本体から 1 セルピッチ（54px）ほど離れた小島として
/// 描かれる。構造上そのセルより先に点灯は無いので、ここまでは繋いでよい。
const MAX_CELL_GAP: usize = 56;
/// これを超える異物はスプライトの遮蔽。細いものは背景の透け。
const MAX_FOREIGN: usize = 8;
/// 回復バーの中に出る描画ノイズを跨ぐ幅。
const MAX_GRAY_GAP: usize = 6;
/// 回復バーとして認める最小の幅。
const MIN_GRAY_SLAB: usize = 10;

impl DriveGaugeRead {
    /// 読めなかった。HUD の消失、全画面フラッシュ、遮蔽がこれに当たる。
    fn unreadable() -> Self {
        Self {
            value: 0.0,
            burnout: false,
            recovery: 0.0,
            uncertain: true,
        }
    }

    /// 点灯の遠端から出した残量。値は本数で、満タンが 6 本。
    fn stocks(far: usize, roi_w: usize) -> Self {
        Self {
            value: ((far + 1) as f32 / roi_w as f32 * 6.0).min(6.0),
            burnout: false,
            recovery: 0.0,
            uncertain: false,
        }
    }

    /// バーンアウトに入った瞬間。バーは空で、回復もまだ始まっていない。
    fn burnout_entry() -> Self {
        Self {
            value: 0.0,
            burnout: true,
            recovery: 0.0,
            uncertain: false,
        }
    }

    /// バーンアウトからの回復中。灰色の帯の遠端が進み具合になる。
    fn recovering(far: usize, roi_w: usize) -> Self {
        Self {
            value: 0.0,
            burnout: true,
            recovery: ((far + 1) as f32 / roi_w as f32).min(1.0),
            uncertain: false,
        }
    }
}

/// アンカー側から点灯のランを繋いだ結果。
enum LitChain {
    /// 繋がった遠端。
    Reaches(usize),
    /// 点灯はあるが、ゲージとしてあり得ない並び方をしている。
    Occluded,
    /// 点灯が無い。
    Missing,
}

/// アンカー側から点灯のランを繋ぐ。
///
/// ゲージはアンカー側のセルが最後に減るので、繋がった範囲より先に本物の
/// 点灯は無い。あるとすれば遮蔽で連鎖が分断されたか、ゲージ以外の何かが
/// 光っている。
fn chain_lit(runs: &[(DriveColClass, usize, usize)]) -> LitChain {
    /// 通常のセル間の隙間の上限。実測 2〜4px。
    const MAX_SEAM_GAP: usize = 8;
    /// 大きな隙間の先で許す小島や文字ストロークの幅。
    const SLIVER_MAX_W: usize = 24;
    /// 繋がる範囲の外にあっても無視してよい点灯の幅。
    const NEGLIGIBLE_W: usize = 3;

    let mut far: Option<usize> = None;
    for &(class, start, end) in runs {
        if class != DriveColClass::Lit {
            continue;
        }
        let width = end - start + 1;
        let Some(reached) = far else {
            // 先頭の点灯がアンカーから離れていれば、アンカー側が隠れている。
            if start > MAX_EDGE_SKIP {
                return LitChain::Occluded;
            }
            far = Some(end);
            continue;
        };
        if start > reached + MAX_CELL_GAP {
            // 繋がる範囲の外にある実体のある点灯は、ゲージではない何か。
            return if width >= NEGLIGIBLE_W {
                LitChain::Occluded
            } else {
                LitChain::Reaches(reached)
            };
        }
        if start > reached + 1 + MAX_SEAM_GAP && width > SLIVER_MAX_W {
            // 大きな隙間の先の幅広いランは、小島ではなく遮蔽体。
            return LitChain::Occluded;
        }
        far = Some(end);
    }
    far.map_or(LitChain::Missing, LitChain::Reaches)
}

/// 幅のある異物がゲージのどこかに重なっているか。
///
/// 遮蔽体はゲージ本体を暗転させて連鎖を短く切ることがあるので、繋がった
/// 範囲に限らず全域で見る。
fn has_wide_foreign(runs: &[(DriveColClass, usize, usize)]) -> bool {
    runs.iter().any(|&(class, start, end)| {
        class == DriveColClass::Foreign && end - start + 1 > MAX_FOREIGN
    })
}

/// 繋がった点灯から残量を読む。
///
/// ゲージはアンカー側のセルが最後に減るので、本物なら実セル幅のランを
/// 必ず一本は含む。細切れの点灯しかないのは、バーンアウト突入演出の
/// 「EMPTY」の文字か、キャラクターの遮蔽のどちらか。
fn read_lit_chain(
    runs: &[(DriveColClass, usize, usize)],
    far: usize,
    roi_w: usize,
) -> DriveGaugeRead {
    /// 本物のゲージが埋めているはずの割合。
    const MIN_LIT_COVERAGE: f32 = 0.70;
    /// 実セル（54px）由来と見なす最小のラン幅。
    const CELL_RUN_MIN: usize = 35;
    /// EMPTY の文字列が占める幅。実測 109〜133px。残った部分セルの
    /// 断片はこれより狭い（27px 以下）。
    const EMPTY_MIN_EXTENT: usize = 80;
    const EMPTY_MAX_EXTENT: usize = 150;
    /// 文字のストロークの幅。実測 22px 以下。
    const EMPTY_MAX_STROKE: usize = 24;
    /// EMPTY と認めるストロークの本数。
    const EMPTY_MIN_STROKES: usize = 3;

    let widths: Vec<usize> = runs
        .iter()
        .filter(|&&(class, start, _)| class == DriveColClass::Lit && start <= far)
        .map(|&(_, start, end)| end - start + 1)
        .collect();
    let covered: usize = widths.iter().sum();
    let coverage = covered as f32 / (far + 1) as f32;
    let widest = widths.iter().copied().max().unwrap_or(0);

    // 十分埋まっているか、実セル幅のランが一本でもあれば残量として通す。
    // 排出中の分離小島は本体セルを含むのでここを通る。
    if coverage >= MIN_LIT_COVERAGE || widest >= CELL_RUN_MIN {
        return DriveGaugeRead::stocks(far, roi_w);
    }

    let looks_like_empty_text = (EMPTY_MIN_EXTENT..=EMPTY_MAX_EXTENT).contains(&far)
        && widest <= EMPTY_MAX_STROKE
        && widths.len() >= EMPTY_MIN_STROKES;
    if looks_like_empty_text {
        return DriveGaugeRead::burnout_entry();
    }
    DriveGaugeRead::unreadable()
}

/// アンカー側から伸びる灰色の帯を繋ぐ。バーンアウト中の回復バー。
/// 返すのは帯の始まりと遠端。
fn chain_gray(runs: &[(DriveColClass, usize, usize)]) -> Option<(usize, usize)> {
    let mut slab_start: Option<usize> = None;
    let mut far: Option<usize> = None;
    for &(class, start, end) in runs {
        if class != DriveColClass::Gray {
            continue;
        }
        let Some(reached) = far else {
            // アンカーから離れた灰色は背景の透け。回復バーではない。
            if start <= MAX_EDGE_SKIP {
                slab_start = Some(start);
                far = Some(end);
            }
            continue;
        };
        if start > reached + MAX_GRAY_GAP {
            break;
        }
        far = Some(end);
    }
    slab_start.zip(far)
}

/// ラン列からドライブゲージを読む。
pub(crate) fn decode_drive_runs(
    runs: &[(DriveColClass, usize, usize)],
    roi_w: usize,
) -> DriveGaugeRead {
    match chain_lit(runs) {
        LitChain::Occluded => return DriveGaugeRead::unreadable(),
        LitChain::Reaches(far) => {
            if has_wide_foreign(runs) {
                return DriveGaugeRead::unreadable();
            }
            return read_lit_chain(runs, far, roi_w);
        }
        LitChain::Missing => {}
    }

    // 点灯が皆無 → バーンアウト中の回復バーを探す。
    if has_wide_foreign(runs) {
        return DriveGaugeRead::unreadable();
    }
    let Some((slab_start, slab_far)) = chain_gray(runs) else {
        // 点灯も回復バーも無い。HUD の消失、全画面フラッシュ、バーンアウト
        // 突入直後（バー幅ゼロ）は互いに区別できない。
        return DriveGaugeRead::unreadable();
    };
    if slab_far - slab_start + 1 < MIN_GRAY_SLAB {
        return DriveGaugeRead::unreadable();
    }

    // 繋がる範囲の先に幅のある灰色があるのは、遮蔽体がバーを暗転させて
    // 分断したということ。背景の透け（13px 以下）とは幅で区別する。
    const GRAY_BEYOND_MIN: usize = 20;
    let split_by_occlusion = runs.iter().any(|&(class, start, end)| {
        class == DriveColClass::Gray
            && start > slab_far + MAX_GRAY_GAP
            && end - start + 1 >= GRAY_BEYOND_MIN
    });
    if split_by_occlusion {
        return DriveGaugeRead::unreadable();
    }

    DriveGaugeRead::recovering(slab_far, roi_w)
}
