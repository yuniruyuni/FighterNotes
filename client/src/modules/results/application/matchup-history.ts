import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import {
  type AnalysisHistoryRecord,
  aggregateMatchups,
  createAnalysisHistoryRecord,
  type MatchupSummary,
} from "../domain/history.js";
import type {
  AnalysisHistoryRepository,
  AnalysisHistorySavingPreference,
} from "./ports.js";

export interface MatchupHistorySnapshot {
  records: AnalysisHistoryRecord[];
  summaries: MatchupSummary[];
  saving: AnalysisHistorySavingPreference;
}

export async function recordAndLoadMatchupHistory(
  file: File,
  context: AnalysisContext,
  report: AdviceReport,
  repository: AnalysisHistoryRepository,
): Promise<MatchupHistorySnapshot> {
  const saving = await repository.getSavingPreference();
  if (saving.enabled) {
    await repository.save(
      await createAnalysisHistoryRecord(file, context, report),
    );
  }
  return loadMatchupHistory(report.ruleset_version, repository, saving);
}

export async function loadMatchupHistory(
  rulesetVersion: number,
  repository: AnalysisHistoryRepository,
  knownSaving?: AnalysisHistorySavingPreference,
): Promise<MatchupHistorySnapshot> {
  const [records, saving] = await Promise.all([
    repository.load(),
    knownSaving
      ? Promise.resolve(knownSaving)
      : repository.getSavingPreference(),
  ]);
  records.sort((left, right) => right.createdAt.localeCompare(left.createdAt));
  return {
    records,
    summaries: aggregateMatchups(records, rulesetVersion),
    saving,
  };
}

export async function recordAndSummarizeMatchups(
  file: File,
  context: AnalysisContext,
  report: AdviceReport,
  repository: AnalysisHistoryRepository,
): Promise<MatchupSummary[]> {
  return (await recordAndLoadMatchupHistory(file, context, report, repository))
    .summaries;
}
