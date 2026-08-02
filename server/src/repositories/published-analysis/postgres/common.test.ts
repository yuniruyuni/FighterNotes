import { describe, expect, test } from "bun:test";
import type {
  PublishedAnalysisContent,
  ShareId,
} from "../../../models/published-analysis";
import { createPublishedAnalysisContent } from "../../../models/published-analysis";
import {
  candidate,
  persistableAnalysis,
  v9Candidate,
} from "../../test-support/published-analysis";
import {
  type AnalysisRow,
  type FindingRow,
  hydratePublishedAnalysis,
} from "./common";

describe("hydratePublishedAnalysis", () => {
  test("storage rowから全フィールドを復元する", () => {
    const persisted = persistableAnalysis();
    const { row, findings } = storageRows(persisted.content);

    expect(hydratePublishedAnalysis(row, findings)).toEqual({
      id: persisted.id,
      content: persisted.content,
      createdAt: persisted.createdAt,
      expiresAt: persisted.expiresAt,
    });
  });

  test("ruleset v9の両者SA/CA集計とavailabilityを復元する", () => {
    const content = validContent(v9Candidate());
    const { row, findings } = storageRows(content);

    expect(hydratePublishedAnalysis(row, findings)?.content.superArts).toEqual(
      content.superArts,
    );
    expect(
      hydratePublishedAnalysis(
        {
          ...row,
          super_art_analysis_id: null,
          own_super_art_analysis_id: null,
          opponent_super_art_analysis_id: null,
        },
        findings,
      ),
    ).toBeNull();
  });

  test("ruleset v9のmarker有・side行なしを集計不能として復元する", () => {
    const input = v9Candidate();
    input.superArts = {
      own: { availability: "unavailable" },
      opponent: { availability: "unavailable" },
    };
    const content = validContent(input);
    const { row, findings } = storageRows(content);

    expect(row.super_art_analysis_id).not.toBeNull();
    expect(row.own_super_art_analysis_id).toBeNull();
    expect(row.opponent_super_art_analysis_id).toBeNull();
    expect(hydratePublishedAnalysis(row, findings)?.content.superArts).toEqual(
      content.superArts,
    );
  });

  test("ruleset v9のcomplete=false side行をpartialとして復元する", () => {
    const input = v9Candidate();
    const observed = input.superArts;
    if (!observed || observed.own.availability === "unavailable") {
      throw new Error("fixture is invalid");
    }
    input.superArts = {
      own: { ...observed.own, availability: "partial" },
      opponent: { availability: "unavailable" },
    };
    const content = validContent(input);
    const { row, findings } = storageRows(content);

    expect(row.own_super_art_complete).toBe(false);
    expect(hydratePublishedAnalysis(row, findings)?.content.superArts).toEqual(
      content.superArts,
    );
  });

  test("complete=falseなのに検出済み使用が0件のdrift rowをfail closedする", () => {
    const content = validContent(v9Candidate());
    const { row, findings } = storageRows(content);

    expect(
      hydratePublishedAnalysis(
        {
          ...row,
          own_super_art_complete: false,
          own_sa1: 0,
          own_sa2: 0,
          own_sa3: 0,
          own_ca: 0,
        },
        findings,
      ),
    ).toBeNull();
  });

  test("ruleset v8 rowにSA/CA markerが混入した場合はfail closedする", () => {
    const legacy = candidate();
    legacy.rulesetVersion = 8;
    const { row, findings } = storageRows(validContent(legacy));
    expect(
      hydratePublishedAnalysis(
        { ...row, super_art_analysis_id: row.id },
        findings,
      ),
    ).toBeNull();
  });

  test("schema versionが異なる行を拒否する", () => {
    const { row, findings } = storageRows(validContent());
    expect(
      hydratePublishedAnalysis({ ...row, schema_version: 2 }, findings),
    ).toBeNull();
  });

  test("presentation revisionが異なる行を拒否する", () => {
    const { row, findings } = storageRows(validContent());
    expect(
      hydratePublishedAnalysis({ ...row, presentation_revision: 2 }, findings),
    ).toBeNull();
  });

  test("モデル制約を満たさない再構築結果を拒否する", () => {
    const { row, findings } = storageRows(validContent());
    expect(
      hydratePublishedAnalysis({ ...row, rounds_detected: 3 }, findings),
    ).toBeNull();
  });

  test("legacy findingのNULL assessmentを当時の規則で復元する", () => {
    const { row, findings } = storageRows(validContent());
    const legacyRows = findings.map((finding, index) =>
      index === 0 ? { ...finding, assessment: null } : finding,
    );

    expect(
      hydratePublishedAnalysis(row, legacyRows)?.content.findings[0].assessment,
    ).toBe("diagnosis");
  });

  test("保存済みassessmentをlegacy既定値で上書きしない", () => {
    const content = validContent({
      ...candidate(),
      rulesetVersion: 6,
      findings: [
        {
          kind: "anti_air",
          assessment: "statistic",
          occurrences: 1,
          severityBp: 100,
        },
      ],
    });
    const { row, findings } = storageRows(content);

    expect(
      hydratePublishedAnalysis(row, findings)?.content.findings[0].assessment,
    ).toBe("statistic");
  });
});

function validContent(input: unknown = candidate()): PublishedAnalysisContent {
  const content = createPublishedAnalysisContent(input);
  if (!content.ok) throw new Error("fixture is invalid");
  return content.value;
}

