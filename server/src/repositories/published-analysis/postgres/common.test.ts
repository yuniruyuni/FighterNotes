import { describe, expect, test } from "bun:test";
import type {
  PublishedAnalysisContent,
  ShareId,
} from "../../../models/published-analysis";
import { createPublishedAnalysisContent } from "../../../models/published-analysis";
import {
  candidate,
  persistableAnalysis,
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
    },
    findings: content.findings.map((finding) => ({
      kind: finding.kind,
      assessment: finding.assessment,
      occurrences: finding.occurrences,
      severity_bp: finding.severityBp,
    })),
  };
}
