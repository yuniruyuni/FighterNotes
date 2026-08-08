use super::*;
use meter_tracker::MeterTimeline;
use std::collections::HashSet;

fn takeoff_timing_matches(
    features: &[FrameFeatures],
    meter_gf: &[i64],
    meter_epoch: &[i32],
    run_start_index: usize,
    input_frame: u32,
) -> bool {
    let run_start = features[run_start_index].frame_index;
    if run_start < input_frame.saturating_sub(JUMP_CONFIRM_BACK)
        || run_start > input_frame.saturating_add(JUMP_CONFIRM_FWD)
    {
        return false;
    }
    if run_start >= input_frame {
        return true;
    }

    // 動画上で入力表示が遅れても、ヒットストップ中なら game frame 差は
    // ほぼゼロになる。ゲーム自体が数フレーム以上進んでいる場合、先行する
    // 移動ランをその上入力が発生させたとはいえない。
    let input_index = idx_of(features, input_frame);
    let (Some(&run_gf), Some(&input_gf)) =
        (meter_gf.get(run_start_index), meter_gf.get(input_index))
    else {
        return true;
    };
    if run_gf < 0 || input_gf < 0 {
        return true;
    }
    input_gf >= run_gf
        && input_gf - run_gf <= JUMP_CONFIRM_BACK_GF
        && continuous_epoch(meter_epoch, run_start_index, input_index).is_some()
}

pub(crate) struct JumpInputs<'a> {
    pub(crate) features: &'a [FrameFeatures],
    pub(crate) segments: &'a [Vec<InputSegment>; 2],
    pub(crate) p1_stun: &'a [bool],
    pub(crate) p2_stun: &'a [bool],
    pub(crate) meter: Option<(&'a MeterTimeline, &'a MeterTimeline)>,
    pub(crate) meter_game_frame: &'a [Vec<i64>; 2],
    pub(crate) meter_state: &'a [Vec<MeterState>; 2],
    pub(crate) meter_epoch: &'a [Vec<i32>; 2],
    pub(crate) damage: &'a [DamageEvent],
    pub(crate) contacts: &'a [ContactEvent],
    pub(crate) rounds: &'a [RoundInfo],
    pub(crate) characters: [Option<&'a str>; 2],
}

