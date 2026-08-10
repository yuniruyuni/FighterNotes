//! ContactEvent（ヒットストップ signature によるヒット/ガード接触）の抽出
//!
//! match_events.rs からの機械的分割（挙動不変）。

use super::*;

/// メータータイムラインからコンタクトイベントを抽出する。
///
/// 両者の「dwell ≥ PAUSE_MIN の停止セル」を突き合わせ、停止スパンが
/// 重なっていて片側が active/projectile_active・もう片側が stun なら
/// 攻撃の接触。両者 stun は相打ち（双方向に記録）。
/// hit/block の分類は被弾側の HP 減少（damage イベント）の有無で行う
/// （ガード硬直も stun 表示 + ブロックストップのため、メーター単体では
/// 区別できない）。
pub(crate) fn extract_contacts(
    left: &MeterTimeline,
    right: &MeterTimeline,
    damage: &[DamageEvent],
    rounds: &[RoundInfo],
) -> Vec<ContactEvent> {
    let paused = |tl: &MeterTimeline| -> Vec<(i64, i64, String, i32)> {
        let mut v: Vec<(i64, i64, String, i32)> = tl
            .segments
            .iter()
            .flat_map(|segment| {
                segment.entries.iter().map(|entry| {
                    (
                        entry.video_frame_first,
                        entry.video_frame_last,
                        entry.state.clone(),
                        segment.segment_id,
                    )
                })
            })
            .filter(|entry| entry.1 - entry.0 + 1 >= PAUSE_MIN)
            .collect();
        v.sort_by_key(|p| p.0);
        v
    };
    let attacking = |st: &str| st == "active" || st == "projectile_active";

    let lp = paused(left);
    let rp = paused(right);
    let mut out: Vec<ContactEvent> = Vec::new();
    for a in &lp {
        for b in &rp {
            // メーターリセット前後の停止セルを同一接触へ結び付けない。
            if a.3 != b.3 {
                continue;
            }
            let overlap = a.1.min(b.1) - a.0.max(b.0) + 1;
            if overlap < PAUSE_OVERLAP_MIN {
                continue;
            }
            let frame = a.0.max(b.0).max(0) as u32;
            let Some(round_no) = round_of(rounds, frame) else {
                continue;
            };
            let mut pairs: Vec<(u8, u8, bool)> = Vec::new(); // (attacker, victim, projectile)
            if attacking(&a.2) && b.2 == "stun" {
                pairs.push((1, 2, a.2 == "projectile_active"));
            } else if attacking(&b.2) && a.2 == "stun" {
                pairs.push((2, 1, b.2 == "projectile_active"));
            } else if a.2 == "stun" && b.2 == "stun" {
                pairs.push((1, 2, false));
                pairs.push((2, 1, false)); // 相打ち（弾かどうか不明なので false）
            }
            for (attacker, victim, projectile) in pairs {
                let hit = damage.iter().any(|d| {
                    d.victim == victim && d.start_frame + 5 >= frame && d.start_frame <= frame + 25
                });
                out.push(ContactEvent {
                    frame,
                    attacker,
                    victim,
                    hit,
                    projectile,
                    round_no,
                });
            }
        }
    }
    out.sort_by_key(|c| (c.frame, c.attacker));
    out.dedup_by(|x, y| x.frame == y.frame && x.attacker == y.attacker && x.victim == y.victim);
    out
}

#[cfg(test)]
mod tests;
