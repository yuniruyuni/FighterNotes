import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import {
  aggregateMatchups,
  createAnalysisHistoryRecord,
  type MatchupSummary,
} from "../domain/history.js";
import type { AnalysisHistoryRepository } from "./ports.js";

export async function recordAndSummarizeMatchups(
  file: File,
  context: AnalysisContext,
  report: AdviceReport,
  repository: AnalysisHistoryRepository,
): Promise<MatchupSummary[]> {
  await repository.save(
    await createAnalysisHistoryRecord(file, context, report),
  );
  return aggregateMatchups(await repository.load(), report.ruleset_version);
}
