use super::support::*;

// ─── classify_cell_pair ───────────────────────────────────────────────────────

// 状態色: Fresh（Dimでない両チャンネル）

#[test]
fn pair_counter_fresh() {
    let (s, b) = classify_cell_pair(counter(), counter());
    assert_eq!(s, CellState::Counter);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_counter_tint_is_counter_fresh() {
    // CounterTint は Counter ファミリー、not-dim → Fresh
    let (s, b) = classify_cell_pair(counter_tint(), counter_tint());
    assert_eq!(s, CellState::Counter);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_motion_recovery_fresh() {
    let (s, b) = classify_cell_pair(motion_recovery(), motion_recovery());
    assert_eq!(s, CellState::MotionRecovery);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_punish_counter_fresh() {
    let (s, b) = classify_cell_pair(punish_counter(), punish_counter());
    assert_eq!(s, CellState::PunishCounter);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_active_fresh() {
    let (s, b) = classify_cell_pair(active(), active());
    assert_eq!(s, CellState::Active);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_projectile_active_fresh() {
    let (s, b) = classify_cell_pair(projectile_active(), projectile_active());
    assert_eq!(s, CellState::ProjectileActive);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_stun_fresh() {
    let (s, b) = classify_cell_pair(stun(), stun());
    assert_eq!(s, CellState::Stun);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_stun_at_webcodecs_observed_distance_is_accepted() {
    // WebCodecs で観測した最大距離 (~92) を Stun パレットからのずれとして再現する。
    // 閾値を以前の 80 に戻すと、この回帰テストは Other になって失敗する。
    let shifted_stun = [147.0, 255.0, 247.0];
    let (s, b) = classify_cell_pair(shifted_stun, shifted_stun);
    assert_eq!(s, CellState::Stun);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_color_beyond_reject_distance_is_other() {
    // Active が最近傍だが距離は約 101。許容範囲を広げすぎないことも固定する。
    let beyond_limit = [0.0, 0.0, 210.0];
    let (s, b) = classify_cell_pair(beyond_limit, beyond_limit);
    assert_eq!(s, CellState::Other);
    assert_eq!(b, BrightClass::None_);
}

#[test]
fn pair_parry_fresh() {
    let (s, b) = classify_cell_pair(parry(), parry());
    assert_eq!(s, CellState::Parry);
    assert_eq!(b, BrightClass::Fresh);
}

// 状態色: Low（両チャンネルが Dim）

#[test]
fn pair_counter_dim_low() {
    let (s, b) = classify_cell_pair(counter_dim(), counter_dim());
    assert_eq!(s, CellState::Counter);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn pair_motion_recovery_dim_low() {
    let (s, b) = classify_cell_pair(motion_recovery_dim(), motion_recovery_dim());
    assert_eq!(s, CellState::MotionRecovery);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn pair_punish_counter_dim_low() {
    let (s, b) = classify_cell_pair(punish_counter_dim(), punish_counter_dim());
    assert_eq!(s, CellState::PunishCounter);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn pair_active_dim_low() {
    let (s, b) = classify_cell_pair(active_dim(), active_dim());
    assert_eq!(s, CellState::Active);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn pair_projectile_active_dim_low() {
    let (s, b) = classify_cell_pair(projectile_active_dim(), projectile_active_dim());
    assert_eq!(s, CellState::ProjectileActive);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn pair_stun_dim_low() {
    let (s, b) = classify_cell_pair(stun_dim(), stun_dim());
    assert_eq!(s, CellState::Stun);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn pair_parry_dim_low() {
    let (s, b) = classify_cell_pair(parry_dim(), parry_dim());
    assert_eq!(s, CellState::Parry);
    assert_eq!(b, BrightClass::Low);
}

// 無敵系: Fresh（非 Dim）

#[test]
fn pair_inv_full_fresh() {
    let (s, b) = classify_cell_pair(white(), gray());
    assert_eq!(s, CellState::InvFull);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_inv_strike_fresh() {
    let (s, b) = classify_cell_pair(white(), stripe_pink());
    assert_eq!(s, CellState::InvStrike);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_inv_proj_fresh() {
    let (s, b) = classify_cell_pair(white(), stripe_orange());
    assert_eq!(s, CellState::InvProj);
    assert_eq!(b, BrightClass::Fresh);
}

// 無敵系: Low（Dim）

#[test]
fn pair_inv_full_low() {
    let (s, b) = classify_cell_pair(white_dim(), gray_dim());
    assert_eq!(s, CellState::InvFull);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn pair_inv_strike_low() {
    let (s, b) = classify_cell_pair(white_dim(), stripe_pink_dim());
    assert_eq!(s, CellState::InvStrike);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn pair_inv_proj_low() {
    let (s, b) = classify_cell_pair(white_dim(), stripe_orange_dim());
    assert_eq!(s, CellState::InvProj);
    assert_eq!(b, BrightClass::Low);
}

// 空・その他

#[test]
fn pair_empty() {
    let (s, _) = classify_cell_pair(black(), black());
    assert_eq!(s, CellState::Empty);
}

#[test]
fn pair_white_white_is_other() {
    // 白+白 = 有効だが分類不能 → Other
    let (s, _) = classify_cell_pair(white(), white());
    assert_eq!(s, CellState::Other);
}

// チャンネル不一致 → B チャンネルを採用

#[test]
fn pair_mismatch_uses_b_channel() {
    // A=Counter, B=Active → (Active, Fresh)
    let (s, b) = classify_cell_pair(counter(), active());
    assert_eq!(s, CellState::Active);
    assert_eq!(b, BrightClass::Fresh);
}

#[test]
fn pair_mismatch_uses_b_dim() {
    // A=Counter, B=ActiveDim → (Active, Low)
    let (s, b) = classify_cell_pair(counter(), active_dim());
    assert_eq!(s, CellState::Active);
    assert_eq!(b, BrightClass::Low);
}

#[test]
fn observed_pair_calibration_samples_remain_classified() {
    let cases: &[([f32; 3], [f32; 3], CellState, BrightClass)] = &[
        (
            [109.5, 150.8, 14.2],
            [109.5, 150.8, 14.2],
            CellState::Counter,
            BrightClass::Low,
        ),
        (
            [135.0, 84.0, 11.2],
            [135.0, 84.0, 11.2],
            CellState::PunishCounter,
            BrightClass::Low,
        ),
        (
            [41.2, 191.2, 185.2],
            [41.2, 191.2, 185.2],
            CellState::Stun,
            BrightClass::Low,
        ),
        (
            [160.0, 170.0, 181.0],
            counter(),
            CellState::Counter,
            BrightClass::Fresh,
        ),
        (
            punish_counter(),
            black(),
            CellState::PunishCounter,
            BrightClass::Fresh,
        ),
        (black(), stun(), CellState::Stun, BrightClass::Fresh),
        (counter(), stun(), CellState::Stun, BrightClass::Fresh),
        (
            motion_recovery(),
            active(),
            CellState::Active,
            BrightClass::Fresh,
        ),
        (
            white_dim(),
            white_dim(),
            CellState::Other,
            BrightClass::None_,
        ),
        (
            [42.0, 40.0, 42.0],
            black(),
            CellState::Empty,
            BrightClass::None_,
        ),
    ];

    for (a, b, expected_state, expected_brightness) in cases {
        let (state, brightness) = classify_cell_pair(*a, *b);
        assert_eq!(&state, expected_state, "a={a:?}, b={b:?}");
        assert_eq!(&brightness, expected_brightness, "a={a:?}, b={b:?}");
    }
}