function storageRows(content: PublishedAnalysisContent): {
  row: AnalysisRow;
  findings: FindingRow[];
} {
  const persisted = persistableAnalysis();
  const { tactics } = content;
  const own = content.superArts?.own;
  const opponent = content.superArts?.opponent;
  const ownObserved = own !== undefined && own.availability !== "unavailable";
  const opponentObserved =
    opponent !== undefined && opponent.availability !== "unavailable";
  return {
    row: {
      id: persisted.id as ShareId,
      schema_version: content.schemaVersion,
      ruleset_version: content.rulesetVersion,
      presentation_revision: content.presentationRevision,
      own_character: content.ownCharacter,
      opponent_character: content.opponentCharacter,
      rounds_detected: content.rounds.detected,
      rounds_won: content.rounds.won,
      rounds_lost: content.rounds.lost,
      rounds_unresolved: content.rounds.unresolved,
      created_at: persisted.createdAt.toISOString(),
      expires_at: persisted.expiresAt.toISOString(),
      anti_air_opportunities: tactics.antiAir.opportunities,
      anti_air_successes: tactics.antiAir.successes,
      jump_ins_allowed: tactics.antiAir.jumpInsAllowed,
      di_faced: tactics.driveImpact.faced,
      di_returned: tactics.driveImpact.returned,
      di_blocked: tactics.driveImpact.blocked,
      di_parried: tactics.driveImpact.parried,
      di_hit: tactics.driveImpact.hit,
      di_avoided: tactics.driveImpact.avoided,
      di_unconfirmed: tactics.driveImpact.unconfirmed,
      raw_drive_rushes_faced: tactics.rawDriveRush.faced,
      raw_drive_rushes_defended: tactics.rawDriveRush.defended,
      raw_drive_rushes_hit: tactics.rawDriveRush.hit,
      raw_drive_rushes_unconfirmed: tactics.rawDriveRush.unconfirmed,
      dash_throws_faced: tactics.dashThrow.faced,
      throw_whiffs: tactics.throwWhiff.count,
      minus_defense_opportunities: tactics.fastestChallenge.opportunities,
      fastest_strike_challenges: tactics.fastestChallenge.strikeAttempts,
      fastest_strike_losses: tactics.fastestChallenge.strikeLosses,
      fastest_throw_challenges: tactics.fastestChallenge.throwAttempts,
      fastest_throw_losses: tactics.fastestChallenge.throwLosses,
      burnout_count: tactics.burnout.count,
      burnout_duration_deciseconds: tactics.burnout.durationDeciseconds,
      burnout_hp_lost_bp: tactics.burnout.hpLostBp,
      burnout_hp_dealt_bp: tactics.burnout.hpDealtBp,
      burnout_self_initiated: tactics.burnout.selfInitiated,
      burnout_forced: tactics.burnout.forced,
      burnout_mixed: tactics.burnout.mixed,
      burnout_unknown: tactics.burnout.unknown,
      super_art_analysis_id:
        content.superArts === undefined ? null : (persisted.id as ShareId),
      own_super_art_analysis_id: ownObserved ? (persisted.id as ShareId) : null,
      opponent_super_art_analysis_id: opponentObserved
        ? (persisted.id as ShareId)
        : null,
      own_super_art_complete: ownObserved
        ? own.availability === "complete"
        : null,
      opponent_super_art_complete: opponentObserved
        ? opponent.availability === "complete"
        : null,
      own_sa1: ownObserved ? own.levels.sa1 : null,
      own_sa2: ownObserved ? own.levels.sa2 : null,
      own_sa3: ownObserved ? own.levels.sa3 : null,
      own_ca: ownObserved ? own.levels.ca : null,
      own_hit: ownObserved ? own.outcomes.hit : null,
      own_block: ownObserved ? own.outcomes.block : null,
      own_no_immediate_contact: ownObserved
        ? own.outcomes.noImmediateContact
        : null,
      own_punished: ownObserved ? own.outcomes.punished : null,
      own_ko: ownObserved ? own.outcomes.ko : null,
      own_combo: ownObserved ? own.contexts.combo : null,
      own_punish: ownObserved ? own.contexts.punish : null,
      own_reversal: ownObserved ? own.contexts.reversal : null,
      own_neutral: ownObserved ? own.contexts.neutral : null,
      opponent_sa1: opponentObserved ? opponent.levels.sa1 : null,
      opponent_sa2: opponentObserved ? opponent.levels.sa2 : null,
      opponent_sa3: opponentObserved ? opponent.levels.sa3 : null,
      opponent_ca: opponentObserved ? opponent.levels.ca : null,
      opponent_hit: opponentObserved ? opponent.outcomes.hit : null,
      opponent_block: opponentObserved ? opponent.outcomes.block : null,
      opponent_no_immediate_contact: opponentObserved
        ? opponent.outcomes.noImmediateContact
        : null,
      opponent_punished: opponentObserved ? opponent.outcomes.punished : null,
      opponent_ko: opponentObserved ? opponent.outcomes.ko : null,
    },
    findings: content.findings.map((finding) => ({
      kind: finding.kind,
      assessment: finding.assessment,
      occurrences: finding.occurrences,
      severity_bp: finding.severityBp,
    })),
  };
}
