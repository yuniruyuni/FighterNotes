//! 上位 crate から検出器の配線を検査するための、最小の発火 fixture。
//!
//! 個々の検出器テストで固定している成立条件と同じイベントを使い、カード
//! ごとに一つの入力へまとめる。上位層はこの表を回すことで、検出器の呼び
//! 出しが削除されたり、必要な引数が落ちたりしていないことを確認できる。

use crate::attack_info::AttackAttribute;
use crate::frame_features::FrameFeatures;
use crate::match_events::*;
use crate::RoundSummary;
use match_event_layer::test_support::feat;

pub use match_event_layer::test_support::empty_events;

/// 一つのカードを成立させる入力一式。
pub struct CardFixture {
    pub id: &'static str,
    pub features: Vec<FrameFeatures>,
    pub events: MatchEvents,
    pub own: u8,
    pub own_index: usize,
    pub own_character: Option<&'static str>,
    pub round_summaries: Vec<RoundSummary>,
    /// 引数が結果へ届いたことまで確認するための、カード固有の文言。
    pub description_contains: Option<&'static str>,
}

impl CardFixture {
    fn new(id: &'static str, events: MatchEvents) -> Self {
        Self {
            id,
            features: Vec::new(),
            events,
            own: 1,
            own_index: 0,
            own_character: None,
            round_summaries: Vec::new(),
            description_contains: None,
        }
    }
}

fn damage(victim: u8, start_frame: u32, end_frame: u32, drop: f32) -> DamageEvent {
    DamageEvent {
        victim,
        start_frame,
        pre_freeze_frame: start_frame,
        end_frame,
        hp_before: 1.0,
        hp_after: 1.0 - drop,
        drop,
        round_no: 1,
    }
}

fn layered_defense() -> CardFixture {
    let projectile_start = 100;
    let followup = projectile_start + 60;
    let mut events = empty_events();
    events.compound_threats.push(CompoundThreat {
        attacker: 2,
        defender: 1,
        projectile_start_frame: projectile_start,
        teleport_frame: projectile_start + 30,
        followup_attack_frame: followup,
        followup_contact_frame: Some(followup + 10),
        projectile_response: Some(DefenseResponse {
            side: 1,
            kind: DefenseResponseKind::Parry,
            start_frame: projectile_start + 5,
            end_frame: followup - 5,
        }),
        followup_response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.18,
        round_no: 1,
        confidence: 1.0,
    });
    CardFixture::new("layered_defense", events)
}

fn teleport_defense() -> CardFixture {
    let input_frame = 300;
    let followup = input_frame + 40;
    let mut events = empty_events();
    events.teleports.push(TeleportEvent {
        attacker: 2,
        defender: 1,
        input_frame,
        inv_start_frame: input_frame + 2,
        inv_end_frame: input_frame + 20,
        followup_attack_frame: Some(followup),
        followup_contact_frame: Some(followup + 6),
        airborne: true,
        defender_actionable: true,
        context: TeleportContext::NakedAttack,
        response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.20,
        dp_reachability: DpReachability::Confirmed,
        round_no: 1,
        confidence: 1.0,
    });
    CardFixture::new("teleport_defense", events)
}

fn jump(side: u8, frame: u32, outcome: JumpOutcome) -> JumpEvent {
    JumpEvent {
        side,
        frame,
        outcome,
        input_dir: "UR".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(frame + 20),
        takeoff_confirmed: true,
        air_end: frame + 45,
        round_no: 1,
    }
}

fn anti_air() -> CardFixture {
    let mut events = empty_events();
    events.jumps.push(jump(2, 500, JumpOutcome::LandedHit));
    events.damage.push(damage(1, 520, 550, 0.15));
    CardFixture::new("anti_air", events)
}

fn own_jumps() -> CardFixture {
    let mut events = empty_events();
    events.jumps.push(jump(1, 700, JumpOutcome::GotHit));
    events.damage.push(damage(1, 720, 750, 0.15));
    CardFixture::new("own_jumps", events)
}

