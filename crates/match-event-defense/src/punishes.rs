//! 確定反撃の機会（相手の後隙 × 自分が行動可能）と結果の抽出
//!
//! match_events.rs からの機械的分割（挙動不変）。

use super::runs::{runs_of, MeterRun};
use super::*;

pub struct PunishInputs<'a> {
    pub features: &'a [FrameFeatures],
    pub meter_state: &'a [Vec<MeterState>; 2],
    pub meter_epoch: &'a [Vec<i32>; 2],
    pub meter_game_frame: &'a [Vec<i64>; 2],
    pub contacts: &'a [ContactEvent],
    pub damage: &'a [DamageEvent],
    pub segments: &'a [Vec<InputSegment>; 2],
    pub rounds: &'a [RoundInfo],
}

/// 確定反撃の機会（相手の後隙 × 自分が行動可能）と結果を抽出する。
///
/// 距離情報が無いため、攻撃しなかった場面は時間上の候補としてだけ記録する。
/// `PunishReachability::Confirmed` への昇格は後段の空間解析に任せる。
pub fn extract_punishes(inputs: PunishInputs<'_>) -> Vec<PunishChance> {
    let PunishInputs {
        features,
        meter_state,
        meter_epoch,
        meter_game_frame: meter_gf,
        contacts,
        damage,
        segments,
        rounds,
    } = inputs;
    let n = meter_state[0].len();
    let Some(last_index) = n.checked_sub(1) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for s in 0..2usize {
        let me = s as u8 + 1;
        let own = &meter_state[s];
        let opp = &meter_state[1 - s];
        let own_epoch = &meter_epoch[s];
        let opp_epoch = &meter_epoch[1 - s];
        let opp_gf = &meter_gf[1 - s];
        // 相手の Recovery run を列挙
        for MeterRun {
            start: rs,
            end: re,
            epoch: recovery_epoch,
        } in runs_of(opp, opp_epoch, MeterState::Recovery)
        {
            #[allow(clippy::too_many_arguments)]
            fn chance_from_run(
                features: &[FrameFeatures],
                own: &[MeterState],
                opp: &[MeterState],
                own_epoch: &[i32],
                opp_epoch: &[i32],
                opp_gf: &[i64],
                contacts: &[ContactEvent],
                damage: &[DamageEvent],
                own_segments: &[InputSegment],
                rounds: &[RoundInfo],
                last_index: usize,
                me: u8,
                rs: usize,
                re: usize,
                recovery_epoch: i32,
            ) -> Option<PunishChance> {
                let rs_frame = features[rs].frame_index;
                let re_frame = features[re].frame_index;

                // Recovery 表示だけでは、遠距離で相手が技を振っただけなのか、
                // ガード後の確反なのかを区別できない。直前の block を起点として
                // 保持し、攻撃が当たらなかった候補にはこの因果確認を必須にする。
                let contact_block_frame = contacts
                    .iter()
                    .filter(|c| {
                        c.attacker == 3 - me
                            && !c.hit
                            && c.frame <= rs_frame
                            && c.frame + PUNISH_MISSED_LOOKBACK >= rs_frame
                            && {
                                let frame = idx_of(features, c.frame);
                                own_epoch.get(frame).copied() == Some(recovery_epoch)
                                    && opp_epoch.get(frame).copied() == Some(recovery_epoch)
                            }
                    })
                    .max_by_key(|c| c.frame)
                    .map(|contact| contact.frame);
                // 短い接触ではヒットストップの dwell 条件を満たさず ContactEvent が
                // 作れない場合がある（2026-06-16 f8690 実測）。相手 Active と自分
                // Stun の同時表示があり、近傍に HP 減少がなければ block の補助証拠
                // とする。ヒットは damage で棄却するため、コンボ中の stun を
                // ガード起点へ取り違えない。
                let meter_block_frame = own
                    .iter()
                    .enumerate()
                    .take(rs)
                    .rev()
                    .take(PUNISH_MISSED_LOOKBACK as usize)
                    .find(|&(k, _)| {
                        own_epoch.get(k).copied() == Some(recovery_epoch)
                            && opp_epoch.get(k).copied() == Some(recovery_epoch)
                            && matches!(opp[k], MeterState::Active | MeterState::ProjectileActive)
                            && own[k] == MeterState::Stun
                            && !damage.iter().any(|d| {
                                let frame = features[k].frame_index;
                                d.victim == me
                                    && d.start_frame + PUNISH_CONTACT_ALIGNMENT_GRACE >= frame
                                    && d.start_frame <= frame + PUNISH_MISSED_LOOKBACK
                            })
                    })
                    .map(|(k, _)| features[k].frame_index);
                let source_block_frame = contact_block_frame.or(meter_block_frame);

                // 因果検証: この後隙が「自分の攻撃をガード/被弾した結果」なら
                // 確反機会ではない（ケース2 実測: 自分の SA3 をガードさせた後の
                // ガード硬直明けを後隙と誤認していた）。後隙開始の直前に自分が
                // attacker のコンタクトがあれば除外
                let caused_by_me = contacts.iter().any(|c| {
                    c.attacker == me
                        && c.frame <= rs_frame
                        && c.frame + PUNISH_CAUSE_LOOKBACK >= rs_frame
                        && {
                            let frame = idx_of(features, c.frame);
                            own_epoch.get(frame).copied() == Some(recovery_epoch)
                                && opp_epoch.get(frame).copied() == Some(recovery_epoch)
                        }
                });
                if caused_by_me {
                    return None;
                }

                // 自分が行動可能になった最初のフレーム。stun 明けに即攻撃した場合
                // （先行入力の確反）は Free を経由せず Startup に直行するので、
                // 「stun でない最初のフレーム」を採用する（f8721 実測）。
                // Invincible（自分の SA・無敵技の発生）は「相手の後隙への反撃」
                // ではないので機会の起点にしない
                let t = first_actionable(own, own_epoch, recovery_epoch, rs, re)?;
                // 機会窓内に自分の無敵技発生があれば、それは自分から撃った
                // SA/無敵技であって確反ではない
                if has_invincible(own, t, re.min(last_index)) {
                    return None;
                }
                // 有利フレームは game frame で数える。ヒットストップ・演出で
                // メーターが停止している間も video frame は進むため、video frame の
                // 引き算では停止分だけ過大になる（実 +2 が +12 と表示された実例）。
                // 写像が無い場合（相手 run 外・メーターなし）は video frame に
                // フォールバックする
                let adv = inclusive_advantage(opp_gf, t, re);
                if !is_punishable_advantage(adv) {
                    return None;
                }
                let t_frame = features[t].frame_index;
                let round_no = round_of(rounds, t_frame)?;

                // 自分のジャンプ（上入力セグメント）中の空振りはジャンプ攻撃の
                // 話であって地上確反ではない（own_jumps カードの領分。
                // ジャンプ 1 サイクル = 45F + 予備 4F ぶんを重なりとみなす）
                let in_own_jump = own_segments.iter().any(|g| {
                    matches!(g.dir.as_str(), "U" | "UR" | "UL")
                        && t_frame + 5 >= g.start_frame
                        && t_frame <= g.start_frame + 49
                });
                if in_own_jump {
                    return None;
                }

                // 自分の攻撃開始（Startup 突入）を機会窓内で探す
                let attack_start =
                    first_attack_start(own, own_epoch, recovery_epoch, t, re.min(last_index));
                let Some(attack_start) = attack_start else {
                    // 攻撃していない。直前の block コンタクトは時間上の候補を
                    // 裏付けるが、長い手足の先端ガードでは本体同士が離れている。
                    // ここでは距離を Unknown のまま保持し、空間解析後だけ助言する。
                    // 攻撃しなかった見逃し候補は、誤検出時に行動の裏付けがない。
                    // そのため補助的なメーター一致では増やさず、ContactEvent で
                    // ガードを確認できた場面だけ保持する。
                    return contact_block_frame.map(|block_frame| PunishChance {
                        frame: t_frame,
                        side: me,
                        advantage: adv,
                        outcome: PunishOutcome::Missed,
                        origin: PunishOrigin::BlockedMove,
                        recovery_start_frame: rs_frame,
                        recovery_end_frame: re_frame,
                        source_contact_frame: Some(block_frame),
                        attack_start_frame: None,
                        attack_active_frame: None,
                        reachability: PunishReachability::Unknown,
                        punished_drop: 0.0,
                        pressed: String::new(),
                        round_no,
                    });
                };
                let attack_start_frame = features[attack_start].frame_index;
                // Startup は「押した」時刻にすぎない。通常技の Active が相手の
                // 後隙中に出て初めて時間上の反撃になる。ProjectileActive は弾の
                // 生成時刻なので、後段で接触時刻と分けて扱う。
                let active_search_end = re
                    .saturating_add(PUNISH_FOLLOWUP_WINDOW as usize)
                    .min(last_index);
                let attack_active = first_attack_active(
                    own,
                    own_epoch,
                    recovery_epoch,
                    attack_start,
                    active_search_end,
                );
                let attack_active_frame = attack_active.map(|k| features[k].frame_index);
                let projectile_attempt =
                    attack_active.is_some_and(|k| own[k] == MeterState::ProjectileActive);
                // 成否判定:
                //   Success = 後隙終端との境界ずれ範囲内に自分のヒットコンタクト
                //   接触したがガードされた（block）= 空振りではない → 機会自体を除外
                //     （「届いている」ので指摘対象外。ケース1 実測）
                //   WhiffFail = 機会窓内に自分の attacker コンタクトが一切ない
                let hit = contacts.iter().any(|c| {
                    c.attacker == me
                        && c.hit
                        && c.frame >= t_frame
                        && c.frame <= re_frame + PUNISH_CONTACT_ALIGNMENT_GRACE
                        && {
                            let frame = idx_of(features, c.frame);
                            own_epoch.get(frame).copied() == Some(recovery_epoch)
                                && opp_epoch.get(frame).copied() == Some(recovery_epoch)
                        }
                });
                let any_contact = contacts.iter().any(|c| {
                    c.attacker == me
                        && c.frame >= t_frame
                        && c.frame <= re_frame + PUNISH_FOLLOWUP_WINDOW
                        && {
                            let frame = idx_of(features, c.frame);
                            own_epoch.get(frame).copied() == Some(recovery_epoch)
                                && opp_epoch.get(frame).copied() == Some(recovery_epoch)
                        }
                });
                // 接触したがヒットしていない（＝ガードされた）→ 空振りではないので
                // このカードの対象外
                if any_contact && !hit {
                    return None;
                }
                let success = hit;
                if !success {
                    // 攻撃していない場面と同様、失敗を断定できるのはガード起点
                    // だけ。遠距離で相手の通常技と自分の牽制が重なっただけの
                    // ニュートラル交換はここで棄却する。
                    source_block_frame?;
                    // 弾の ProjectileActive は生成時刻であって着弾時刻ではない。
                    // 接触が確認できない弾を「届かなかった確反」とは断定しない。
                    if projectile_attempt {
                        return None;
                    }
                    // ボタンを押していても、攻撃判定が後隙終了までに出ていなければ
                    // 距離ミスではなく単に時間上間に合っていない（または判定不明）。
                    if attack_active.is_none_or(|active| active > re) {
                        return None;
                    }
                }
                // 空振り後の被弾
                let punished_drop = if success {
                    0.0
                } else {
                    damage
                        .iter()
                        .filter(|d| {
                            d.victim == me
                                && d.start_frame >= re_frame
                                && d.start_frame <= re_frame + PUNISH_PUNISHED_WINDOW
                        })
                        .map(|d| d.drop)
                        .fold(0.0f32, f32::max)
                };
                // 使ったボタン（攻撃開始付近の入力バッジ）
                let atk_frame = attack_start_frame;
                let pressed = own_segments
                    .iter()
                    .find(|g| {
                        g.has_button()
                            && g.start_frame + 6 >= atk_frame
                            && g.start_frame <= atk_frame + 2
                    })
                    .map(|g| {
                        let mut parts = g.badges.clone();
                        if g.auto {
                            parts.push("AUTO".to_string());
                        }
                        if g.throw {
                            parts.push("投げ".to_string());
                        }
                        parts.join(" ")
                    })
                    .unwrap_or_default();

                Some(PunishChance {
                    frame: t_frame,
                    side: me,
                    advantage: adv,
                    outcome: if success {
                        PunishOutcome::Success
                    } else {
                        PunishOutcome::WhiffFail
                    },
                    origin: if source_block_frame.is_some() {
                        PunishOrigin::BlockedMove
                    } else {
                        PunishOrigin::VerifiedWhiff
                    },
                    recovery_start_frame: rs_frame,
                    recovery_end_frame: re_frame,
                    source_contact_frame: source_block_frame,
                    attack_start_frame: Some(attack_start_frame),
                    attack_active_frame,
                    reachability: if success {
                        PunishReachability::Confirmed
                    } else {
                        PunishReachability::Unknown
                    },
                    punished_drop,
                    pressed,
                    round_no,
                })
            }
            let chance = chance_from_run(
                features,
                own,
                opp,
                own_epoch,
                opp_epoch,
                opp_gf,
                contacts,
                damage,
                &segments[s],
                rounds,
                last_index,
                me,
                rs,
                re,
                recovery_epoch,
            );
            if let Some(chance) = chance {
                out.push(chance);
            }
        }
    }
    out.sort_by_key(|p| p.frame);
    out
}