pub(crate) fn extract_jumps(inputs: JumpInputs<'_>) -> Vec<JumpEvent> {
    let JumpInputs {
        features,
        segments,
        p1_stun,
        p2_stun,
        meter,
        meter_game_frame: meter_gf,
        meter_state,
        meter_epoch,
        damage,
        contacts,
        rounds,
        characters,
    } = inputs;
    let n = features.len();

    let stun = [p1_stun, p2_stun];
    let movementish: [Vec<bool>; 2] = match meter {
        Some((l, r)) => [movementish_per_frame(l, n), movementish_per_frame(r, n)],
        None => [Vec::new(), Vec::new()],
    };
    // (event, 空中ヒットとして扱える終端)。air_end は着地側の小さな
    // マージンも含むため、通常キャラでは別の上限を保持する。
    let mut pending_jumps: Vec<(JumpEvent, u32)> = Vec::new();
    let mut claimed_movement_runs: [HashSet<usize>; 2] = [HashSet::new(), HashSet::new()];
    for s in 0..2usize {
        for seg in &segments[s] {
            if !matches!(seg.dir.as_str(), "U" | "UR" | "UL") {
                continue;
            }
            if seg.end_frame - seg.start_frame + 1 < JUMP_MIN_HOLD {
                continue;
            }
            let mut f0 = seg.start_frame;
            let st = stun[s];
            if !st.is_empty() {
                let a = idx_of(features, seg.start_frame);
                if st.get(a).copied().unwrap_or(false) {
                    // stun 中に始まった上入力: stun 明けまで保持されていれば
                    // 明けフレームで数え直し、stun 中に終わったら不成立
                    let b = idx_of(features, seg.end_frame);
                    match (a..=b).find(|&i| !st.get(i).copied().unwrap_or(false)) {
                        Some(j) => f0 = features[j].frame_index,
                        None => continue,
                    }
                }
            } else {
                // フォールバック: 自分の被弾シーケンス近傍はやられ継続中とみなす
                let in_dmg = damage.iter().any(|d| {
                    d.victim == s as u8 + 1
                        && f0 + 2 >= d.start_frame
                        && f0 <= d.end_frame + JUMP_DMG_TAIL
                });
                if in_dmg {
                    continue;
                }
            }
            // メーター確認: 本物のジャンプは移動系ラン（緑/シアン）が
            // メーターに必ず出る。開幕演出中のスティック遊び・演出中の
            // レバガチャ・入力誤読では出ない（検証済み試合: 偽ジャンプ
            // 5 件すべてラン無しで棄却、本物 2 件は 29gf/38gf のラン）。
            // 短いランは攻撃の発生（同じ緑）と区別できないため、
            // gf 長 >= JUMP_CONFIRM_MIN_GF か「ラン直後に自分が stun」
            // （予備動作狩られ）を要求する
            // 通常キャラは物理上限を固定する。ダルシムのフロート等、明示的に
            // 長滞空を許したキャラだけ確認ランの終端まで延長する。
            let mut air_end = f0 + JUMP_C_HIT_MAX;
            let mut movement_run_start = None;
            let mut takeoff_confirmed = movementish[s].is_empty();
            if !movementish[s].is_empty() {
                let lo = idx_of(features, f0.saturating_sub(JUMP_CONFIRM_BACK));
                let hi = idx_of(features, f0 + JUMP_CONFIRM_FWD).min(n - 1);
                // 窓に重なる全ランを列挙する。窓の左端へ伸びる古いランと、
                // 入力直後に始まる本物の離陸ランが同時に入ることがあるため、
                // 単純な先頭一致では決めない。
                let mut runs = Vec::new();
                for hit_i in (lo..=hi).filter(|&i| movementish[s][i]) {
                    let mut a = hit_i;
                    while a > 0 && movementish[s][a - 1] {
                        a -= 1;
                    }
                    if runs.last().is_some_and(|&(previous, _)| previous == a) {
                        continue;
                    }
                    let mut b = hit_i;
                    while b + 1 < n && movementish[s][b + 1] {
                        b += 1;
                    }
                    runs.push((a, b));
                }
                // 短い攻撃発生ランを先に選んで有効な離陸ランを捨てないよう、
                // 長さ条件を満たす候補だけを順位付けする。入力との時間整合、
                // 地上攻撃チェーンでないこと、未使用であること、距離の順で
                // 優先する。
                let mut candidates = Vec::new();
                for (a, b) in runs {
                    let gf_len = match (meter_gf[s].get(a), meter_gf[s].get(b)) {
                        (Some(&g0), Some(&g1)) if g0 >= 0 && g1 >= g0 => g1 - g0 + 1,
                        _ => (b - a + 1) as i64,
                    };
                    let clipped_into_stun = (b + 1..(b + 4).min(n))
                        .any(|i| meter_state[s].get(i) == Some(&MeterState::Stun));
                    if gf_len < JUMP_CONFIRM_MIN_GF && !clipped_into_stun {
                        continue;
                    }
                    let run_start = features[a].frame_index;
                    let timing_matches =
                        takeoff_timing_matches(features, &meter_gf[s], &meter_epoch[s], a, f0);
                    let ground_attack_chain =
                        movement_run_is_ground_attack_chain(&meter_state[s], &meter_epoch[s], a, b);
                    candidates.push((
                        a,
                        b,
                        timing_matches,
                        ground_attack_chain,
                        claimed_movement_runs[s].contains(&a),
                        run_start.abs_diff(f0),
                    ));
                }
                let Some((a, b, timing_matches, ground_attack_chain, claimed, _)) =
                    candidates.into_iter().min_by_key(
                        |(_, _, timing_matches, ground_attack_chain, claimed, distance)| {
                            (!*timing_matches, *ground_attack_chain, *claimed, *distance)
                        },
                    )
                else {
                    continue;
                };
                movement_run_start = Some(a);
                takeoff_confirmed = !claimed && !ground_attack_chain && timing_matches;
                if crate::frame_data::has_extended_airtime(characters[s]) {
                    air_end = air_end.max(features[b].frame_index + JUMP_LAND_EPS);
                }
            }
            let Some(round_no) = round_of(rounds, f0) else {
                continue;
            };
            // メーターランの同一性が取れないフォールバックでも、方向グリフの
            // 数フレームの揺れだけは同じ入力として統合する。
            if let Some((last, _)) = pending_jumps
                .iter()
                .rev()
                .find(|(j, _)| j.side == s as u8 + 1)
            {
                if f0 <= last.frame + JUMP_INPUT_FRAGMENT_GAP {
                    continue;
                }
                if !takeoff_confirmed && f0 <= last.frame + JUMP_AMBIGUOUS_REUSE_GAP {
                    continue;
                }
            }
            if let (true, Some(a)) = (takeoff_confirmed, movement_run_start) {
                claimed_movement_runs[s].insert(a);
            }
            let air_hit_end = if crate::frame_data::has_extended_airtime(characters[s]) {
                air_end
            } else {
                f0 + JUMP_C_ATK_MAX
            };
            pending_jumps.push((
                JumpEvent {
                    side: s as u8 + 1,
                    frame: f0,
                    outcome: JumpOutcome::Neutral,
                    input_dir: seg.dir.clone(),
                    direction: if seg.dir == "U" {
                        JumpDirection::Neutral
                    } else {
                        JumpDirection::Unknown
                    },
                    contact_frame: None,
                    takeoff_confirmed,
                    air_end,
                    round_no,
                },
                air_hit_end,
            ));
        }
    }
    pending_jumps.sort_by_key(|(j, _)| j.frame);

    // コンタクトを最も近い有効なジャンプへ排他的に割り当てる。被弾側を
    // 優先し、空対空でも同じ接触を LandedHit と GotHit の両方に数えない。
    if !contacts.is_empty() {
        let mut used_contacts = HashSet::new();
        for (contact_i, contact) in contacts.iter().enumerate() {
            let candidate = pending_jumps
                .iter()
                .enumerate()
                .filter(|(_, (jump, air_hit_end))| {
                    jump.outcome == JumpOutcome::Neutral
                        && jump.side == contact.victim
                        && contact.frame >= jump.frame
                        && contact.frame <= jump.air_end
                        // SF6 は空中ガードが無い。HP が演出で読めず contact.hit
                        // が false でも、確認済み離陸の空中窓ならヒット候補として
                        // 空間解析へ送り、接地していれば後段で棄却する。
                        && (contact.hit
                            || (jump.takeoff_confirmed
                                && contact.frame > jump.frame + JUMP_C_PRE_MAX
                                && contact.frame <= *air_hit_end))
                })
                .max_by_key(|(_, (jump, _))| jump.frame)
                .map(|(i, _)| i);
            let Some(jump_i) = candidate else { continue };
            let (jump, air_hit_end) = &mut pending_jumps[jump_i];
            jump.outcome = if !contact.hit {
                JumpOutcome::UnverifiedHit
            } else if contact.frame <= jump.frame + JUMP_C_PRE_MAX {
                JumpOutcome::PreJumpClipped
            } else if contact.frame <= *air_hit_end {
                if jump.takeoff_confirmed {
                    JumpOutcome::GotHit
                } else {
                    JumpOutcome::UnverifiedHit
                }
            } else {
                JumpOutcome::GroundedHit
            };
            jump.contact_frame = Some(contact.frame);
            used_contacts.insert(contact_i);
        }
        for (contact_i, contact) in contacts.iter().enumerate().filter(|(_, c)| c.hit) {
            if used_contacts.contains(&contact_i) {
                continue;
            }
            let candidate = pending_jumps
                .iter()
                .enumerate()
                .filter(|(_, (jump, _))| {
                    if jump.outcome != JumpOutcome::Neutral
                        || jump.side != contact.attacker
                        || contact.frame < jump.frame + JUMP_C_ATK_MIN
                        || contact.frame > jump.frame + JUMP_C_ATK_MAX
                    {
                        return false;
                    }
                    // ジャンプ後のテレポート・無敵技によるヒットは飛び込みへ
                    // 帰属しない（検証済み空中テレポート例）。
                    let side = jump.side as usize - 1;
                    let a = idx_of(features, jump.frame);
                    let b = idx_of(features, contact.frame);
                    !(a..=b).any(|k| meter_state[side].get(k) == Some(&MeterState::Invincible))
                })
                .max_by_key(|(_, (jump, _))| jump.frame)
                .map(|(i, _)| i);
            if let Some(jump_i) = candidate {
                pending_jumps[jump_i].0.outcome = JumpOutcome::LandedHit;
                pending_jumps[jump_i].0.contact_frame = Some(contact.frame);
            }
        }
    } else {
        // メーター無しのフォールバックも、1つのダメージを最も近い
        // ジャンプ候補だけへ割り当てる。
        let mut used_damage = HashSet::new();
        for (damage_i, hit) in damage.iter().enumerate() {
            let candidate = pending_jumps
                .iter()
                .enumerate()
                .filter(|(_, (jump, _))| {
                    jump.outcome == JumpOutcome::Neutral
                        && jump.side == hit.victim
                        && hit.start_frame >= jump.frame + JUMP_SELF_HIT_MIN
                        && hit.start_frame <= jump.frame + JUMP_SELF_HIT_WINDOW
                })
                .max_by_key(|(_, (jump, _))| jump.frame)
                .map(|(i, _)| i);
            if let Some(jump_i) = candidate {
                pending_jumps[jump_i].0.outcome = JumpOutcome::GotHit;
                used_damage.insert(damage_i);
            }
        }
        for (damage_i, hit) in damage.iter().enumerate() {
            if used_damage.contains(&damage_i) {
                continue;
            }
            let candidate = pending_jumps
                .iter()
                .enumerate()
                .filter(|(_, (jump, _))| {
                    jump.outcome == JumpOutcome::Neutral
                        && jump.side == 3 - hit.victim
                        && hit.start_frame >= jump.frame + JUMP_ATTACK_MIN
                        && hit.start_frame <= jump.frame + JUMP_ATTACK_MAX
                })
                .max_by_key(|(_, (jump, _))| jump.frame)
                .map(|(i, _)| i);
            if let Some(jump_i) = candidate {
                pending_jumps[jump_i].0.outcome = JumpOutcome::LandedHit;
            }
        }
    }
    pending_jumps.into_iter().map(|(jump, _)| jump).collect()
}
