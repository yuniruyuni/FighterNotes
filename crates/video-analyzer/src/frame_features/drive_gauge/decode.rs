use super::model::{DriveColClass, DriveGaugeRead};

/// アンカー正規化済みラン列（index 0 = 中央端）をデコードする。サイド非依存。
///
/// 空き領域の見た目は不定（半透明でステージ背景が透ける）ため、
/// 厳格なゾーン文法ではなく「Lit 連鎖」で読む:
///
///   値モード      : アンカー近傍から始まる Lit ランを、ギャップ ≤1 セルピッチで
///                   連鎖した遠端が現在値。ギャップの色クラスは問わない
///                   （セル間ギャップ・排出中セルの分離小島・シャイン演出・
///                   背景透けをすべて自然に許容する）。
///   バーンアウト  : Lit が皆無で、アンカー近傍から Gray スラブ（回復バー）が
///                   始まる場合。回復進捗 = Gray 連鎖の遠端。
///   uncertain     : Lit も Gray スラブも無い（HUD 消失・全画面フラッシュ）、
///                   または幅のある Foreign（スプライト遮蔽）が計測範囲に重なる。
pub(crate) fn decode_drive_runs(
    runs: &[(DriveColClass, usize, usize)],
    roi_w: usize,
) -> DriveGaugeRead {
    const MAX_EDGE_SKIP: usize = 8; // アンカー縁の非 Lit 許容幅
                                    // 排出中の部分セルは本体ブロックから最大 ≈1 セルピッチ離れた小島として
                                    // 描画される（frame 3400 実測: 15px ギャップ + 5px 島）。構造上 lit は
                                    // 排出セルより先に存在しないため、1 ピッチ以内のギャップ許容は安全。
    const MAX_CELL_GAP: usize = 56; // ≈1 セルピッチ（54px）+ マージン
    const MAX_FOREIGN: usize = 8; // これを超える Foreign = スプライト遮蔽
    const MAX_GRAY_GAP: usize = 6; // 回復バー内の描画ノイズ許容幅
    const MIN_GRAY_SLAB: usize = 10; // 回復バーとして認める最小幅

    // ── Lit 連鎖 ─────────────────────────────────────────────────────────
    // 実ゲージの構造制約（アンカー側セルが最後に減る）から:
    //   - 連鎖範囲より先に実体 Lit は存在しない（あれば遮蔽で連鎖が分断された）
    //   - 大ギャップ（セル間ギャップ >8px）の先に来る正当な Lit は
    //     排出中セルの分離小島（幅 ≤24px）のみ。幅広ランは遮蔽体
    const MAX_SEAM_GAP: usize = 8; // 通常のセル間ギャップ上限（実測 2-4px）
    const SLIVER_MAX_W: usize = 24; // 大ギャップ先で許容する小島/文字ストローク幅
    let mut lit_far: Option<usize> = None;
    let mut lit_blocked = false; // 先頭 Lit がアンカーから離れすぎ（アンカー側遮蔽）
    let mut lit_occluded = false; // 構造上あり得ない Lit 配置（遮蔽体）
    for &(class, start, end) in runs {
        if class != DriveColClass::Lit {
            continue;
        }
        let w = end - start + 1;
        match lit_far {
            None => {
                if start > MAX_EDGE_SKIP {
                    lit_blocked = true;
                }
                lit_far = Some(end);
            }
            Some(far) => {
                if start <= far + MAX_CELL_GAP {
                    if start > far + 1 + MAX_SEAM_GAP && w > SLIVER_MAX_W {
                        lit_occluded = true; // 大ギャップ先の幅広ラン = 遮蔽体
                        break;
                    }
                    lit_far = Some(end);
                } else {
                    if w >= 3 {
                        lit_occluded = true;
                    } // 連鎖範囲外の実体 Lit
                    break;
                }
            }
        }
    }

    if let Some(far) = lit_far {
        // 幅のある Foreign がゲージ内のどこかにある → スプライト遮蔽の証拠。
        // 遮蔽体はゲージ本体を暗転させて連鎖を短く切ることがある
        // （frame 2221 実測: Foreign 24px が連鎖の先に出現、値は偽の低値）
        // ため、連鎖範囲に限らず全域でチェックする。
        let occluded = runs
            .iter()
            .any(|&(c, s, e)| c == DriveColClass::Foreign && e - s + 1 > MAX_FOREIGN);
        if lit_blocked || occluded || lit_occluded {
            return DriveGaugeRead {
                value: 0.0,
                burnout: false,
                recovery: 0.0,
                uncertain: true,
            };
        }

        // ── Lit 被覆率チェック ────────────────────────────────────────────
        // 実ゲージはアンカー側セルが最後に減るため、実セル幅（≥35px）の
        // ランを 1 本も含まない細切れ Lit 連鎖は本物ではない:
        //   - バーンアウト突入演出の「EMPTY」文字（ストローク ≤22px の群、
        //     被覆率 ≈0.5、アンカー側 ≤150px。frame 2095-2115 / 4000 実測）
        //     → バーンアウト突入の瞬間として確定
        //   - スプライト遮蔽（ジャンプでキャラがゲージ手前に来る等、
        //     幅広ストロークや広範囲の断片）→ uncertain
        // 排出中の分離小島（本体セル幅ラン + 細い小島、被覆率 0.5-0.8）は
        // 実セル幅ランを含むため値として通す。
        const MIN_LIT_COVERAGE: f32 = 0.70;
        const CELL_RUN_MIN: usize = 35; // 実セル（54px）由来と見なす最小ラン幅
        const EMPTY_MIN_EXTENT: usize = 80; // EMPTY 文字列の最小幅（実測 109-133px。
                                            // 残存部分セルの断片は ≤27px: frame 1452-1464 実測）
        const EMPTY_MAX_EXTENT: usize = 150; // EMPTY 文字列の最大幅（実測 ≈133px）
        const EMPTY_MAX_STROKE: usize = 24; // 文字ストロークの最大幅（実測 ≤22px）
        let lit_runs: Vec<usize> = runs
            .iter()
            .filter(|&&(c, s, _)| c == DriveColClass::Lit && s <= far)
            .map(|&(_, s, e)| e - s + 1)
            .collect();
        let lit_cols: usize = lit_runs.iter().sum();
        let coverage = lit_cols as f32 / (far + 1) as f32;
        let max_stroke = lit_runs.iter().copied().max().unwrap_or(0);
        if coverage < MIN_LIT_COVERAGE && max_stroke < CELL_RUN_MIN {
            if (EMPTY_MIN_EXTENT..=EMPTY_MAX_EXTENT).contains(&far)
                && max_stroke <= EMPTY_MAX_STROKE
                && lit_runs.len() >= 3
            {
                // EMPTY 文字シグネチャ → バーンアウト突入の瞬間
                return DriveGaugeRead {
                    value: 0.0,
                    burnout: true,
                    recovery: 0.0,
                    uncertain: false,
                };
            }
            // それ以外の細切れ Lit = 遮蔽
            return DriveGaugeRead {
                value: 0.0,
                burnout: false,
                recovery: 0.0,
                uncertain: true,
            };
        }

        return DriveGaugeRead {
            value: ((far + 1) as f32 / roi_w as f32 * 6.0).min(6.0),
            burnout: false,
            recovery: 0.0,
            uncertain: false,
        };
    }

    // ── Lit なし → バーンアウト回復バーを探す ────────────────────────────
    let mut gray_far: Option<usize> = None;
    let mut gray_start: Option<usize> = None;
    for &(class, start, end) in runs {
        match class {
            DriveColClass::Gray => match gray_far {
                None => {
                    if start <= MAX_EDGE_SKIP {
                        gray_start = Some(start);
                        gray_far = Some(end);
                    }
                    // アンカーから離れた Gray は背景透け → 無視
                }
                Some(far) => {
                    if start <= far + MAX_GRAY_GAP {
                        gray_far = Some(end);
                    } else {
                        break;
                    }
                }
            },
            DriveColClass::Foreign => {
                let w = end - start + 1;
                if w > MAX_FOREIGN {
                    return DriveGaugeRead {
                        value: 0.0,
                        burnout: false,
                        recovery: 0.0,
                        uncertain: true,
                    };
                }
            }
            _ => {}
        }
    }

    match (gray_start, gray_far) {
        (Some(gs), Some(gf)) if gf - gs + 1 >= MIN_GRAY_SLAB => {
            // 回復バーの連鎖範囲より先に幅広 Gray がある = 遮蔽体がバーを
            // 暗転させて分断した（frame 2996 実測: recovery が偽の低値になる）。
            // 背景透けの細い Gray（≤13px 実測）とはバー本体の幅で区別する。
            let gray_beyond = runs.iter().any(|&(c, s, e)| {
                c == DriveColClass::Gray && s > gf + MAX_GRAY_GAP && e - s + 1 >= 20
            });
            if gray_beyond {
                return DriveGaugeRead {
                    value: 0.0,
                    burnout: false,
                    recovery: 0.0,
                    uncertain: true,
                };
            }
            DriveGaugeRead {
                value: 0.0,
                burnout: true,
                recovery: ((gf + 1) as f32 / roi_w as f32).min(1.0),
                uncertain: false,
            }
        }
        // Lit も回復バーも無い: HUD 消失・全画面フラッシュ・バーンアウト
        // 突入直後（バー幅ゼロ）は区別不能 → uncertain
        _ => DriveGaugeRead {
            value: 0.0,
            burnout: false,
            recovery: 0.0,
            uncertain: true,
        },
    }
}
