use super::support::*;

#[test]
fn test_mashing_excludes_jump_attributed_damage() {
    use crate::match_events::InputSegment;
    let mut ev = empty_events();
    // 被圧の材料: 小被弾（f900 終了）
    ev.damage.push(DamageEvent {
        victim: 1,
        start_frame: 880,
        pre_freeze_frame: 880,
        end_frame: 900,
        hp_before: 1.0,
        hp_after: 0.96,
        drop: 0.04,
        round_no: 1,
    });
    // 大被弾 2 件: f1000（純正暴れ）と f1200（ジャンプ迎撃の帰結）。
    // どちらも直前 240F 以内に被弾があり被圧文脈を満たす
    for f in [1000u32, 1200] {
        ev.damage.push(DamageEvent {
            victim: 1,
            start_frame: f,
            pre_freeze_frame: f,
            end_frame: f + 20,
            hp_before: 0.9,
            hp_after: 0.78,
            drop: 0.12,
            round_no: 1,
        });
    }
    // どちらの被弾も直前にボタン押下がある
    let press = |f: u32| InputSegment {
        start_frame: f,
        end_frame: f + 5,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    };
    ev.segments[0] = vec![press(990), press(1190)];
    // f1200 の被弾はジャンプ f1160 GotHit の帰結（+40F）
    ev.jumps.push(JumpEvent {
        side: 1,
        frame: 1160,
        outcome: JumpOutcome::GotHit,
        input_dir: "UR".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: None,
        takeoff_confirmed: true,
        air_end: (1160) + 47,
        round_no: 1,
    });

    let report = build_report(&[], &ev, "p1", None);
    // ジャンプ帰属の f1200 は除外され、純正暴れ f1000 だけを確認場面にする。
    let card = report
        .cards
        .iter()
        .find(|card| card.id == "mashing")
        .unwrap();
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_invites_user_review(card);
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.evidence[0].frame, 990);

    // ジャンプが無ければ両方暴れとして計上される（対照実験）
    ev.jumps.clear();
    let report = build_report(&[], &ev, "p1", None);
    let card = report
        .cards
        .iter()
        .find(|c| c.id == "mashing")
        .expect("対照ではカードが出るべき");
    assert_eq!(card.evidence.len(), 2);
    assert_eq!(card.evidence[0].frame, 990, "判断した入力を開始点にする");
    assert_eq!(card.evidence[0].end_frame, Some(1020));
    assert_eq!(card.confidence, EventConfidence::Medium);
    assert!(!card.description.contains("ニュートラル"));
    assert!(card.description.contains("相手の攻めを受けている途中"));
}
