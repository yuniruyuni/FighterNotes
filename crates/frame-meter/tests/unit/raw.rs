use super::support::*;

// ─── classify_cell_raw ────────────────────────────────────────────────────────
//
// この関数は HSV 色相/彩度/明度の範囲で直接分類する。
// 各パレット色を入力として、期待される状態を検証する。
// テスト内の HSV 計算は lib.rs の bgr_to_hsv() と同じ式で手計算済み。

#[test]
fn raw_empty_black() {
    // V ≈ 23 < EMPTY_V_MAX(55) → Empty
    let s = classify_cell_raw(black(), 0.0, None);
    assert_eq!(s, CellState::Empty);
}

#[test]
fn raw_empty_at_threshold() {
    // V = EMPTY_V_MAX - epsilon → Empty
    let dark = [30.0, 30.0, 30.0];
    let s = classify_cell_raw(dark, 0.0, None);
    assert_eq!(s, CellState::Empty);
}

#[test]
fn raw_stripe_white_is_inv_full() {
    // stripe=Some(true), 白色(S≈3<60) → InvFull
    let s = classify_cell_raw(white(), 0.0, Some(true));
    assert_eq!(s, CellState::InvFull);
}

#[test]
fn raw_stripe_via_white_frac_is_inv_full() {
    // white_frac > STRIPE_WF_MIN で自動 stripe 判定
    let s = classify_cell_raw(white(), STRIPE_WF_MIN + 0.01, None);
    assert_eq!(s, CellState::InvFull);
}

#[test]
fn raw_stripe_pink_is_inv_strike() {
    // StripePink: BGR=[140,80,200] → h_opencv≈165 >= 145, s≈153 >= 60 → InvStrike
    let s = classify_cell_raw(stripe_pink(), 0.0, Some(true));
    assert_eq!(s, CellState::InvStrike);
}

#[test]
fn raw_stripe_orange_is_inv_proj() {
    // StripeOrange: BGR=[40,130,230] → h_opencv≈14, not in [145+] → InvProj
    let s = classify_cell_raw(stripe_orange(), 0.0, Some(true));
    assert_eq!(s, CellState::InvProj);
}

#[test]
fn raw_stripe_dark_not_empty() {
    // stripe フラグは v < EMPTY_V_MAX より先に評価されるため Empty にはならない
    let s = classify_cell_raw(black(), 0.0, Some(true));
    // S≈3 < 60 → InvFull（Empty ではない）
    assert_eq!(s, CellState::InvFull);
}

#[test]
fn raw_parry() {
    // Parry: BGR=[87,17,65] → h_opencv≈140, s≈205, v≈87
    // h in [138,152] && s>=150 → Parry
    let s = classify_cell_raw(parry(), 0.0, None);
    assert_eq!(s, CellState::Parry);
}

#[test]
fn raw_active() {
    // Active: BGR=[93,20,176] → h_opencv≈166, s≈226 → (h>=145||h<=8) && s>=40 → Active
    let s = classify_cell_raw(active(), 0.0, None);
    assert_eq!(s, CellState::Active);
}

#[test]
fn raw_stun() {
    // Stun: BGR=[55,255,247] → h_opencv≈31, s≈200 → h in [22,38] → Stun
    let s = classify_cell_raw(stun(), 0.0, None);
    assert_eq!(s, CellState::Stun);
}

#[test]
fn raw_projectile_active() {
    // ProjectileActive: BGR=[18,127,186] → h_opencv≈19, s≈230 → h in [9,21] → ProjectileActive
    let s = classify_cell_raw(projectile_active(), 0.0, None);
    assert_eq!(s, CellState::ProjectileActive);
}

#[test]
fn raw_counter() {
    // Counter: BGR=[146,201,19] → h_opencv≈81, s≈231 → h in [75,100], h<=85 → Counter
    let s = classify_cell_raw(counter(), 0.0, None);
    assert_eq!(s, CellState::Counter);
}

#[test]
fn raw_motion_recovery() {
    // MotionRecovery: BGR=[237,255,88] → h_opencv≈87, s≈167 → h in [75,100], h>85 && s<200 → MotionRecovery
    let s = classify_cell_raw(motion_recovery(), 0.0, None);
    assert_eq!(s, CellState::MotionRecovery);
}

#[test]
fn raw_punish_counter() {
    // PunishCounter: BGR=[180,112,15] → h_opencv≈102, s≈234 → h in [101,137] → PunishCounter
    let s = classify_cell_raw(punish_counter(), 0.0, None);
    assert_eq!(s, CellState::PunishCounter);
}

#[test]
fn raw_unmapped_hue_is_other() {
    // BGR=[0,200,50]: h_opencv≈72, s≈191 → 72 は Counter/Recovery 範囲 [75,100] に入らない → Other
    let s = classify_cell_raw([0.0, 200.0, 50.0], 0.0, None);
    assert_eq!(s, CellState::Other);
}

#[test]
fn observed_raw_calibration_samples_remain_classified() {
    let cases: &[([f32; 3], f32, Option<bool>, CellState)] = &[
        ([143.0, 196.0, 18.0], 0.0, None, CellState::Counter),
        ([111.0, 140.0, 15.0], 0.0, None, CellState::Counter),
        ([236.0, 247.0, 82.0], 0.0, None, CellState::MotionRecovery),
        ([130.0, 65.0, 8.0], 0.0, None, CellState::PunishCounter),
        ([80.0, 22.0, 138.0], 0.0, None, CellState::Active),
        ([17.0, 80.0, 136.0], 0.0, None, CellState::ProjectileActive),
        ([84.0, 13.0, 63.0], 0.0, None, CellState::Parry),
        ([41.0, 162.0, 171.0], 0.0, None, CellState::Stun),
        ([126.0, 126.0, 140.0], 0.3, Some(true), CellState::InvFull),
        ([121.0, 92.0, 140.0], 0.3, Some(true), CellState::InvStrike),
        ([96.0, 119.0, 140.0], 0.3, Some(true), CellState::InvProj),
        ([126.0, 126.0, 140.0], 0.3, None, CellState::InvFull),
        ([15.0, 15.0, 15.0], 0.0, None, CellState::Empty),
        ([22.0, 22.0, 22.0], 0.0, None, CellState::Empty),
        ([49.0, 49.0, 49.0], 0.0, None, CellState::Empty),
        ([43.0, 118.0, 116.0], 0.0, None, CellState::Stun),
        ([98.0, 49.0, 6.0], 0.0, None, CellState::PunishCounter),
    ];

    for (bgr, white_fraction, stripe, expected) in cases {
        let actual = classify_cell_raw(*bgr, *white_fraction, *stripe);
        assert_eq!(&actual, expected, "bgr={bgr:?}");
    }
}
