import { describe, expect, test } from "bun:test";
import {
  syntheticAdviceReport,
  syntheticTacticStats,
} from "~/test-support/analysis.js";
import type { AnalysisHistoryRecord } from "../domain/history.js";
import { recordAndSummarizeMatchups } from "./matchup-history.js";
import type { AnalysisHistoryRepository } from "./ports.js";

class MemoryHistoryRepository implements AnalysisHistoryRepository {
  readonly records: AnalysisHistoryRecord[] = [];

  async save(record: AnalysisHistoryRecord): Promise<void> {
    this.records.push(record);
  }

  async load(): Promise<AnalysisHistoryRecord[]> {
    return [...this.records];
  }
}

describe("recordAndSummarizeMatchups", () => {
  test("新しい解析を保存して同じrulesetの組み合わせを集計する", async () => {
    const repository = new MemoryHistoryRepository();
    const report = syntheticAdviceReport({
      rounds_detected: 2,
      tactic_stats: syntheticTacticStats({
        anti_air_opportunities: 3,
        anti_air_successes: 2,
      }),
    });

    const summaries = await recordAndSummarizeMatchups(
      new File(["video"], "replay.mp4", { type: "video/mp4" }),
      {
        ownSide: "p1",
        p1: { character: "JURI" },
        p2: { character: "KEN" },
      },
      report,
      repository,
    );

    expect(repository.records).toHaveLength(1);
    expect(repository.records[0]).toMatchObject({
      rulesetVersion: 6,
      ownCharacter: "JURI",
      opponentCharacter: "KEN",
      rounds: 2,
    });
    expect(summaries).toMatchObject([
      {
        matches: 1,
        antiAirOpportunities: 3,
        antiAirSuccesses: 2,
      },
    ]);
  });
});
