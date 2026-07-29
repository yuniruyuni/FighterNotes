use super::support::*;

#[test]
fn damage_sequence_uses_gameplay_gap_across_sa_freeze() {
    // 同一コンボの 2 回の HP 減少の間に 163 video frame の SA 演出停止がある。
    // 実ゲーム進行差は 48F。通常の 45F 区切りをわずかに超えても、停止の
    // 根拠がある場合だけ 1 ダメージイベントへまとめる。
    let mut fs: Vec<_> = (0..600u32).map(|i| feat(i, 1.0, 1.0)).collect();
    for i in 190..195u32 {
        fs[i as usize] = feat(i, 1.0 - 0.02 * (i - 189) as f32, 1.0);
    }
    for i in 195..405u32 {
        fs[i as usize] = feat(i, 0.9, 1.0);
    }
    for i in 405..410u32 {
        fs[i as usize] = feat(i, 0.9 - 0.02 * (i - 404) as f32, 1.0);
    }
    for i in 410..600u32 {
        fs[i as usize] = feat(i, 0.8, 1.0);
    }

    let p1 = synth_timeline(vec![(10, "stun", 243, 405)]);
    let p2 = synth_timeline(vec![(10, "active", 243, 405)]);
    let with_freeze = build_match_events(&fs, &[], &[], Some((&p1, &p2)), "p1");
    let taken: Vec<_> = with_freeze
        .damage
        .iter()
        .filter(|d| d.victim == 1)
        .collect();
    assert_eq!(taken.len(), 1, "SA 停止をまたいでも同一連携: {taken:?}");
    assert!((taken[0].drop - 0.2).abs() < 0.03);

    let without_freeze = build_match_events(&fs, &[], &[], None, "p1");
    assert_eq!(
        without_freeze
            .damage
            .iter()
            .filter(|d| d.victim == 1)
            .count(),
        2,
        "停止根拠が無い同じ動画フレーム差は別被弾"
    );

    let p2_running = synth_timeline(synth_run(10, "active", 243, 405));
    let one_sided = build_match_events(&fs, &[], &[], Some((&p1, &p2_running)), "p1");
    assert_eq!(
        one_sided.damage.iter().filter(|d| d.victim == 1).count(),
        2,
        "片側だけの長時間表示は演出停止として結合しない"
    );
}

#[test]
fn damage_sequence_stays_joined_while_the_victim_never_becomes_actionable() {
    let mut fs: Vec<_> = (0..500u32).map(|i| feat(i, 1.0, 1.0)).collect();
    for i in 190..195u32 {
        fs[i as usize] = feat(i, 1.0 - 0.02 * (i - 189) as f32, 1.0);
    }
    for i in 195..310u32 {
        fs[i as usize] = feat(i, 0.9, 1.0);
    }
    for i in 310..315u32 {
        fs[i as usize] = feat(i, 0.9 - 0.02 * (i - 309) as f32, 1.0);
    }
    for i in 315..500u32 {
        fs[i as usize] = feat(i, 0.8, 1.0);
    }

    // 各 game frame が1 video frameずつ進むため、長い dwell による
    // freeze 判定には該当しない。それでも被弾側は一度も stun を抜けず、
    // SA/CA の一連のコンボである。
    let victim = synth_timeline(synth_run(0, "stun", 190, 315));
    let attacker = synth_timeline(synth_run(0, "active", 190, 315));
    let events = build_match_events(&fs, &[], &[], Some((&victim, &attacker)), "p1");
    let taken: Vec<_> = events.damage.iter().filter(|d| d.victim == 1).collect();
    assert_eq!(taken.len(), 1, "連続 stun の被弾を分割しない: {taken:?}");
    assert!((taken[0].drop - 0.2).abs() < 0.03);

    // 次の被弾までに行動可能な区間があれば、動画時間が同じでも別の被弾。
    let recovered = synth_timeline(synth_run(0, "stun", 190, 250));
    let events = build_match_events(&fs, &[], &[], Some((&recovered, &attacker)), "p1");
    assert_eq!(
        events.damage.iter().filter(|d| d.victim == 1).count(),
        2,
        "stun が切れた後の被弾は結合しない"
    );
}
