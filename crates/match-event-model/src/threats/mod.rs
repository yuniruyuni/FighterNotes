//! Character actions that remain meaningful across multiple meter segments.
//!
//! The frame meter exposes the player's current action. A projectile, however,
//! can remain on screen after its owner starts jumping or teleporting. This
//! module keeps projectile and reposition events as independent entities, then
//! joins them into compound threats without flattening them into one state.

use super::*;

pub struct ThreatInputs<'a> {
    pub features: &'a [FrameFeatures],
    pub timelines: [&'a MeterTimeline; 2],
    pub meter_state: &'a [Vec<MeterState>; 2],
    pub segments: &'a [Vec<InputSegment>; 2],
    pub jumps: &'a [JumpEvent],
    pub contacts: &'a [ContactEvent],
    pub damage: &'a [DamageEvent],
    pub rounds: &'a [RoundInfo],
    pub characters: [Option<&'a str>; 2],
}

/// How long a projectile may remain relevant after its last visible meter cell.
///
/// The meter switches to a new segment when the owner starts another action.
/// The verified Yoga Fire sample is last observed at f6228 and contacts at f6247, so the
/// observed end is not the projectile's disappearance time.
pub const PROJECTILE_CARRY_WINDOW: u32 = 90;
/// Search window from teleport invincibility to its attacking follow-up. The
/// verified Dhalsim cases take at most 21 video frames; extra headroom covers
/// capture jitter without attaching an unrelated later normal.
pub const TELEPORT_FOLLOWUP_WINDOW: u32 = 36;
/// A teleport command should appear shortly before the invincibility run.
pub const TELEPORT_INPUT_LOOKBACK: u32 = 24;
/// Dhalsim's teleport invincibility is short. Long runs are supers/cinematics.
pub const TELEPORT_INV_MAX: u32 = 12;
/// A persistent projectile must advance through several game frames. A single
/// projectile-active cell held by hitstop is usually a normal hit/effect.
pub const PROJECTILE_MIN_GAME_FRAMES: usize = 8;
/// Meter-only projectile contact search after its observed active run.
pub const PROJECTILE_CONTACT_WINDOW: u32 = 36;
/// Damage bars update after contact/hitstop, so allow a delayed HP transition.
pub const THREAT_DAMAGE_WINDOW: u32 = 25;

mod extraction;
mod model;
mod response;
mod state_runs;
mod teleport;

pub use extraction::extract_threats;
pub use model::*;
use response::{damage_assigned_to_contact, response_in_window};
use state_runs::{state_runs, StateRun};
use teleport::{is_dhalsim, teleport_input};

#[cfg(test)]
mod tests;
