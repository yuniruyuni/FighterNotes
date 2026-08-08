//! ガード入力崩れ（入力・メーター・HP の 3 点一致）の抽出
//!
//! match_events.rs からの機械的分割（挙動不変）。

use super::*;

/// 入力方向がガード（back / down-back）を維持しているか。
///
/// 画面反転に頑健にするため、ブロック時に実際に握っていた方向から
/// ガード側（左右どちら向きの back か）を導出して判定する。
pub(crate) fn guard_side_set(dir: &str) -> Option<[&'static str; 2]> {
    match dir {
        "R" | "DR" => Some(["R", "DR"]), // 右向きガード（P2 が左を向いている）
        "L" | "DL" => Some(["L", "DL"]), // 左向きガード（P1 が右を向いている）
        _ => None,
    }
}

/// 指定フレームの「確実に観測された」入力方向を返す。
///
/// 遮蔽・スーパー演出中はパネルが読めず（uncertain）、トラッカーが補修
/// （repaired）した推測値になる。ガード入力崩れは「入力が実際にガードから
/// 外れたのを目視できる」ことが要件なので、補修値では判定しない。
pub(crate) fn observed_dir(inputs: &[TrackedInput], frame: u32) -> Option<&str> {
    let t = inputs.get(frame as usize)?;
    if t.uncertain || t.repaired {
        return None;
    }
    match t.dir {
        InputDir::Unknown => None,
        d => Some(d.as_str()),
    }
}

/// ガード方向を握っていたのが崩れて「上（ジャンプ）or 前（歩き）」へ抜けたか。
pub(crate) fn broke_direction(gset: [&str; 2], hd: &str) -> bool {
    match gset {
        // ガード=右（back=右）→ 崩れ = 上 or 前（左方向）
        ["R", "DR"] => matches!(hd, "U" | "UR" | "UL" | "L" | "DL"),
        // ガード=左（back=左）→ 崩れ = 上 or 前（右方向）
        ["L", "DL"] => matches!(hd, "U" | "UR" | "UL" | "R" | "DR"),
        _ => false,
    }
}

