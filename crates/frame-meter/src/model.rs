use crate::constants::CELL_COUNT;
use crate::digits::UNCOMPUTED_CORRELATION;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CellState {
    Counter,
    PunishCounter,
    MotionRecovery,
    Active,
    ProjectileActive,
    Parry,
    Stun,
    InvFull,
    InvStrike,
    InvProj,
    Empty,
    Other,
    Unknown,
}

impl CellState {
    pub fn is_stripe(&self) -> bool {
        matches!(
            self,
            CellState::InvFull | CellState::InvStrike | CellState::InvProj
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CellState::Counter => "counter",
            CellState::PunishCounter => "punish_counter",
            CellState::MotionRecovery => "motion_recovery",
            CellState::Active => "active",
            CellState::ProjectileActive => "projectile_active",
            CellState::Parry => "parry",
            CellState::Stun => "stun",
            CellState::InvFull => "inv_full",
            CellState::InvStrike => "inv_strike",
            CellState::InvProj => "inv_proj",
            CellState::Empty => "empty",
            CellState::Other => "other",
            CellState::Unknown => "unknown",
        }
    }

    pub fn is_info(&self) -> bool {
        !matches!(
            self,
            CellState::Empty | CellState::Other | CellState::Unknown
        )
    }

    // Kept as an inherent method for the existing crate-root API. Unknown values
    // intentionally fall back instead of returning FromStr::Err.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "counter" => CellState::Counter,
            "punish_counter" => CellState::PunishCounter,
            "motion_recovery" => CellState::MotionRecovery,
            "active" => CellState::Active,
            "projectile_active" => CellState::ProjectileActive,
            "parry" => CellState::Parry,
            "stun" => CellState::Stun,
            "inv_full" => CellState::InvFull,
            "inv_strike" => CellState::InvStrike,
            "inv_proj" => CellState::InvProj,
            "empty" => CellState::Empty,
            "other" => CellState::Other,
            _ => CellState::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BrightClass {
    Fresh,
    Low,
    None_,
}

impl BrightClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            BrightClass::Fresh => "fresh",
            BrightClass::Low => "low",
            BrightClass::None_ => "none",
        }
    }

    // Kept as an inherent method for the existing crate-root API. Unknown values
    // intentionally fall back instead of returning FromStr::Err.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "fresh" => BrightClass::Fresh,
            "low" => BrightClass::Low,
            _ => BrightClass::None_,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RowObs {
    pub v: Vec<f32>,
    pub wf: Vec<f32>,
    pub states: Vec<CellState>,
    pub bright: Vec<BrightClass>,
    pub fresh_edge: i32,
    pub bgr: Vec<[f32; 3]>,
    pub stripe: Vec<bool>,
    pub cols: Option<Vec<f32>>,
    pub cols_w: usize,
    pub rescued: Vec<bool>,
    pub quality: Vec<f32>,
    pub digit_corr: Option<Vec<[f32; 10]>>,
    pub slab_pos: i32,
    pub slab_state: Option<CellState>,
}

impl RowObs {
    pub fn empty() -> Self {
        Self {
            v: vec![0.0; CELL_COUNT],
            wf: vec![0.0; CELL_COUNT],
            states: vec![CellState::Empty; CELL_COUNT],
            bright: vec![BrightClass::None_; CELL_COUNT],
            fresh_edge: -1,
            bgr: vec![[0.0; 3]; CELL_COUNT],
            stripe: vec![false; CELL_COUNT],
            cols: None,
            cols_w: 0,
            rescued: vec![false; CELL_COUNT],
            quality: vec![0.0; CELL_COUNT],
            digit_corr: None,
            slab_pos: -1,
            slab_state: None,
        }
    }

    /// Returns digit scores when the cell was included in template matching.
    pub fn digit_correlation(&self, index: usize) -> Option<&[f32; 10]> {
        let correlation = self.digit_corr.as_ref()?.get(index)?;
        if correlation[0] == UNCOMPUTED_CORRELATION {
            return None;
        }
        Some(correlation)
    }
}