fn first_actionable(
    states: &[MeterState],
    epochs: &[i32],
    epoch: i32,
    start: usize,
    end: usize,
) -> Option<usize> {
    (start..=end).find(|&index| {
        epochs.get(index).copied() == Some(epoch)
            && matches!(states[index], MeterState::Free | MeterState::Startup)
    })
}

fn has_invincible(states: &[MeterState], start: usize, end: usize) -> bool {
    (start..=end).any(|index| states[index] == MeterState::Invincible)
}

fn inclusive_advantage(game_frames: &[i64], start: usize, end: usize) -> u32 {
    match (game_frames.get(start), game_frames.get(end)) {
        (Some(&first), Some(&last)) if first >= 0 && last >= first => (last - first + 1) as u32,
        _ => (end - start + 1) as u32,
    }
}

fn is_punishable_advantage(advantage: u32) -> bool {
    advantage >= PUNISH_MIN_ADV
}

fn first_attack_start(
    states: &[MeterState],
    epochs: &[i32],
    epoch: i32,
    start: usize,
    end: usize,
) -> Option<usize> {
    (start..=end).find(|&index| {
        epochs.get(index).copied() == Some(epoch) && states[index] == MeterState::Startup
    })
}

fn first_attack_active(
    states: &[MeterState],
    epochs: &[i32],
    epoch: i32,
    start: usize,
    end: usize,
) -> Option<usize> {
    (start..=end).find(|&index| {
        epochs.get(index).copied() == Some(epoch)
            && matches!(
                states[index],
                MeterState::Active | MeterState::ProjectileActive
            )
    })
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::test_support::feat;

    #[test]
    fn state_searches_include_their_end_and_honor_nonzero_epochs() {
        let epochs = vec![7; 6];

        let mut actionable = vec![MeterState::Stun; 6];
        actionable[4] = MeterState::Free;
        assert_eq!(first_actionable(&actionable, &epochs, 7, 2, 4), Some(4));

        let mut invincible = vec![MeterState::Free; 6];
        invincible[4] = MeterState::Invincible;
        assert!(has_invincible(&invincible, 2, 4));

        let mut attack = vec![MeterState::Free; 6];
        attack[4] = MeterState::Startup;
        assert_eq!(first_attack_start(&attack, &epochs, 7, 2, 4), Some(4));
        attack[5] = MeterState::Active;
        assert_eq!(first_attack_active(&attack, &epochs, 7, 4, 5), Some(5));

        let mut before_start = vec![MeterState::Free; 6];
        before_start[0] = MeterState::Invincible;
        assert!(!has_invincible(&before_start, 2, 4));
        before_start[0] = MeterState::Startup;
        before_start[4] = MeterState::Startup;
        assert_eq!(first_attack_start(&before_start, &epochs, 7, 2, 4), Some(4));
        before_start[0] = MeterState::Active;
        before_start[4] = MeterState::Free;
        before_start[5] = MeterState::Active;
        assert_eq!(
            first_attack_active(&before_start, &epochs, 7, 4, 5),
            Some(5)
        );
    }

    #[test]
    fn advantage_uses_valid_inclusive_game_frames_or_the_exact_video_fallback() {
        assert_eq!(inclusive_advantage(&[0], 0, 0), 1);
        assert_eq!(inclusive_advantage(&[0, -1, 5], 1, 2), 2);
        assert_eq!(inclusive_advantage(&[0, 5, 4], 1, 2), 2);
        assert_eq!(inclusive_advantage(&[-1; 5], 2, 4), 3);
        assert_eq!(inclusive_advantage(&[9, 0, 0, 0], 1, 3), 1);
        assert_eq!(inclusive_advantage(&[9, 5, 5, 5], 1, 3), 1);
        assert!(is_punishable_advantage(PUNISH_MIN_ADV));
        assert!(!is_punishable_advantage(PUNISH_MIN_ADV - 1));
    }

    #[test]
    fn a_nonzero_recovery_epoch_is_fully_evaluated() {
        let length = 60;
        let mut own = vec![MeterState::Free; length];
        let mut opponent = vec![MeterState::Free; length];
        own[10..20].fill(MeterState::Stun);
        opponent[10..20].fill(MeterState::Active);
        opponent[20..40].fill(MeterState::Recovery);
        let features: Vec<_> = (0..length)
            .map(|frame| feat(frame as u32, 1.0, 1.0))
            .collect();
        let epochs = [vec![7; length], vec![7; length]];
        let game_frames = [
            (0..length as i64).collect::<Vec<_>>(),
            (0..length as i64).collect::<Vec<_>>(),
        ];
        let contacts = [ContactEvent {
            frame: 10,
            attacker: 2,
            victim: 1,
            hit: false,
            projectile: false,
            round_no: 1,
        }];
        let rounds = [RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: length as u32 - 1,
            winner: None,
            p1_hp_end: 1.0,
            p2_hp_end: 1.0,
        }];

        let chances = extract_punishes(PunishInputs {
            features: &features,
            meter_state: &[own, opponent],
            meter_epoch: &epochs,
            meter_game_frame: &game_frames,
            contacts: &contacts,
            damage: &[],
            segments: &[vec![], vec![]],
            rounds: &rounds,
        });

        assert_eq!(chances.len(), 1);
        assert_eq!(chances[0].outcome, PunishOutcome::Missed);
    }
}
