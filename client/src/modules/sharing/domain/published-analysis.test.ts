import { describe, expect, test } from "bun:test";
import type {
  AdviceReport,
  AnalysisContext,
  TacticStats,
} from "~/modules/analysis/contracts.js";
import {
  PublishedAnalysisCandidate,
  ShareProjectionError,
} from "./published-analysis";
import { SHAREABLE_FINDING_KINDS } from "./published-analysis-contract";

const tactics = (): TacticStats => ({
  anti_air_opportunities: 3,
  anti_air_successes: 2,
  jump_ins_allowed: 1,
  di_faced: 2,
  di_returned: 1,
  di_blocked: 0,
  di_parried: 0,
  di_hit: 1,
  di_avoided: 0,
  di_unconfirmed: 0,
  raw_drive_rushes_faced: 2,
  raw_drive_rushes_defended: 1,
  raw_drive_rushes_hit: 1,
  raw_drive_rushes_unconfirmed: 0,
  dash_throws_faced: 1,
  throw_whiffs: 2,
  minus_defense_opportunities: 5,
  knockdowns_scored: 0,
  okizeme_meaty: 0,
  okizeme_pressured: 0,
  okizeme_neutral: 0,
  knockdowns_taken: 0,
  okizeme_faced_meaty: 0,
  own_di_used: 0,
  own_di_hit: 0,
  own_di_blocked: 0,
  own_di_parried: 0,
  own_di_countered: 0,
  own_di_whiffed: 0,
  own_di_unconfirmed: 0,
  own_raw_drive_rushes: 0,
  own_raw_drive_rush_hits: 0,
  own_raw_drive_rush_defended: 0,
  drive_spent_on_impacts: 0,
  drive_spent_on_rushes: 0,
  drive_damage_from_impacts: 0,
  drive_damage_from_rushes: 0,
  drive_spend_samples: 0,
  whiffs: 0,
  whiffs_punished: 0,
  opponent_whiffs: 0,
  opponent_whiffs_punished: 0,
  advantage_opportunities: 0,
  advantage_continued: 0,
  advantage_abandoned: 0,
  advantage_turns_lost: 0,
  fastest_strike_challenges: 3,
  fastest_strike_losses: 1,
  fastest_throw_challenges: 2,
  fastest_throw_losses: 1,
  burnout_count: 1,
  burnout_seconds: 12.34,
  burnout_hp_lost: 0.2134,
  burnout_hp_dealt: 0.0876,
  burnout_self_initiated: 1,
  burnout_forced: 0,
  burnout_mixed: 0,
  burnout_unknown: 0,
});

function report(): AdviceReport {
  return {
    ruleset_version: 3,
    total_frames: 1000,
    rounds_detected: 2,
    damage_taken_events: [],
    weaknesses: [],
    practice_items: ["<script>practice-marker</script>"],
    summary: "<script>summary-marker</script>",
    cards: [
      {
        id: "big_hits",
        kind: "observation",
        title: "<img src=x onerror=title-marker>",
        severity: 0.31234,
        description: "description-marker",
        practice: "card-practice-marker",
        evidence: [
          { frame: 123, end_frame: 180, label: "evidence-label-marker" },
          { frame: 456, label: "another-label-marker" },
        ],
      },
      {
        id: "anti_air",
        kind: "diagnosis",
        title: "対空",
        severity: 0.1,
        description: "description",
        practice: "practice",
        evidence: [{ frame: 100, label: "label" }],
      },
    ],
    round_summaries: [
      {
        round_no: 1,
        start_frame: 0,
        end_frame: 400,
        won: true,
        own_hp_end: 0.4,
        opp_hp_end: 0,
        own_hp_lost: 0.6,
        opp_hp_lost: 1,
        own_hits_taken: 2,
        early_hit: false,
        own_burnouts: 0,
      },
      {
        round_no: 2,
        start_frame: 401,
        end_frame: 900,
        won: null,
        own_hp_end: 0.2,
        opp_hp_end: 0.1,
        own_hp_lost: 0.8,
        opp_hp_lost: 0.9,
        own_hits_taken: 3,
        early_hit: true,
        own_burnouts: 1,
      },
    ],
    input_stats: null,
    tactic_stats: tactics(),
  };
}

