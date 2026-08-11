use super::*;

/// 2 フレーム間の動画時間から、両者同時停止していたフレームを差し引く。
/// SA の 3 秒演出を「3 秒間コンボが途切れた」と扱わないためのゲーム進行差。
fn effective_gameplay_gap(from: u32, to: u32, freeze_spans: &[(u32, u32)]) -> u32 {
    let raw = to.saturating_sub(from);
    let frozen: u32 = freeze_spans
        .iter()
        .map(|&(a, b)| {
            let start = a.max(from.saturating_add(1));
            let end = b.min(to);
            if start <= end {
                end - start + 1
            } else {
                0
            }
        })
        .sum();
    raw.saturating_sub(frozen)
}

/// HP の下降を、最後の下降から `DMG_GAP` 以内のまとまりへ変換する。
/// `freeze_spans` を渡すと、その停止時間を間隔から除外する。
/// `stun` が連続している間は、SA/CA 演出でフレームメーター自体が進んでも
/// 被弾側に行動可能な切れ目がないため同じコンボとして扱う。
pub(crate) fn extract_damage_sequences(
    features: &[FrameFeatures],
    hp: &[Vec<f32>; 2],
    rounds: &[RoundInfo],
    freeze_spans: &[(u32, u32)],
    stun: [&[bool]; 2],
) -> Vec<DamageEvent> {
    let n = features.len();
    let mut damage = Vec::new();
    if n == 0 {
        return damage;
    }
    for round in rounds {
        let (a, b) = (
            idx_of(features, round.start_frame),
            idx_of(features, round.end_frame),
        );
        for (side, values) in hp.iter().enumerate() {
            let mut consumed_through = a;
            for start in a..=b {
                if start <= consumed_through || values[start] >= values[start - 1] - DMG_EPS {
                    continue;
                }

                let mut last_drop = start;
                let mut searching = true;
                for j in start.saturating_add(1)..=b {
                    if searching {
                        let from = features[last_drop].frame_index;
                        let to = features[j].frame_index;
                        let gap = effective_gameplay_gap(from, to, freeze_spans);
                        let freeze_relevant =
                            freeze_spans.iter().any(|&(freeze_start, freeze_end)| {
                                (freeze_start <= to && freeze_end > from)
                                    || (freeze_start > to
                                        && freeze_start.saturating_sub(from)
                                            <= DMG_GAP_ACROSS_FREEZE)
                            });
                        let max_gap = if freeze_relevant {
                            DMG_GAP_ACROSS_FREEZE
                        } else {
                            DMG_GAP as u32
                        };
                        let continuous_stun = stun[side]
                            .get(last_drop..=j)
                            .is_some_and(|span| span.iter().all(|value| *value));
                        if gap > max_gap && !continuous_stun {
                            searching = false;
                        } else if values[j] < values[j - 1] - DMG_EPS {
                            last_drop = j;
                        }
                    }
                }
                let drop = values[start - 1] - values[last_drop];
                if drop.total_cmp(&DMG_MIN_DROP).is_ge() && values[start - 1] > DEAD_HP {
                    damage.push(DamageEvent {
                        victim: side as u8 + 1,
                        start_frame: features[start].frame_index,
                        pre_freeze_frame: features[start].frame_index,
                        end_frame: features[last_drop].frame_index,
                        hp_before: values[start - 1],
                        hp_after: values[last_drop],
                        drop,
                        round_no: round.round_no,
                    });
                }
                consumed_through = last_drop;
            }
        }
    }
    damage.sort_by_key(|event| event.start_frame);
    damage
}

/// HP の最後の安定読みによる終端が SA/KO 停止の途中にある場合、演出末尾まで
/// ラウンドを延長する。次ラウンド開始を越えず、HP 終値も延長先へ同期する。
pub(crate) fn extend_rounds_through_freezes(
    rounds: &mut [RoundInfo],
    features: &[FrameFeatures],
    hp: &[Vec<f32>; 2],
    freeze_spans: &[(u32, u32)],
) {
    for index in 0..rounds.len() {
        let limit = rounds.get(index + 1).map_or_else(
            || {
                features
                    .last()
                    .map_or(rounds[index].end_frame, |f| f.frame_index)
            },
            |next| next.start_frame.saturating_sub(1),
        );
        let mut end = rounds[index].end_frame;
        if let Some((_, freeze_end)) = freeze_spans
            .iter()
            .copied()
            .find(|&(freeze_start, freeze_end)| freeze_start <= end && end <= freeze_end)
        {
            let extended = freeze_end.min(limit);
            end = end.max(extended);
        }
        if end == rounds[index].end_frame {
            continue;
        }
        rounds[index].end_frame = end;
        let at = idx_of(features, end);
        if hp[0][at] >= 0.0 {
            rounds[index].p1_hp_end = hp[0][at];
        }
        if hp[1][at] >= 0.0 {
            rounds[index].p2_hp_end = hp[1][at];
        }
    }
}

#[cfg(test)]
mod tests;
