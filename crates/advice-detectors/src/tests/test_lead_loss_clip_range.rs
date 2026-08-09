use super::support::*;

#[test]
fn test_lead_loss_clip_range() {
    // own(P1) が f1000 で相手を 0.6 に落としてリード 0.4 を f3000 まで維持
    // → f3000 から own が転落し f3250 過ぎに相手にリードを許す
    let mut ev = empty_events();
    for i in 0..6000usize {
        ev.hp[1][i] = if i < 1000 { 1.0 } else { 0.6 };
        ev.hp[0][i] = if i < 3000 {
            1.0
        } else if i < 3500 {
            1.0 - 0.8 * (i - 3000) as f32 / 500.0
        } else {
            0.2
        };
    }
    let rs = RoundSummary {
        round_no: 1,
        start_frame: 0,
        end_frame: 5999,
        won: Some(false),
        own_hp_end: 0.2,
        opp_hp_end: 0.6,
        own_hp_lost: 0.8,
        opp_hp_lost: 0.4,
        own_hits_taken: 2,
        early_hit: false,
        own_burnouts: 0,
        detection_confidence: "high".to_string(),
    };
    let card = detect_lead_loss(&ev, &[rs], 0).expect("lead_loss カード");
    assert_invites_user_review(&card);
    let e = &card.evidence[0];
    // 開始 = 最大リードの最後の瞬間（≈f3000。ラウンド開始 f0 ではない）
    assert!(
        (2995..=3005).contains(&e.frame),
        "開始は最大リード時点: f{}",
        e.frame
    );
    // 終端 = 相手にリードを許した瞬間（own < 0.6 になる ≈f3250）
    let end = e.end_frame.expect("区間クリップ");
    assert!((3245..=3260).contains(&end), "終端は逆転時点: f{end}");
}
