use super::super::{HpColColor, HpZone};

/// アンカー正規化済みゾーン列のデコード結果。index はすべてアンカー相対
/// （0 = cap 端、大きいほどバーの空き側）。
pub(crate) struct HpZonesDecode {
    pub(crate) fill_ratio: f32,
    pub(crate) orange_fill: f32,
    pub(crate) uncertain: bool,
    pub(crate) fill_edge_a: Option<usize>, // 充填端（fill_edge White のアンカーから遠い側の端）
    pub(crate) damage_left_a: Option<usize>, // ダメージゾーン境界（同上）
}

/// アンカー正規化済みゾーン列をステートマシンで前方スキャンし、
/// 充填端・ダメージ端を検出する。サイド非依存。
///
/// 事前バリデーションは行わず、以下の知識をすべてスキャン中に判定する:
///   - 純白ゾーン幅 > 3 → 遮蔽（エンコードにじみは最大 3px）
///   - 純白ゾーン数の上限 3（cap / fill_edge / 遠端）はステート遷移で自然に保証
///   - Fill が cap より先に出現 → cap が塞がれている → uncertain
///
/// SeekCap      : 外側フレームをスキップして最初の White cap を探す。
///                Fill/Ghost に出会ったら cap 消失確定 → uncertain。
/// FillScan     : HP 充填色ゾーンをスキャン。
///                White(幅≤3) → fill_edge 確定 → AfterFillEdge。
///                幅 > MAX_DARK_IN_FILL の Dark → fill 充填端は last_fill_zone で
///                  確定するが、安定HP の空き端とスプライト遮蔽が zone 構造上
///                  区別できないため uncertain として終了。
///                  （fill を一度も見ていなければ HP≈0% → 確定として終了）
///                Ghost（fill 未検出）→ HP=0 でダメージ残像のみ点灯（KO 直後）
///                  → HP≈0% 確定として InDamage へ。
/// AfterFillEdge: ダメージゾーンか遠端 cap を探す。
/// InDamage     : 受けダメージの橙色帯をスキャン。Dark → 橙色帯終端 → 終了。
pub(crate) fn decode_hp_zones(zones: &[HpZone], roi_w: usize) -> HpZonesDecode {
    const MAX_WHITE_WIDTH: usize = 3; // これを超える純白ゾーン幅 → 遮蔽
                                      // アンカー cap のみ 6px まで許容: ヒット時のバー白フラッシュ／ピンチ点滅で
                                      // cap 隣接の淡色グラデーション列 1〜3 本が White 判定に繰り上がり、
                                      // cap 幅が 3→4〜6px に膨らむため（ピンチ P2 実測で最大 6px）。
    const MAX_CAP_WHITE_WIDTH: usize = 6;

    #[derive(Clone, Copy)]
    enum Sm {
        SeekCap,
        FillScan,
        AfterFillEdge,
        InDamage,
    }

    let mut sm = Sm::SeekCap;
    let mut fill_edge_zone: Option<HpZone> = None;
    let mut damage_left_zone: Option<HpZone> = None;
    // FillScan 中に見た最後の Fill ゾーン。White fill_edge がない場合の fill 端 代替。
    let mut last_fill_zone: Option<HpZone> = None;
    let mut uncertain = false;

    'scan: for (zone_index, zone) in zones.iter().enumerate() {
        sm = match sm {
            // HP バー端 cap（白枠）を探す。ROI は平行四辺形スキャンで HP ゲージ白枠に
            // 合わせてあるため外縁スキップ不要。Fill が先に出たら cap が塞がれている。
            Sm::SeekCap => match zone.color {
                HpColColor::White => {
                    if zone.width() > MAX_CAP_WHITE_WIDTH {
                        // 太い白は cap ではなく遮蔽。cap を見つけないまま
                        // 終わるので、走査後に uncertain が立つ。
                        break 'scan;
                    }
                    Sm::FillScan
                }
                // 満タン付近では、動画圧縮と平行四辺形の斜めスキャンにより
                // cap の外側 1 列だけが Fill に分類されることがある。
                // 実測: P2 の `Fill(1) -> White(2) -> Fill(...) -> White(2)`。
                // 直後に妥当な白 cap がある 1px Fill に限って AA ノイズとして
                // 読み飛ばす。2px 以上、Ghost、白 cap が続かない Fill は従来通り
                // 遮蔽として扱い、スプライトを HP と誤認しない。
                HpColColor::Fill
                    if zone.width() == 1
                        && zones.get(zone_index + 1).is_some_and(|next| {
                            next.color == HpColColor::White && next.width() <= MAX_CAP_WHITE_WIDTH
                        }) =>
                {
                    Sm::SeekCap
                }
                // cap より先に fill や残像が出るのは、cap が塞がれている。
                // これも cap を見ないまま終わる。
                HpColColor::Fill | HpColColor::Ghost => break 'scan,
                HpColColor::Dark | HpColColor::YellowWhite | HpColColor::Orange => Sm::SeekCap,
            },

            // HP 充填域: Fill と YW（fill→cap 境界のにじみ）を許容。
            // HP メーターにギャップはないが、フレームメーター描画により 2〜12px 程度の
            // 細い Dark ゾーンが fill 内に出現する（正常描画アーティファクト）。
            Sm::FillScan => match zone.color {
                HpColColor::Fill => {
                    last_fill_zone = Some(*zone);
                    Sm::FillScan
                }
                // YW は境界ブレンド列。幅 1 の YW だけで「fill を見た」と誤認しないよう
                // last_fill_zone にはセットせず継続のみ。
                HpColColor::YellowWhite => Sm::FillScan,
                // Ghost = 暗い黄橙の残像。fill の検出前後で意味が変わる:
                // fill 検出後 → fill 内側の一時減光ゾーン（コンボ中、White edge が
                //   さらに先にある。frame 4010-4013 実測）→ fill 同様に継続。
                // fill 未検出 → 残量ゼロでダメージ残像のみ点灯（KO 直後、frame
                //   4063-4078 実測）→ HP≈0% 確定（uncertain=false・fill_edge なし）
                //   として InDamage へ。
                HpColColor::Ghost => {
                    if last_fill_zone.is_some() {
                        last_fill_zone = Some(*zone);
                        Sm::FillScan
                    } else {
                        Sm::InDamage
                    }
                }
                HpColColor::Dark => {
                    const MAX_DARK_IN_FILL: usize = 15;
                    if zone.width() > MAX_DARK_IN_FILL {
                        // 大きな Dark = fill→空き端（安定HP）or スプライト遮蔽。
                        // fill を一度でも見ていた場合: 安定HP とスプライト遮蔽は zone 構造上
                        // 区別不能なため uncertain=true。fill_edge_zone は last_fill_zone で確定。
                        // fill なし → HP≈0%（cap 直後が完全に空き）→ uncertain=false のまま。
                        if last_fill_zone.is_some() {
                            fill_edge_zone = last_fill_zone;
                            uncertain = true;
                            break 'scan;
                        } else {
                            break 'scan; // HP≈0%: fill_edge_zone は None のまま
                        }
                    } else {
                        Sm::FillScan
                    }
                }
                HpColColor::White => {
                    if zone.width() > MAX_WHITE_WIDTH {
                        uncertain = true;
                        break 'scan;
                    }
                    fill_edge_zone = Some(*zone);
                    Sm::AfterFillEdge
                }
                HpColColor::Orange => {
                    uncertain = true;
                    break 'scan;
                }
            },

            // fill_edge 以降: ダメージゾーンか左端 cap を探す。
            // Dark は HP バーの空き端（正常終端）または YW 境界の間にある遷移列。
            // YW は fill_edge と Orange の境界ブレンドピクセル → Orange と同様に InDamage へ。
            Sm::AfterFillEdge => match zone.color {
                HpColColor::Orange | HpColColor::YellowWhite | HpColColor::Ghost => Sm::InDamage,
                HpColColor::Fill | HpColColor::Dark => Sm::AfterFillEdge,
                HpColColor::White => {
                    if zone.width() > MAX_WHITE_WIDTH {
                        uncertain = true;
                        break 'scan;
                    }
                    break 'scan; // 左端 cap（正常な 3 番目の White）→ 終了
                }
            },

            // ダメージゾーン（受けダメージの橙色帯 / 暗いゴースト残像）
            // ゾーン境界付近の半透明ピクセルが Fill に落ちる場合もあるため Orange と同等に継続する。
            Sm::InDamage => match zone.color {
                HpColColor::Orange | HpColColor::Fill | HpColColor::Ghost => Sm::InDamage,
                HpColColor::Dark => break 'scan,
                HpColColor::White => {
                    if zone.width() > MAX_WHITE_WIDTH {
                        uncertain = true;
                        break 'scan;
                    }
                    damage_left_zone = Some(*zone);
                    break 'scan;
                }
                HpColColor::YellowWhite => {
                    damage_left_zone = Some(*zone);
                    break 'scan;
                }
            },
        };
    }

    // SeekCap のまま終了 → White cap が一度も見つからなかった → uncertain
    if matches!(sm, Sm::SeekCap) {
        uncertain = true;
    }

    // fill_edge_a / damage_left_a: ゾーンのアンカーから遠い側の端 = z.end
    // （fill は [0..=fill_edge_a] を占め、damage はその先 [fill_edge_a..=damage_left_a]）
    let fill_edge_a = fill_edge_zone.map(|z| z.end);
    let damage_left_a = damage_left_zone.map(|z| z.end);

    // orange_fill: fill_edge と damage 境界の間の幅を ROI 幅で正規化。
    let orange_fill = match (fill_edge_a, damage_left_a) {
        (Some(fe), Some(dl)) => dl.saturating_sub(fe) as f32 / roi_w as f32,
        _ => 0.0,
    };

    // fill_ratio 算出。fill_edge_a が得られた場合は境界ベース:
    // fill はアンカー側 [0..=fe] を占める → (fe + 1) / roi_w。
    //
    // fill_edge が見つからない場合:
    //   last_fill_zone.is_some() → fill が端まで達している = HP≈100%
    //   last_fill_zone.is_none() → fill なし = HP≈0%
    let fill_ratio = match fill_edge_a {
        Some(fe) => (fe + 1).min(roi_w) as f32 / roi_w as f32,
        None if uncertain => 0.0,
        None if last_fill_zone.is_some() => 1.0, // HP≈100%
        None => 0.0,                             // HP≈0%
    };

    HpZonesDecode {
        fill_ratio,
        orange_fill,
        uncertain,
        fill_edge_a,
        damage_left_a,
    }
}