fn burnout() -> CardFixture {
    let mut events = empty_events();
    events.burnouts.push(BurnoutPeriod {
        side: 1,
        start_frame: 800,
        end_frame: 1_700,
        hp_lost: 0.20,
        hp_dealt: 0.08,
        cause: BurnoutCause::ForcedByGuard,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    CardFixture::new("burnout", events)
}

/// P2 の配列だけへ観測を置く。`own_index` が 0 に落ちると発火しないため、
/// 上位層が side と index を別々に正しく渡していることも固定できる。
fn committed_button_vs_di() -> CardFixture {
    let mut events = empty_events();
    events.damage.push(damage(2, 1_000, 1_120, 0.24));
    events.segments[1].push(InputSegment {
        start_frame: 990,
        end_frame: 994,
        dir: "N".to_string(),
        badges: vec!["強K".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
    events.drive_impacts.push(DriveImpactEvent {
        side: 1,
        input_frame: 970,
        active_frame: Some(997),
        contact_frame: Some(1_000),
        outcome: DriveImpactOutcome::Hit,
        damage: 0.24,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events.meter_state = [vec![MeterState::Free; 1_300], vec![MeterState::Free; 1_300]];
    events.meter_state[1][990..997].fill(MeterState::Startup);
    events.meter_state[1][997..1_000].fill(MeterState::Active);
    events.meter_state[1][1_000] = MeterState::Recovery;
    events.meter_confidence = [vec![1.0; 1_300], vec![1.0; 1_300]];

    let mut fixture = CardFixture::new("committed_button_vs_di", events);
    fixture.own = 2;
    fixture.own_index = 1;
    fixture
}

fn mashing() -> CardFixture {
    let mut events = empty_events();
    events.damage.push(damage(1, 1_000, 1_020, 0.12));
    events.segments[0].push(InputSegment {
        start_frame: 990,
        end_frame: 995,
        dir: "N".to_string(),
        badges: vec!["弱".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
    let features = (0..1_200u32)
        .map(|frame| {
            let mut feature = feat(frame, 1.0, 1.0);
            feature.left_drive_ratio = if frame < 880 {
                1.0
            } else {
                (1.0 - 0.20 * (frame - 880) as f32 / 120.0).max(0.0)
            };
            feature
        })
        .collect();
    let mut fixture = CardFixture::new("mashing", events);
    fixture.features = features;
    fixture
}

fn minus_press(kind: DefensiveActionKind, id: &'static str) -> CardFixture {
    let mut events = empty_events();
    events.presses_while_minus.push(MinusPressEvent {
        side: 1,
        frame: 1_300,
        minus_frames: 5,
        pressed: if kind == DefensiveActionKind::Throw {
            "投げ"
        } else {
            "弱"
        }
        .to_string(),
        action_kind: kind,
        outcome: MinusPressOutcome::CounterHit,
        drop: 0.12,
        confidence: EventConfidence::High,
        source_contact_frame: 1_280,
        round_no: 1,
    });
    CardFixture::new(id, events)
}

fn advantage_abandoned() -> CardFixture {
    let mut events = empty_events();
    events.advantage_situations.push(AdvantageSituationEvent {
        side: 1,
        frame: 1_500,
        plus_frames: 5,
        follow_up: None,
        action_frame: None,
        pressed: String::new(),
        outcome: AdvantageOutcome::TurnLost,
        drop: 0.10,
        confidence: EventConfidence::High,
        source_contact_frame: 1_480,
        round_no: 1,
    });
    CardFixture::new("advantage_abandoned", events)
}

fn guard_break() -> CardFixture {
    let frame = 1_700;
    let mut events = empty_events();
    events.guard_breaks.push(GuardBreakEvent {
        side: 1,
        frame,
        drop: 0.15,
        guard_dir: "DR".to_string(),
        broke_to: "UR".to_string(),
        round_no: 1,
    });
    events.damage.push(damage(1, frame, frame + 30, 0.15));
    CardFixture::new("guard_break", events)
}

fn reversal_punished() -> CardFixture {
    let mut events = empty_events();
    events.reversals.push(ReversalEvent {
        side: 1,
        frame: 1_900,
        drop: 0.25,
        blocked: true,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    CardFixture::new("reversal_punished", events)
}

fn low_scaling_super() -> CardFixture {
    let frame = 2_100;
    let mut events = empty_events();
    events.super_arts.push(SuperArtEvent {
        side: 1,
        frame,
        gauge_drop_frame: frame + 5,
        level: 3,
        critical_art: false,
        gauge_before: 3.0,
        gauge_after: 0.0,
        context: SuperArtContext::Combo,
        outcome: SuperArtOutcome::Hit,
        contact_frame: Some(frame + 20),
        damage: 0.15,
        ko: false,
        punished: false,
        punished_damage: 0.0,
        confidence: EventConfidence::High,
        round_no: 1,
    });
    events
        .attack_evidence
        .super_arts
        .push(SuperArtAttackEvidence {
            side: 1,
            super_frame: frame,
            combo_damage: 3_600,
            marginal_damage: Some(400),
            entry_scaling_percent: Some(30),
            final_scaling_percent: 30,
            confidence: EventConfidence::High,
        });
    let linked_damage = damage(2, frame + 20, frame + 90, 0.30);
    events.attack_evidence.damage.push(DamageAttackEvidence {
        victim: 2,
        attacker: 1,
        damage_start_frame: linked_damage.start_frame,
        sequence_start_frame: linked_damage.start_frame,
        sequence_end_frame: linked_damage.end_frame,
        combo_damage: 3_600,
        sequence_count: 1,
        final_scaling_percent: 30,
        starter_attribute: None,
        final_attribute: AttackAttribute::Middle,
        complete: true,
        recovered_from_max: false,
        confidence: EventConfidence::High,
        hp_consistency: AttackDamageConsistency::Consistent,
        sequence_indices: Vec::new(),
    });
    events.damage.push(linked_damage);
    CardFixture::new("low_scaling_super", events)
}

fn punish_chance(frame: u32, outcome: PunishOutcome) -> PunishChance {
    PunishChance {
        frame,
        side: 1,
        advantage: 12,
        outcome,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: frame.saturating_sub(20),
        recovery_end_frame: frame,
        source_contact_frame: Some(frame.saturating_sub(30)),
        attack_start_frame: Some(frame + 2),
        attack_active_frame: Some(frame + 8),
        reachability: PunishReachability::Confirmed,
        punished_drop: 0.0,
        pressed: "強P".to_string(),
        round_no: 1,
    }
}

fn punish_fail() -> CardFixture {
    let mut events = empty_events();
    events
        .punishes
        .push(punish_chance(2_300, PunishOutcome::WhiffFail));
    CardFixture::new("punish_fail", events)
}

fn punish_missed() -> CardFixture {
    let mut events = empty_events();
    events
        .punishes
        .push(punish_chance(2_500, PunishOutcome::Missed));
    let mut fixture = CardFixture::new("punish_missed", events);
    fixture.own_character = Some("LUKE");
    fixture.description_contains = Some("威力");
    fixture
}

fn low_conversion() -> CardFixture {
    let frame = 2_700;
    let mut events = empty_events();
    events
        .punishes
        .push(punish_chance(frame, PunishOutcome::Success));
    events.contacts.push(ContactEvent {
        frame: frame + 5,
        attacker: 1,
        victim: 2,
        hit: true,
        projectile: false,
        round_no: 1,
    });
    events.damage.push(damage(2, frame + 10, frame + 50, 0.06));
    CardFixture::new("low_conversion", events)
}

fn throw_action(outcome: ThrowOutcome, input_frame: u32) -> ThrowActionEvent {
    ThrowActionEvent {
        thrower: 1,
        input_frame,
        startup_frame: Some(input_frame + 3),
        active_frame: Some(input_frame + 5),
        outcome,
        damage: 0.0,
        approach: ThrowApproach::Unknown,
        confidence: EventConfidence::High,
        round_no: 1,
    }
}

fn throw_interrupted_by_invincible() -> CardFixture {
    let mut events = empty_events();
    events
        .throw_actions
        .push(throw_action(ThrowOutcome::InterruptedByInvincible, 2_900));
    events.damage.push(damage(1, 2_909, 2_970, 0.20));
    CardFixture::new("throw_interrupted_by_invincible", events)
}

fn throw_whiff_punished() -> CardFixture {
    let mut events = empty_events();
    events
        .throw_actions
        .push(throw_action(ThrowOutcome::ExecutedWhiff, 3_100));
    events.damage.push(damage(1, 3_130, 3_170, 0.20));
    CardFixture::new("throw_whiff_punished", events)
}

fn whiff_punished() -> CardFixture {
    let frame = 3_300;
    let mut events = empty_events();
    events.whiffs.push(WhiffEvent {
        side: 1,
        frame,
        end_frame: frame + 8,
        outcome: WhiffOutcome::Punished,
        drop: 0.20,
        punished_frame: Some(frame + 15),
        confidence: EventConfidence::High,
        round_no: 1,
    });
    CardFixture::new("whiff_punished", events)
}

fn throw_loop() -> CardFixture {
    let mut events = empty_events();
    events.throws.push(ThrowEvent {
        thrower: 2,
        frame: 3_500,
        connected: true,
        round_no: 1,
    });
    CardFixture::new("throw_loop", events)
}

fn early_hits() -> CardFixture {
    let mut events = empty_events();
    events.damage.push(damage(1, 100, 140, 0.12));
    let mut fixture = CardFixture::new("early_hits", events);
    fixture.round_summaries.push(RoundSummary {
        round_no: 1,
        start_frame: 0,
        end_frame: 3_000,
        won: Some(false),
        own_hp_end: 0.0,
        opp_hp_end: 0.5,
        own_hp_lost: 1.0,
        opp_hp_lost: 0.5,
        own_hits_taken: 4,
        early_hit: true,
        own_burnouts: 0,
        detection_confidence: "high".to_string(),
    });
    fixture
}

fn lead_loss() -> CardFixture {
    let mut events = empty_events();
    let length = 600usize;
    let ramp = |from: f32, to: f32| -> Vec<f32> {
        (0..length)
            .map(|frame| from + (to - from) * frame as f32 / (length - 1) as f32)
            .collect()
    };
    events.hp = [ramp(1.0, 0.0), ramp(0.6, 0.4)];
    events.rounds[0].end_frame = length as u32 - 1;
    let mut fixture = CardFixture::new("lead_loss", events);
    fixture.round_summaries.push(RoundSummary {
        round_no: 1,
        start_frame: 0,
        end_frame: length as u32 - 1,
        won: Some(false),
        own_hp_end: 0.0,
        opp_hp_end: 0.4,
        own_hp_lost: 1.0,
        opp_hp_lost: 0.6,
        own_hits_taken: 5,
        early_hit: false,
        own_burnouts: 0,
        detection_confidence: "high".to_string(),
    });
    fixture
}

fn big_hits() -> CardFixture {
    let mut events = empty_events();
    events.damage.push(damage(1, 3_700, 3_750, 0.30));
    CardFixture::new("big_hits", events)
}

/// `build_advice_cards` が持つ全検出器と、最後の大被弾一覧を一対一で覆う。
pub fn card_fixtures() -> Vec<CardFixture> {
    vec![
        layered_defense(),
        teleport_defense(),
        anti_air(),
        own_jumps(),
        burnout(),
        committed_button_vs_di(),
        mashing(),
        minus_press(DefensiveActionKind::Strike, "press_while_minus"),
        minus_press(DefensiveActionKind::Throw, "throw_while_minus"),
        advantage_abandoned(),
        guard_break(),
        reversal_punished(),
        low_scaling_super(),
        punish_fail(),
        punish_missed(),
        low_conversion(),
        throw_interrupted_by_invincible(),
        throw_whiff_punished(),
        whiff_punished(),
        throw_loop(),
        early_hits(),
        lead_loss(),
        big_hits(),
    ]
}
