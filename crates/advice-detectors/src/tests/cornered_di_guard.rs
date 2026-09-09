//! 画面端でのDIガード(壁やられ)に対する契約。
//!
//! ガード自体は正しい入力なので、端の確認が取れた場面だけを指摘する。
//! 中央でのDIガードや、ヒット・パリィなど別の結末を混ぜると、
//! 「端でだけ回答を変える」という指摘の根拠が崩れる。

use super::support::*;

fn blocked_di(contact_frame: u32) -> DriveImpactEvent {
    DriveImpactEvent {
        side: 2,
        input_frame: contact_frame - 26,
        active_frame: Some(contact_frame - 2),
        contact_frame: Some(contact_frame),
        outcome: DriveImpactOutcome::Blocked,
        damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn wall_splat_damage(start_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim: 1,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame: start_frame + 90,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

fn corner(start_frame: u32, end_frame: u32) -> CornerSpan {
    CornerSpan {
        side: 1,
        start_frame,
        end_frame,
    }
}

/// 端の確認が取れたガード場面だけを数え、壁やられからの反撃を損失として
/// 帰属する。中央のガードは同じ結末にならないので混ぜない。
#[test]
fn only_a_corner_confirmed_di_guard_is_reported() {
    let mut events = empty_events();
    events.corner_spans.push(corner(950, 1000));
    // 数えない場面が先に来ても、後続の探索は止まらない。
    // 接触未確定のDIと、中央でのDIガード。
    events.drive_impacts.push(DriveImpactEvent {
        contact_frame: None,
        ..blocked_di(500)
    });
    events.drive_impacts.push(blocked_di(2000));
    events.drive_impacts.push(blocked_di(1000));
    events.damage.push(wall_splat_damage(1080, 0.30));

    let card = detect_cornered_di_guard(&events, 1).expect("端でのDIガードを提示");
    assert_eq!(card.id, "cornered_di_guard");
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.confidence, EventConfidence::High);
    assert_eq!(card.title, "画面端でDIをガードした場面");
    assert!(card.practice.contains("DI返し"), "{}", card.practice);
    assert_eq!(card.hp_lost, Some(0.30));
    assert_eq!(card.evidence.len(), 1, "中央のガードを混ぜない");
    assert_eq!(card.evidence[0].frame, 974, "DI開始からクリップを再生する");
    assert!(card.evidence[0].label.contains("端でDIをガード"));
    assert!(card.evidence[0].label.contains("-30%"));
    assert_invites_user_review(&card);
    assert!(
        detect_big_hits(&events, 1, &[card]).is_none(),
        "専用カードが同じ大被弾を所有する"
    );
}

/// 繰り返せば診断へ昇格し、損失は各場面の窓の合計になる。
#[test]
fn repeated_cornered_guards_become_a_diagnosis() {
    let mut events = empty_events();
    events.corner_spans.push(corner(950, 1000));
    events.corner_spans.push(corner(2950, 3000));
    events.drive_impacts.push(blocked_di(1000));
    events.drive_impacts.push(blocked_di(3000));
    events.damage.push(wall_splat_damage(1080, 0.30));
    events.damage.push(wall_splat_damage(3100, 0.20));
    // 窓の外の被弾はこの場面の損失として帰属しない。
    events.damage.push(wall_splat_damage(1300, 0.10));

    let card = detect_cornered_di_guard(&events, 1).expect("繰り返した端DIガードを提示");
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert_eq!(card.title, "画面端でDIをガードして壁やられを繰り返している");
    assert!(card.practice.contains("DI返し"), "{}", card.practice);
    assert!((card.hp_lost.unwrap() - 0.50).abs() < 1e-6);
    assert!(
        (card.severity - 0.56).abs() < 1e-3,
        "損失と回数の両方が重みになる: {}",
        card.severity
    );
    assert_eq!(card.evidence.len(), 2);
    assert!(card.description.contains("2 回"));
    assert!(card.description.contains("50%"));
}

/// span 終端の直後は追跡の乱れとみなして端のまま扱うが、猶予を超えたら
/// 端と確認できない。ヒットやパリィなど別の結末、低確度の検出も対象外。
#[test]
fn tail_grace_other_outcomes_and_low_confidence_are_excluded() {
    let mut events = empty_events();
    events.corner_spans.push(corner(900, 970));
    // 終端 + 30F 以内なので端のまま。
    events.drive_impacts.push(blocked_di(1000));

    let card = detect_cornered_di_guard(&events, 1).expect("猶予内の接触を端として扱う");
    assert_eq!(card.evidence.len(), 1);
    assert_eq!(card.hp_lost, Some(0.0), "反撃が無くても場面自体は提示する");

    // 猶予を超えると端と確認できない。
    events.corner_spans[0].end_frame = 900;
    assert!(detect_cornered_di_guard(&events, 1).is_none());

    // 端でも、ガード以外の結末は別のカードの領分。
    events.corner_spans[0].end_frame = 970;
    events.drive_impacts[0].outcome = DriveImpactOutcome::Hit;
    assert!(detect_cornered_di_guard(&events, 1).is_none());

    // 低確度の検出から断定しない。
    events.drive_impacts[0].outcome = DriveImpactOutcome::Blocked;
    events.drive_impacts[0].confidence = EventConfidence::Medium;
    assert!(detect_cornered_di_guard(&events, 1).is_none());

    // 自分のDIがガードされた場面は対象外。
    events.drive_impacts[0].confidence = EventConfidence::High;
    events.drive_impacts[0].side = 1;
    assert!(detect_cornered_di_guard(&events, 1).is_none());
}
