#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DefenseResponseKind {
    Parry,
    Invincible,
    Guard,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DefenseResponse {
    pub side: u8,
    pub kind: DefenseResponseKind,
    pub start_frame: u32,
    pub end_frame: u32,
}

/// A projectile created by a character action.
///
/// `observed_end_frame` is only the end of the frame-meter evidence. The
/// projectile remains a possible threat through `threat_end_frame`, or until a
/// later spatial pass supplies a more precise contact/disappearance frame.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProjectileThreat {
    pub owner: u8,
    pub observed_start_frame: u32,
    pub observed_end_frame: u32,
    pub threat_end_frame: u32,
    pub contact_frame: Option<u32>,
    pub round_no: u32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TeleportContext {
    /// Teleport followed by an attack while no earlier projectile may remain.
    NakedAttack,
    /// A projectile may still reach the defender during the teleport follow-up.
    ProjectileCovered,
    /// Defender was locked in hit/block stun, knockdown, or another state in
    /// which an immediate anti-air was unavailable. The frame meter cannot
    /// reliably distinguish those causes, so this does not claim a combo.
    DefenderUnavailable,
    /// No attacking follow-up was observed.
    MovementOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DpReachability {
    /// No spatial observation is available. Advice must abstain.
    Unknown,
    /// Spatial/action calibration says the anti-air reaches this appearance.
    Confirmed,
    /// Spatial/action calibration says the anti-air cannot reach.
    OutOfRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThreatOutcome {
    Hit,
    Defended,
    Whiffed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TeleportEvent {
    pub attacker: u8,
    pub defender: u8,
    pub input_frame: u32,
    pub inv_start_frame: u32,
    pub inv_end_frame: u32,
    pub followup_attack_frame: Option<u32>,
    pub followup_contact_frame: Option<u32>,
    pub airborne: bool,
    pub defender_actionable: bool,
    pub context: TeleportContext,
    pub response: Option<DefenseResponse>,
    pub outcome: ThreatOutcome,
    pub damage: f32,
    pub dp_reachability: DpReachability,
    pub round_no: u32,
    pub confidence: f32,
}

/// Two independently moving threats whose impact windows overlap.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompoundThreat {
    pub attacker: u8,
    pub defender: u8,
    pub projectile_start_frame: u32,
    pub teleport_frame: u32,
    pub followup_attack_frame: u32,
    pub followup_contact_frame: Option<u32>,
    pub projectile_response: Option<DefenseResponse>,
    pub followup_response: Option<DefenseResponse>,
    pub outcome: ThreatOutcome,
    pub damage: f32,
    pub round_no: u32,
    pub confidence: f32,
}
