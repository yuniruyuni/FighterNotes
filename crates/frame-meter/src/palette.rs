use crate::color::{dim_anchor, l2_dist, Bgr};
use crate::model::CellState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PaletteName {
    Counter,
    CounterDim,
    CounterTint,
    MotionRecovery,
    MotionRecoveryDim,
    PunishCounter,
    PunishCounterDim,
    Active,
    ActiveDim,
    ProjectileActive,
    ProjectileActiveDim,
    Stun,
    StunDim,
    Parry,
    ParryDim,
    White,
    WhiteDim,
    Gray,
    GrayDim,
    StripePink,
    StripePinkDim,
    StripeOrange,
    StripeOrangeDim,
    Black,
    Gap,
    LabelBox,
}

impl PaletteName {
    pub(crate) fn color(self) -> Bgr {
        match self {
            PaletteName::Counter => [146.0, 201.0, 19.0],
            PaletteName::CounterDim => dim_anchor([146.0, 201.0, 19.0]),
            PaletteName::CounterTint => [130.0, 162.0, 49.0],
            PaletteName::MotionRecovery => [237.0, 255.0, 88.0],
            PaletteName::MotionRecoveryDim => dim_anchor([237.0, 255.0, 88.0]),
            PaletteName::PunishCounter => [180.0, 112.0, 15.0],
            PaletteName::PunishCounterDim => dim_anchor([180.0, 112.0, 15.0]),
            PaletteName::Active => [93.0, 20.0, 176.0],
            PaletteName::ActiveDim => dim_anchor([93.0, 20.0, 176.0]),
            PaletteName::ProjectileActive => [18.0, 127.0, 186.0],
            PaletteName::ProjectileActiveDim => dim_anchor([18.0, 127.0, 186.0]),
            PaletteName::Stun => [55.0, 255.0, 247.0],
            PaletteName::StunDim => dim_anchor([55.0, 255.0, 247.0]),
            PaletteName::Parry => [87.0, 17.0, 65.0],
            PaletteName::ParryDim => dim_anchor([87.0, 17.0, 65.0]),
            PaletteName::White => [236.0, 233.0, 233.0],
            PaletteName::WhiteDim => dim_anchor([236.0, 233.0, 233.0]),
            PaletteName::Gray => [200.0, 196.0, 197.0],
            PaletteName::GrayDim => dim_anchor([200.0, 196.0, 197.0]),
            PaletteName::StripePink => [140.0, 80.0, 200.0],
            PaletteName::StripePinkDim => dim_anchor([140.0, 80.0, 200.0]),
            PaletteName::StripeOrange => [40.0, 130.0, 230.0],
            PaletteName::StripeOrangeDim => dim_anchor([40.0, 130.0, 230.0]),
            PaletteName::Black => [23.0, 20.0, 23.0],
            PaletteName::Gap => [42.0, 40.0, 42.0],
            PaletteName::LabelBox => [160.0, 170.0, 181.0],
        }
    }

    pub(crate) fn all() -> &'static [PaletteName] {
        ALL_PALETTE
    }

    pub(crate) fn is_whiteish(self) -> bool {
        matches!(
            self,
            PaletteName::White | PaletteName::WhiteDim | PaletteName::LabelBox
        )
    }

    pub(crate) fn is_grayish(self) -> bool {
        matches!(self, PaletteName::Gray | PaletteName::GrayDim)
    }

    pub(crate) fn is_emptyish(self) -> bool {
        matches!(self, PaletteName::Black | PaletteName::Gap)
    }

    pub(crate) fn is_stripe_pink(self) -> bool {
        matches!(self, PaletteName::StripePink | PaletteName::StripePinkDim)
    }

    pub(crate) fn is_stripe_orange(self) -> bool {
        matches!(
            self,
            PaletteName::StripeOrange | PaletteName::StripeOrangeDim
        )
    }

    pub(crate) fn as_colored_entry(self) -> Option<(CellState, bool)> {
        match self {
            PaletteName::Counter | PaletteName::CounterTint => Some((CellState::Counter, false)),
            PaletteName::CounterDim => Some((CellState::Counter, true)),
            PaletteName::MotionRecovery => Some((CellState::MotionRecovery, false)),
            PaletteName::MotionRecoveryDim => Some((CellState::MotionRecovery, true)),
            PaletteName::PunishCounter => Some((CellState::PunishCounter, false)),
            PaletteName::PunishCounterDim => Some((CellState::PunishCounter, true)),
            PaletteName::Active => Some((CellState::Active, false)),
            PaletteName::ActiveDim => Some((CellState::Active, true)),
            PaletteName::ProjectileActive => Some((CellState::ProjectileActive, false)),
            PaletteName::ProjectileActiveDim => Some((CellState::ProjectileActive, true)),
            PaletteName::Stun => Some((CellState::Stun, false)),
            PaletteName::StunDim => Some((CellState::Stun, true)),
            PaletteName::Parry => Some((CellState::Parry, false)),
            PaletteName::ParryDim => Some((CellState::Parry, true)),
            _ => None,
        }
    }

    pub(crate) fn state_family(self) -> Option<CellState> {
        self.as_colored_entry().map(|(state, _)| state)
    }
}

static ALL_PALETTE: &[PaletteName] = &[
    PaletteName::Counter,
    PaletteName::CounterDim,
    PaletteName::MotionRecovery,
    PaletteName::MotionRecoveryDim,
    PaletteName::PunishCounter,
    PaletteName::PunishCounterDim,
    PaletteName::Active,
    PaletteName::ActiveDim,
    PaletteName::ProjectileActive,
    PaletteName::ProjectileActiveDim,
    PaletteName::Stun,
    PaletteName::StunDim,
    PaletteName::Parry,
    PaletteName::ParryDim,
    PaletteName::White,
    PaletteName::WhiteDim,
    PaletteName::Gray,
    PaletteName::GrayDim,
    PaletteName::StripePink,
    PaletteName::StripePinkDim,
    PaletteName::StripeOrange,
    PaletteName::StripeOrangeDim,
    PaletteName::Black,
    PaletteName::Gap,
    PaletteName::LabelBox,
    PaletteName::CounterTint,
];

pub(crate) fn nearest_palette(bgr: Bgr) -> (PaletteName, f32) {
    let mut best_name = PaletteName::Black;
    let mut best_dist = f32::MAX;
    for &name in PaletteName::all() {
        let distance = l2_dist(bgr, name.color());
        if distance < best_dist {
            best_dist = distance;
            best_name = name;
        }
    }
    (best_name, best_dist)
}

pub(crate) fn state_quality(state: &CellState, a_bgr: Bgr, b_bgr: Bgr) -> f32 {
    let mut min_dist = f32::MAX;
    for &name in PaletteName::all() {
        if name.state_family().as_ref() == Some(state) {
            let color = name.color();
            min_dist = min_dist.min(l2_dist(a_bgr, color).min(l2_dist(b_bgr, color)));
        }
    }
    if min_dist == f32::MAX {
        0.0
    } else {
        min_dist
    }
}
