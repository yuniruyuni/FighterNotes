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
    ]) {
      expect(serialized).not.toContain(marker);
    }
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
});
