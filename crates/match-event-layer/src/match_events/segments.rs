//! 入力セグメント（同一入力の継続区間）の構築
//!
//! match_events.rs からの機械的分割（挙動不変）。

use super::*;

/// 確定層トラッカー出力を同一入力のセグメントに畳む。
pub(crate) fn build_segments(
    features: &[FrameFeatures],
    inputs: &[TrackedInput],
) -> Vec<InputSegment> {
    let n = inputs.len().min(features.len());
    let mut out: Vec<InputSegment> = Vec::new();
    let mut cur: Option<(usize, usize)> = None; // (start_i, last_i)

    let key = |t: &TrackedInput| -> (InputDir, Vec<String>, bool, bool) {
        (
            t.dir,
            t.badges.iter().map(|b| b.label().to_string()).collect(),
            t.auto,
            t.throw,
        )
    };

    let flush = |cur: &mut Option<(usize, usize)>,
                 out: &mut Vec<InputSegment>,
                 inputs: &[TrackedInput],
                 features: &[FrameFeatures]| {
        if let Some((a, b)) = cur.take() {
            let t = &inputs[a];
            out.push(InputSegment {
                start_frame: features[a].frame_index,
                end_frame: features[b].frame_index,
                dir: t.dir.as_str().to_string(),
                badges: t.badges.iter().map(|x| x.label().to_string()).collect(),
                auto: t.auto,
                throw: t.throw,
                evidence: InputEvidence {
                    observed_frames: inputs[a..=b]
                        .iter()
                        .filter(|input| !input.uncertain && !input.repaired)
                        .count() as u32,
                    repaired_frames: inputs[a..=b]
                        .iter()
                        .filter(|input| !input.uncertain && input.repaired)
                        .count() as u32,
                },
            });
        }
    };

    for i in 0..n {
        let t = &inputs[i];
        // 試合画面外・不確定・空読みはセグメントを切る
        let valid = features[i].is_match_screen && !t.uncertain && t.count.is_some();
        if !valid {
            flush(&mut cur, &mut out, inputs, features);
            continue;
        }
        match cur {
            None => cur = Some((i, i)),
            Some((a, last)) => {
                let same_key = key(t) == key(&inputs[a]);
                // count の減少 = 新しい入力（同一キーでも別入力として切る）
                let count_reset = match (inputs[last].count, t.count) {
                    (Some(p), Some(c)) => c < p,
                    _ => false,
                };
                if same_key && !count_reset {
                    cur = Some((a, i));
                } else {
                    flush(&mut cur, &mut out, inputs, features);
                    cur = Some((i, i));
                }
            }
        }
    }
    flush(&mut cur, &mut out, inputs, features);
    out
}

#[cfg(test)]
mod tests;