/// ガード入力崩れを抽出する。
///
/// 入力・メーター・HP の 3 点一致（ユーザー定義）:
///   - 被弾直前、ガード方向（back / down-back）を握って **ブロック硬直**
///     （メーター=stun かつ HP 平坦 = 被弾していない）だった＝実際に守っていた
///   - 入力がそこから「上（ジャンプ）or 前（歩き）」へ意図的に外れた（観測値のみ）
///   - その非ガード状態で打撃を喰らって STUN + HP 減少
///
/// ブロック硬直で判定するため、離散的な block コンタクトが立たない連続
/// ブロック（f8995 実測）も拾える。HP 平坦の要求で被コンボの継続ヒットを、
/// stun 限定で自分の技の後隙（punish_counter）からの被弾を除外する。
/// 一度ガードを外して被弾した後のダメージ片は、直前 HP が平坦ではないため
/// 同じ判断の2回目として数えない。
/// PreJumpClipped（ブロックから上に外れて予備動作を狩られた）は崩れ本体
/// なので除外しない。対空された空中ジャンプ（GotHit）・無敵技・投げは除外。
#[allow(clippy::too_many_arguments)]
pub(crate) fn extract_guard_breaks(
    damage: &[DamageEvent],
    meter_state: &[Vec<MeterState>; 2],
    hp: &[Vec<f32>; 2],
    inputs: [&[TrackedInput]; 2],
    jumps: &[JumpEvent],
    throws: &[ThrowEvent],
    reversals: &[ReversalEvent],
    rounds: &[RoundInfo],
) -> Vec<GuardBreakEvent> {
    if meter_state[0].is_empty() {
        return Vec::new();
    }
    let n = meter_state[0].len();
    let mut out = Vec::new();
    for d in damage.iter().filter(|d| d.drop >= GUARD_BREAK_MIN_DROP) {
        let victim = d.victim;
        let vi = victim as usize - 1;
        let fh = d.start_frame;
        let fhi = (fh as usize).min(n - 1);
        let own = &meter_state[vi];
        let hpv = &hp[vi];

        // ── 被弾直前に HP が平坦だったか（被コンボの継続ヒットを除外） ──
        // ブロック硬直中は HP が減らない。被弾中（コンボ）は減っている
        let lo = fhi.saturating_sub(GB_LOOKBACK);
        let (mut hmin, mut hmax) = (f32::MAX, f32::MIN);
        for &v in &hpv[lo..fhi] {
            hmin = hmin.min(v);
            hmax = hmax.max(v);
        }
        if hmax - hmin > 0.01 {
            continue; // 被弾中だった＝ブロックではない
        }

        // ── ブロック硬直（stun + ガード方向保持）だった区間を確認 ──────
        let mut block_frames = 0usize;
        let mut gset: Option<[&str; 2]> = None;
        let mut guard_dir = String::new();
        for f in (lo..fhi).rev() {
            if own[f] != MeterState::Stun {
                continue;
            }
            if let Some(gd) = observed_dir(inputs[vi], f as u32) {
                if let Some(gs) = guard_side_set(gd) {
                    block_frames += 1;
                    if gset.is_none() {
                        gset = Some(gs);
                        guard_dir = gd.to_string();
                    }
                }
            }
        }
        let (Some(gset), true) = (gset, block_frames >= GB_MIN_BLOCK) else {
            continue;
        };

        // ── ブロック〜被弾の間に自分が攻撃していないこと ──────────────
        // Startup/Active/Recovery/Invincible = 技を振った（暴れ・確反負け）。
        // motion_recovery（ジャンプ・移動）は許容（崩れの本体）
        let acted = (lo..=fhi).any(|k| {
            matches!(
                own.get(k),
                Some(MeterState::Startup)
                    | Some(MeterState::Active)
                    | Some(MeterState::ProjectileActive)
                    | Some(MeterState::Invincible)
                    | Some(MeterState::Recovery)
            )
        });
        if acted {
            continue;
        }

        // ── 被弾直前に「意図的に」ガードから外れた入力（観測値のみ） ──
        // 上（ジャンプ）or 前（歩き）。被弾の 3F 前〜被弾フレームで探す
        let broke_to = (fh.saturating_sub(3)..=fh)
            .rev()
            .find_map(|f| observed_dir(inputs[vi], f).filter(|hd| broke_direction(gset, hd)));
        let Some(broke_to) = broke_to else { continue };

        // ── 帰属排他 ──────────────────────────────────────────────────
        // 対空された空中ジャンプ（GotHit）は own_jumps の領分。ただし
        // PreJumpClipped（地上の予備動作狩られ = 崩れ本体）は除外しない
        let jump_attributed = jumps.iter().any(|j| {
            j.side == victim
                && j.takeoff_confirmed
                && j.outcome == JumpOutcome::GotHit
                && fh >= j.frame
                && fh <= j.frame + JUMP_SELF_HIT_WINDOW
        });
        let reversal_attributed = reversals.iter().any(|r| {
            r.side == victim
                && fh + 5 >= r.frame
                && fh <= r.frame + REVERSAL_WINDOW + REVERSAL_PUNISH_WINDOW
        });
        let throw_attributed = throws.iter().any(|t| {
            t.thrower == 3 - victim && t.connected && fh + 5 >= t.frame && fh <= t.frame + 40
        });
        if jump_attributed || reversal_attributed || throw_attributed {
            continue;
        }

        if let Some(round_no) = round_of(rounds, fh) {
            out.push(GuardBreakEvent {
                side: victim,
                frame: fh,
                drop: d.drop,
                guard_dir,
                broke_to: broke_to.to_string(),
                round_no,
            });
        }
    }
    out.sort_by_key(|g| g.frame);
    out
}
