use crate::test_support::*;

#[test]
fn test_neutral_projectile_during_recovery_is_not_a_failed_punish() {
    // 実ゲーム撮影動画のf2355 / f12165で観測した事象の同型回帰:
    // 中遠距離で相手が通常技を振り、自分が Sand Blast を返しただけ。
    // block 起点も着弾もないので、Recovery 表示との重なりだけでは
    // 「確定反撃が届かなかった」と断定しない。
    for target_frame in [2355, 12165] {
        let mut p1 = vec![MeterState::Free; 100];
        let mut p2 = vec![MeterState::Free; 100];
        p1[50..64].fill(MeterState::Recovery);
        p2[50..64].fill(MeterState::Startup);
        p2[64..68].fill(MeterState::ProjectileActive);

        let punishes = extract_synth_punishes(target_frame - 50, p1, p2, vec![]);
        assert!(
            punishes
                .iter()
                .all(|punish| !(punish.side == 2 && punish.frame == target_frame)),
            "neutral projectile was labeled as punish fail at f{target_frame}: {punishes:?}"
        );
    }
}