describe("share projection", () => {
  test("2P視点を自分対相手へ並べ替え、整数単位へ変換する", () => {
    const context: AnalysisContext = {
      ownSide: "p2",
      p1: { character: "CHUN_LI" },
      p2: { character: "LUKE" },
    };
    const candidate = PublishedAnalysisCandidate.from(context, report());
    expect(candidate.ownCharacter).toBe("LUKE");
    expect(candidate.opponentCharacter).toBe("CHUN_LI");
    expect(candidate.rounds).toEqual({
      detected: 2,
      won: 1,
      lost: 0,
      unresolved: 1,
    });
    expect(candidate.findings.map((finding) => finding.kind)).toEqual([
      "anti_air",
      "big_hits",
    ]);
    expect(candidate.findings[1]).toEqual({
      kind: "big_hits",
      assessment: "observation",
      occurrences: 2,
      severityBp: 3123,
    });
    expect(candidate.tactics.burnout).toMatchObject({
      durationDeciseconds: 123,
      hpLostBp: 2134,
      hpDealtBp: 876,
    });
  });

  test("勝敗と未確定roundをそれぞれ一度だけ数える", () => {
    const value = report();
    value.round_summaries.push({
      ...value.round_summaries[0],
      round_no: 3,
      won: false,
    });
    const context: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "LUKE" },
      p2: { character: "CHUN_LI" },
    };

    expect(PublishedAnalysisCandidate.from(context, value).rounds).toEqual({
      detected: 3,
      won: 1,
      lost: 1,
      unresolved: 1,
    });
  });

  test("自由文、証拠frame、動画依存値を候補へ含めない", () => {
    const context: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "LUKE", controlType: "control-marker" },
      p2: { character: "CHUN_LI", controlType: "opponent-control-marker" },
      battleVersion: "battle-version-marker",
    };
    const candidate = PublishedAnalysisCandidate.from(context, report());
    expect(candidate.ownCharacter).toBe("LUKE");
    expect(candidate.opponentCharacter).toBe("CHUN_LI");
    const serialized = JSON.stringify(candidate);
    for (const marker of [
      "summary-marker",
      "title-marker",
      "description-marker",
      "practice-marker",
      "evidence-label-marker",
      "control-marker",
      "opponent-control-marker",
      "battle-version-marker",
      '"frame"',
      '"end_frame"',
      "super_damage_samples",
      "super_reported_combo_damage",
      "super_reported_marginal_damage",
      "super_low_scaling_uses",
      "super_gauge_end",
      "opponent_super_gauge_end",
    ]) {
      expect(serialized).not.toContain(marker);
    }
    expect(candidate).not.toHaveProperty("superArts");
  });

  test("ruleset v9は両者の公開可能なSA/CA集計だけを射影する", () => {
    const value = report();
    value.ruleset_version = 9;
    Object.assign(value.tactic_stats, {
      super_art_stats_complete: true,
      opponent_super_art_stats_complete: true,
      sa1_used: 1,
      sa2_used: 2,
      sa3_used: 3,
      ca_used: 4,
      super_hits: 5,
      super_blocked: 6,
      super_no_immediate_contact: 7,
      super_punished: 8,
      super_kos: 9,
      super_combo_uses: 10,
      super_punish_uses: 11,
      super_reversal_uses: 12,
      super_neutral_uses: 13,
      opponent_sa1_used: 14,
      opponent_sa2_used: 15,
      opponent_sa3_used: 16,
      opponent_ca_used: 17,
      opponent_super_hits: 18,
      opponent_super_blocked: 19,
      opponent_super_no_immediate_contact: 20,
      opponent_super_punished: 21,
      opponent_super_kos: 22,
      super_damage_samples: 901,
      super_reported_combo_damage: 902,
      super_reported_marginal_damage: 903,
      super_low_scaling_uses: 904,
      super_gauge_end: 2.75,
      opponent_super_gauge_end: 1.5,
    });
    const context: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "LUKE" },
      p2: { character: "CHUN_LI" },
    };

    const candidate = PublishedAnalysisCandidate.from(context, value);
    expect(candidate.superArts).toEqual({
      own: {
        availability: "complete",
        levels: { sa1: 1, sa2: 2, sa3: 3, ca: 4 },
        outcomes: {
          hit: 5,
          block: 6,
          noImmediateContact: 7,
          punished: 8,
          ko: 9,
        },
        contexts: { combo: 10, punish: 11, reversal: 12, neutral: 13 },
      },
      opponent: {
        availability: "complete",
        levels: { sa1: 14, sa2: 15, sa3: 16, ca: 17 },
        outcomes: {
          hit: 18,
          block: 19,
          noImmediateContact: 20,
          punished: 21,
          ko: 22,
        },
      },
    });
    const serialized = JSON.stringify(candidate);
    for (const forbidden of [
      "super_damage_samples",
      "super_reported_combo_damage",
      "super_reported_marginal_damage",
      "super_low_scaling_uses",
      "super_gauge_end",
      "opponent_super_gauge_end",
      "901",
      "902",
      "903",
      "904",
      "2.75",
      "1.5",
    ]) {
      expect(serialized).not.toContain(forbidden);
    }
  });

  test("不完全なSA/CA集計を検出済み下限として射影する", () => {
    const value = report();
    value.ruleset_version = 9;
    Object.assign(value.tactic_stats, {
      super_art_stats_complete: false,
      opponent_super_art_stats_complete: false,
      sa1_used: 1,
      sa2_used: 0,
      sa3_used: 0,
      ca_used: 0,
      super_hits: 1,
      super_blocked: 0,
      super_no_immediate_contact: 0,
      super_punished: 0,
      super_kos: 0,
      super_combo_uses: 1,
      super_punish_uses: 0,
      super_reversal_uses: 0,
      super_neutral_uses: 0,
      opponent_sa1_used: 0,
      opponent_sa2_used: 0,
      opponent_sa3_used: 1,
      opponent_ca_used: 0,
      opponent_super_hits: 0,
      opponent_super_blocked: 1,
      opponent_super_no_immediate_contact: 0,
      opponent_super_punished: 0,
      opponent_super_kos: 0,
    });
    const context: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "LUKE" },
      p2: { character: "CHUN_LI" },
    };

    expect(PublishedAnalysisCandidate.from(context, value).superArts).toEqual({
      own: {
        availability: "partial",
        levels: { sa1: 1, sa2: 0, sa3: 0, ca: 0 },
        outcomes: {
          hit: 1,
          block: 0,
          noImmediateContact: 0,
          punished: 0,
          ko: 0,
        },
        contexts: { combo: 1, punish: 0, reversal: 0, neutral: 0 },
      },
      opponent: {
        availability: "partial",
        levels: { sa1: 0, sa2: 0, sa3: 1, ca: 0 },
        outcomes: {
          hit: 0,
          block: 1,
          noImmediateContact: 0,
          punished: 0,
          ko: 0,
        },
      },
    });
  });

  test("不完全で検出0件ならcountを含めず、availability欠落を拒否する", () => {
    const value = report();
    value.ruleset_version = 9;
    Object.assign(value.tactic_stats, {
      super_art_stats_complete: false,
      opponent_super_art_stats_complete: false,
      sa1_used: 0,
      sa2_used: 0,
      sa3_used: 0,
      ca_used: 0,
      super_hits: 0,
      super_blocked: 0,
      super_no_immediate_contact: 0,
      super_punished: 0,
      super_kos: 0,
      super_combo_uses: 0,
      super_punish_uses: 0,
      super_reversal_uses: 0,
      super_neutral_uses: 0,
      opponent_sa1_used: 0,
      opponent_sa2_used: 0,
      opponent_sa3_used: 0,
      opponent_ca_used: 0,
      opponent_super_hits: 0,
      opponent_super_blocked: 0,
      opponent_super_no_immediate_contact: 0,
      opponent_super_punished: 0,
      opponent_super_kos: 0,
    });
    const context: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "LUKE" },
      p2: { character: "CHUN_LI" },
    };

    expect(PublishedAnalysisCandidate.from(context, value).superArts).toEqual({
      own: { availability: "unavailable" },
      opponent: { availability: "unavailable" },
    });

    value.tactic_stats.super_art_stats_complete = true;
    value.tactic_stats.opponent_super_art_stats_complete = true;
    const complete = PublishedAnalysisCandidate.from(context, value).superArts;
    expect(complete?.own).toMatchObject({
      availability: "complete",
      levels: { sa1: 0, sa2: 0, sa3: 0, ca: 0 },
    });
    expect(complete?.opponent).toMatchObject({
      availability: "complete",
      levels: { sa1: 0, sa2: 0, sa3: 0, ca: 0 },
    });

    value.tactic_stats.super_art_stats_complete = undefined;
    expect(() => PublishedAnalysisCandidate.from(context, value)).toThrow(
      "superArts.own.availability が不正です。",
    );
  });

  test("全finding IDを共有候補へ変換できる", () => {
    const value = report();
    value.cards = SHAREABLE_FINDING_KINDS.map((id) => ({
      id,
      title: id,
      severity: 0,
      description: id,
      practice: id,
      evidence: [{ frame: 1, label: id }],
    }));
    const context: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "LUKE" },
      p2: { character: "CHUN_LI" },
    };
    expect(
      PublishedAnalysisCandidate.from(context, value).findings.map(
        (finding) => finding.kind,
      ),
    ).toEqual([...SHAREABLE_FINDING_KINDS]);
  });

  test("キャラクター未指定、未知finding、NaNを拒否する", () => {
    const context: AnalysisContext = {
      ownSide: "p1",
      p1: {},
      p2: { character: "CHUN_LI" },
    };
    expect(() => PublishedAnalysisCandidate.from(context, report())).toThrow(
      "自分のキャラクターを選択すると共有できます。",
    );

    const missingOpponent: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "LUKE" },
      p2: {},
    };
    expect(() =>
      PublishedAnalysisCandidate.from(missingOpponent, report()),
    ).toThrow("相手のキャラクターを選択すると共有できます。");

    const unsupportedCharacter: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "UNKNOWN" },
      p2: { character: "CHUN_LI" },
    };
    expect(() =>
      PublishedAnalysisCandidate.from(unsupportedCharacter, report()),
    ).toThrow(ShareProjectionError);

    context.p1.character = "LUKE";
    const unknown = report();
    unknown.cards[0].id = "future_detector";
    expect(() => PublishedAnalysisCandidate.from(context, unknown)).toThrow(
      ShareProjectionError,
    );

    const invalid = report();
    invalid.tactic_stats.burnout_seconds = Number.NaN;
    expect(() => PublishedAnalysisCandidate.from(context, invalid)).toThrow(
      ShareProjectionError,
    );

    const invalidRuleset = report();
    invalidRuleset.ruleset_version = Number.NaN;
    expect(() =>
      PublishedAnalysisCandidate.from(context, invalidRuleset),
    ).toThrow("rulesetVersion が不正です。");

    const tooManyRounds = report();
    tooManyRounds.round_summaries = Array.from({ length: 256 }, (_, index) => ({
      ...tooManyRounds.round_summaries[0],
      round_no: index + 1,
      won: index === 255 ? true : null,
    }));
    expect(
      PublishedAnalysisCandidate.from(context, tooManyRounds).rounds,
    ).toEqual({ detected: 255, won: 0, lost: 0, unresolved: 255 });
  });

  test("ruleset v3からv13を共有し、それ以外は理由付きで拒否する", () => {
    const context: AnalysisContext = {
      ownSide: "p1",
      p1: { character: "LUKE" },
      p2: { character: "CHUN_LI" },
    };
    for (const rulesetVersion of [3, 4, 5, 6, 7, 8]) {
      const value = report();
      value.ruleset_version = rulesetVersion;
      expect(
        PublishedAnalysisCandidate.from(context, value).rulesetVersion,
      ).toBe(rulesetVersion);
    }

    // v9以降はSA/CA集計を必須にするため、同じ形で個別に確認する。
    for (const rulesetVersion of [9, 10, 11, 12, 13]) {
      const current = report();
      current.ruleset_version = rulesetVersion;
      Object.assign(current.tactic_stats, {
        super_art_stats_complete: false,
        opponent_super_art_stats_complete: false,
        sa1_used: 0,
        sa2_used: 0,
        sa3_used: 0,
        ca_used: 0,
        opponent_sa1_used: 0,
        opponent_sa2_used: 0,
        opponent_sa3_used: 0,
        opponent_ca_used: 0,
      });
      expect(
        PublishedAnalysisCandidate.from(context, current).rulesetVersion,
      ).toBe(rulesetVersion);
    }

    const unsupported = report();
    unsupported.ruleset_version = 14;
    expect(() => PublishedAnalysisCandidate.from(context, unsupported)).toThrow(
      "この解析ルール世代は共有に対応していません",
    );
  });
});
