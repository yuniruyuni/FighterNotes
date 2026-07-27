use super::support::*;

// ─── brightness_class ────────────────────────────────────────────────────────
//
// 各状態の fresh_v_min 閾値: Counter=172, PunishCounter=145,
// MotionRecovery=216, Active=157, ProjectileActive=160, Stun=219, Parry=95
// stripe 系は STRIPE_WF_MIN=0.10 で判定。

macro_rules! bright_tests {
    ($name_fresh:ident, $name_low:ident, $state:expr, $vmin:expr) => {
        #[test]
        fn $name_fresh() {
            assert_eq!(
                brightness_class(&$state, $vmin, 0.0),
                BrightClass::Fresh,
                "v={} should be Fresh for {:?}",
                $vmin,
                $state
            );
        }
        #[test]
        fn $name_low() {
            assert_eq!(
                brightness_class(&$state, $vmin - 0.1, 0.0),
                BrightClass::Low,
                "v={} should be Low for {:?}",
                $vmin - 0.1,
                $state
            );
        }
    };
}

bright_tests!(
    bright_counter_fresh,
    bright_counter_low,
    CellState::Counter,
    172.0
);
bright_tests!(
    bright_punish_counter_fresh,
    bright_punish_counter_low,
    CellState::PunishCounter,
    145.0
);
bright_tests!(
    bright_motion_recovery_fresh,
    bright_motion_recovery_low,
    CellState::MotionRecovery,
    216.0
);
bright_tests!(
    bright_active_fresh,
    bright_active_low,
    CellState::Active,
    157.0
);
bright_tests!(
    bright_projectile_active_fresh,
    bright_projectile_active_low,
    CellState::ProjectileActive,
    160.0
);
bright_tests!(bright_stun_fresh, bright_stun_low, CellState::Stun, 219.0);
bright_tests!(bright_parry_fresh, bright_parry_low, CellState::Parry, 95.0);

// Stripe 系: wf で判定（v は無視）

macro_rules! bright_stripe_tests {
    ($name_fresh:ident, $name_low:ident, $state:expr) => {
        #[test]
        fn $name_fresh() {
            assert_eq!(
                brightness_class(&$state, 0.0, STRIPE_WF_MIN),
                BrightClass::Fresh
            );
        }
        #[test]
        fn $name_low() {
            assert_eq!(
                brightness_class(&$state, 0.0, STRIPE_WF_MIN - 0.01),
                BrightClass::Low
            );
        }
    };
}

bright_stripe_tests!(
    bright_inv_full_fresh,
    bright_inv_full_low,
    CellState::InvFull
);
bright_stripe_tests!(
    bright_inv_strike_fresh,
    bright_inv_strike_low,
    CellState::InvStrike
);
bright_stripe_tests!(
    bright_inv_proj_fresh,
    bright_inv_proj_low,
    CellState::InvProj
);

// 分類不能な状態は常に None_

#[test]
fn bright_empty_is_none() {
    assert_eq!(
        brightness_class(&CellState::Empty, 255.0, 1.0),
        BrightClass::None_
    );
}

#[test]
fn bright_other_is_none() {
    assert_eq!(
        brightness_class(&CellState::Other, 255.0, 1.0),
        BrightClass::None_
    );
}

#[test]
fn bright_unknown_is_none() {
    assert_eq!(
        brightness_class(&CellState::Unknown, 255.0, 1.0),
        BrightClass::None_
    );
}
