//! 確定反撃の機会（相手の後隙 × 自分が行動可能）と結果の抽出
//!
//! match_events.rs からの機械的分割（挙動不変）。

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
    if meter_state[0].is_empty() {
        return Vec::new();
    }
    let n = meter_state[0].len();
    let mut out = Vec::new();
    for s in 0..2usize {
        let me = s as u8 + 1;
        let own = &meter_state[s];
        let opp = &meter_state[1 - s];
        let own_epoch = &meter_epoch[s];
        let opp_epoch = &meter_epoch[1 - s];
        let opp_gf = &meter_gf[1 - s];
        // 相手の Recovery run を列挙
        let mut i = 0usize;
        while i < n {
            if opp[i] != MeterState::Recovery {
                i += 1;
                continue;
            }
            let rs = i;
            let recovery_epoch = opp_epoch.get(rs).copied().unwrap_or(-1);
            while i < n
                && opp[i] == MeterState::Recovery
                && opp_epoch.get(i).copied() == Some(recovery_epoch)
            {
                i += 1;
            }
            let re = i - 1;
            if recovery_epoch < 0 || continuous_epoch(opp_epoch, rs, re) != Some(recovery_epoch) {
                continue;
            }
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
            let meter_block_frame = (rs.saturating_sub(PUNISH_MISSED_LOOKBACK as usize)..rs)
                .rev()
                .find(|&k| {
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
                .map(|k| features[k].frame_index);
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
                continue;
            }

            // 自分が行動可能になった最初のフレーム。stun 明けに即攻撃した場合
            // （先行入力の確反）は Free を経由せず Startup に直行するので、
            // 「stun でない最初のフレーム」を採用する（f8721 実測）。
            // Invincible（自分の SA・無敵技の発生）は「相手の後隙への反撃」
            // ではないので機会の起点にしない
            let Some(t) = (rs..=re).find(|&k| {
                own_epoch.get(k).copied() == Some(recovery_epoch)
                    && matches!(own[k], MeterState::Free | MeterState::Startup)
            }) else {
                continue;
            };
            // 機会窓内に自分の無敵技発生があれば、それは自分から撃った
            // SA/無敵技であって確反ではない
            if (t..=re.min(n - 1)).any(|k| own[k] == MeterState::Invincible) {
                continue;
            }
            // 有利フレームは game frame で数える。ヒットストップ・演出で
            // メーターが停止している間も video frame は進むため、video frame の
            // 引き算では停止分だけ過大になる（実 +2 が +12 と表示された実例）。
            // 写像が無い場合（相手 run 外・メーターなし）は video frame に
            // フォールバックする
            let adv = match (opp_gf.get(t), opp_gf.get(re)) {
                (Some(&g0), Some(&g1)) if g0 >= 0 && g1 >= g0 => (g1 - g0 + 1) as u32,
                _ => (re - t + 1) as u32,
            };
            if adv < PUNISH_MIN_ADV {
                continue;
            }
            let t_frame = features[t].frame_index;
            let Some(round_no) = round_of(rounds, t_frame) else {
                continue;
            };

            // 自分のジャンプ（上入力セグメント）中の空振りはジャンプ攻撃の
            // 話であって地上確反ではない（own_jumps カードの領分。
            // ジャンプ 1 サイクル = 45F + 予備 4F ぶんを重なりとみなす）
            let in_own_jump = segments[s].iter().any(|g| {
                matches!(g.dir.as_str(), "U" | "UR" | "UL")
                    && t_frame + 5 >= g.start_frame
                    && t_frame <= g.start_frame + 49
            });
            if in_own_jump {
                continue;
            }

            // 自分の攻撃開始（Startup 突入）を機会窓内で探す
            let attack_start = (t..=re.min(n - 1)).find(|&k| {
                own_epoch.get(k).copied() == Some(recovery_epoch) && own[k] == MeterState::Startup
            });
            if attack_start.is_none() {
                // 攻撃していない。直前の block コンタクトは時間上の候補を
                // 裏付けるが、長い手足の先端ガードでは本体同士が離れている。
                // ここでは距離を Unknown のまま保持し、空間解析後だけ助言する。
                // 攻撃しなかった見逃し候補は、誤検出時に行動の裏付けがない。
                // そのため補助的なメーター一致では増やさず、ContactEvent で
                // ガードを確認できた場面だけ保持する。
                if let Some(block_frame) = contact_block_frame {
                    out.push(PunishChance {
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
                }
                continue;
            }
            let attack_start = attack_start.unwrap();
            let attack_start_frame = features[attack_start].frame_index;
            // Startup は「押した」時刻にすぎない。通常技の Active が相手の
            // 後隙中に出て初めて時間上の反撃になる。ProjectileActive は弾の
            // 生成時刻なので、後段で接触時刻と分けて扱う。
            let active_search_end = (re + PUNISH_FOLLOWUP_WINDOW as usize).min(n - 1);
            let attack_active = (attack_start..=active_search_end).find(|&k| {
                own_epoch.get(k).copied() == Some(recovery_epoch)
                    && matches!(own[k], MeterState::Active | MeterState::ProjectileActive)
            });
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
                continue;
            }
            let success = hit;
            if !success {
                // 攻撃していない場面と同様、失敗を断定できるのはガード起点
                // だけ。遠距離で相手の通常技と自分の牽制が重なっただけの
                // ニュートラル交換はここで棄却する。
                if source_block_frame.is_none() {
                    continue;
                }
                // 弾の ProjectileActive は生成時刻であって着弾時刻ではない。
                // 接触が確認できない弾を「届かなかった確反」とは断定しない。
                if projectile_attempt {
                    continue;
                }
                // ボタンを押していても、攻撃判定が後隙終了までに出ていなければ
                // 距離ミスではなく単に時間上間に合っていない（または判定不明）。
                if attack_active.is_none_or(|active| active > re) {
                    continue;
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
            let pressed = segments[s]
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

            out.push(PunishChance {
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
            });
        }
    }
    out.sort_by_key(|p| p.frame);
    out
}
