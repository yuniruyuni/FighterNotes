use crate::color::{bgr_to_hsv, Bgr};
use crate::constants::{EMPTY_V_MAX, PAIR_REJECT_DIST, STRIPE_WF_MIN};
use crate::model::{BrightClass, CellState};
use crate::palette::{nearest_palette, PaletteName};

pub(crate) fn fresh_v_min_for(state: &CellState) -> Option<f32> {
    match state {
        CellState::Counter => Some(172.0),
        CellState::PunishCounter => Some(145.0),
        CellState::MotionRecovery => Some(216.0),
        CellState::Active => Some(157.0),
        CellState::ProjectileActive => Some(160.0),
        CellState::Stun => Some(219.0),
        CellState::Parry => Some(95.0),
        _ => None,
    }
}

/// Classifies a cell from the two mode colors sampled from its stripe regions.
pub fn classify_cell_pair(a_bgr: Bgr, b_bgr: Bgr) -> (CellState, BrightClass) {
    let (a, da) = nearest_palette(a_bgr);
    let (b, db) = nearest_palette(b_bgr);
    let a_rejected = da > PAIR_REJECT_DIST;
    let b_rejected = db > PAIR_REJECT_DIST;

    if !a_rejected && a.is_whiteish() && !b_rejected && b.is_grayish() {
        let bright = if matches!(a, PaletteName::White) && matches!(b, PaletteName::Gray) {
            BrightClass::Fresh
        } else {
            BrightClass::Low
        };
        return (CellState::InvFull, bright);
    }
    if !a_rejected && a.is_whiteish() && !b_rejected && b.is_stripe_pink() {
        let bright = if matches!(a, PaletteName::White) && matches!(b, PaletteName::StripePink) {
            BrightClass::Fresh
        } else {
            BrightClass::Low
        };
        return (CellState::InvStrike, bright);
    }
    if !a_rejected && a.is_whiteish() && !b_rejected && b.is_stripe_orange() {
        let bright = if matches!(a, PaletteName::White) && matches!(b, PaletteName::StripeOrange) {
            BrightClass::Fresh
        } else {
            BrightClass::Low
        };
        return (CellState::InvProj, bright);
    }
    let ca = (!a_rejected).then(|| a.as_colored_entry()).flatten();
    let cb = (!b_rejected).then(|| b.as_colored_entry()).flatten();

    match (ca, cb) {
        (Some((sa, da_dim)), Some((sb, db_dim))) => {
            if sa == sb {
                let bright = if !da_dim && !db_dim {
                    BrightClass::Fresh
                } else {
                    BrightClass::Low
                };
                (sa, bright)
            } else {
                let bright = if !db_dim {
                    BrightClass::Fresh
                } else {
                    BrightClass::Low
                };
                (sb, bright)
            }
        }
        (None, Some((state, is_dim))) | (Some((state, is_dim)), None) => {
            let bright = if is_dim {
                BrightClass::Low
            } else {
                BrightClass::Fresh
            };
            (state, bright)
        }
        (None, None) => {
            if !a_rejected && a.is_emptyish() && !b_rejected && b.is_emptyish() {
                (CellState::Empty, BrightClass::None_)
            } else {
                (CellState::Other, BrightClass::None_)
            }
        }
    }
}

/// Classifies the raw vocabulary for a single cell sample.
pub fn classify_cell_raw(bgr: Bgr, white_frac: f32, stripe: Option<bool>) -> CellState {
    let hsv = bgr_to_hsv(bgr);
    let h = hsv[0] as i32;
    let s = hsv[1] as i32;
    let v = hsv[2];

    let is_stripe = stripe.unwrap_or(white_frac > STRIPE_WF_MIN);
    if is_stripe {
        if s < 60 {
            return CellState::InvFull;
        }
        if h >= 145 || h <= 8 {
            return CellState::InvStrike;
        }
        return CellState::InvProj;
    }

    if v < EMPTY_V_MAX {
        return CellState::Empty;
    }
    if (138..=152).contains(&h) && s >= 150 {
        return CellState::Parry;
    }
    if (h >= 145 || h <= 8) && s >= 40 {
        return CellState::Active;
    }
    if (22..=38).contains(&h) && s >= 55 {
        return CellState::Stun;
    }
    if (9..=21).contains(&h) && s >= 100 {
        return CellState::ProjectileActive;
    }
    if (75..=100).contains(&h) && s >= 40 {
        if h <= 85 || s >= 200 {
            CellState::Counter
        } else {
            CellState::MotionRecovery
        }
    } else if (101..=137).contains(&h) && s >= 40 {
        CellState::PunishCounter
    } else {
        CellState::Other
    }
}

/// Classifies a cell as fresh, dimmed, or non-colored.
pub fn brightness_class(state: &CellState, v: f32, wf: f32) -> BrightClass {
    if state.is_stripe() {
        return if wf >= STRIPE_WF_MIN {
            BrightClass::Fresh
        } else {
            BrightClass::Low
        };
    }
    match fresh_v_min_for(state) {
        None => BrightClass::None_,
        Some(vmin) if v >= vmin => BrightClass::Fresh,
        Some(_) => BrightClass::Low,
    }
}
