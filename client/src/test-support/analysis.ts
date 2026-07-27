import type {
  AdviceReport,
  AnalysisResult,
  TacticStats,
} from "~/modules/analysis/contracts.js";

export function syntheticTacticStats(
  overrides: Partial<TacticStats> = {},
): TacticStats {
  return {
    anti_air_opportunities: 0,
    anti_air_successes: 0,
    jump_ins_allowed: 0,
    di_faced: 0,
    di_returned: 0,
    di_blocked: 0,
    di_parried: 0,
    di_hit: 0,
    di_avoided: 0,
    di_unconfirmed: 0,
    raw_drive_rushes_faced: 0,
    raw_drive_rushes_defended: 0,
    raw_drive_rushes_hit: 0,
    raw_drive_rushes_unconfirmed: 0,
    dash_throws_faced: 0,
    throw_whiffs: 0,
    minus_defense_opportunities: 0,
    fastest_strike_challenges: 0,
    fastest_strike_losses: 0,
    fastest_throw_challenges: 0,
    fastest_throw_losses: 0,
    burnout_count: 0,
    burnout_seconds: 0,
    burnout_hp_lost: 0,
    burnout_hp_dealt: 0,
    burnout_self_initiated: 0,
    burnout_forced: 0,
    burnout_mixed: 0,
    burnout_unknown: 0,
    ...overrides,
  };
}

export function syntheticAdviceReport(
  overrides: Partial<AdviceReport> = {},
): AdviceReport {
  return {
    ruleset_version: 6,
    total_frames: 0,
    rounds_detected: 0,
    damage_taken_events: [],
    weaknesses: [],
    practice_items: [],
    summary: "",
    cards: [],
    round_summaries: [],
    input_stats: null,
    tactic_stats: syntheticTacticStats(),
    ...overrides,
  };
}

export function syntheticAnalysisResult(
  report = syntheticAdviceReport(),
): AnalysisResult {
  return {
    analysisContext: { ownSide: "p1", p1: {}, p2: {} },
    report,
    timeline: {
      left: { side: "left", segments: [] },
      right: { side: "right", segments: [] },
      video_map: {},
    },
    trackedInputs: null,
    hpFeatures: [],
    frameCount: 0,
    frameTimestamps: [],
    sampleData: [],
    videoArrayBuffer: new ArrayBuffer(8),
    codecConfig: null,
    frameToSampleIdx: [],
    spatialObservations: [],
  };
}
