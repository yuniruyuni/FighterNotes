import { describe, expect, test } from "bun:test";
import {
  syntheticAdviceReport,
  syntheticTacticStats,
} from "~/test-support/analysis.js";
import type { AnalysisHistoryRecord } from "../domain/history.js";
import {
  loadMatchupHistory,
  recordAndLoadMatchupHistory,
  recordAndSummarizeMatchups,
} from "./matchup-history.js";
import type { AnalysisHistoryRepository } from "./ports.js";

class MemoryHistoryRepository implements AnalysisHistoryRepository {
  readonly records: AnalysisHistoryRecord[] = [];
  savingEnabled = true;

  async save(record: AnalysisHistoryRecord): Promise<void> {
    this.records.push(record);
  }

  async load(): Promise<AnalysisHistoryRecord[]> {
    return [...this.records];
  }

  async delete(id: string): Promise<void> {
    const index = this.records.findIndex((record) => record.id === id);
    if (index >= 0) this.records.splice(index, 1);
  }

  async clear(): Promise<void> {
    this.records.length = 0;
  }

  async getSavingPreference() {
    return { enabled: this.savingEnabled, persistent: true };
  }

  async setSavingEnabled(enabled: boolean): Promise<void> {
    this.savingEnabled = enabled;
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

  test("保存OFFでは新規recordを作らず、既存rulesetだけを読み込む", async () => {
    const repository = new MemoryHistoryRepository();
    const report = syntheticAdviceReport({ rounds_detected: 2 });
    await recordAndLoadMatchupHistory(
      new File(["first"], "first.mp4", { type: "video/mp4" }),
      {
        ownSide: "p1",
        p1: { character: "JURI" },
        p2: { character: "KEN" },
      },
      report,
      repository,
    );
    const current = repository.records[0];
    if (!current) throw new Error("current history record was not saved");
    repository.records.push({
      ...current,
      id: "v2:legacy-ruleset",
      rulesetVersion: report.ruleset_version - 1,
      createdAt: "2026-01-01T00:00:00.000Z",
    });
    repository.savingEnabled = false;

    const snapshot = await recordAndLoadMatchupHistory(
      new File(["second"], "second.mp4", { type: "video/mp4" }),
      {
        ownSide: "p1",
        p1: { character: "JURI" },
        p2: { character: "KEN" },
      },
      report,
      repository,
    );

    expect(repository.records).toHaveLength(2);
    expect(snapshot.records).toHaveLength(2);
    expect(snapshot.summaries).toHaveLength(1);
    expect(snapshot.saving).toEqual({ enabled: false, persistent: true });
  });

  test("個別削除と全削除の後に全rulesetの保存件数を再読込できる", async () => {
    const repository = new MemoryHistoryRepository();
    const report = syntheticAdviceReport();
    for (const id of ["v2:first", "v2:second"]) {
      repository.records.push({
        id,
        createdAt: "2026-08-03T00:00:00.000Z",
        rulesetVersion: report.ruleset_version,
        ownCharacter: "JURI",
        opponentCharacter: "KEN",
        rounds: 2,
        tactics: report.tactic_stats,
      });
    }

    await repository.delete("v2:first");
    expect(
      (
        await loadMatchupHistory(report.ruleset_version, repository)
      ).records.map((record) => record.id),
    ).toEqual(["v2:second"]);
    await repository.clear();
    expect(
      (await loadMatchupHistory(report.ruleset_version, repository)).records,
    ).toEqual([]);
  });

  /**
   * 履歴一覧は新しい順で見せる。repository の返す順序は保証されないため、
   * 読み込み側で必ず並べ直す。
   */
  test("保存順にかかわらず新しい順で返す", async () => {
    const repository = new MemoryHistoryRepository();
    const report = syntheticAdviceReport();
    const createdAt = [
      "2026-08-01T00:00:00.000Z",
      "2026-08-05T00:00:00.000Z",
      "2026-08-03T00:00:00.000Z",
    ];
    for (const [index, timestamp] of createdAt.entries()) {
      repository.records.push({
        id: `v2:record-${index}`,
        createdAt: timestamp,
        rulesetVersion: report.ruleset_version,
        ownCharacter: "JURI",
        opponentCharacter: "KEN",
        rounds: 2,
        tactics: report.tactic_stats,
      });
    }

    const snapshot = await loadMatchupHistory(
      report.ruleset_version,
      repository,
    );

    expect(snapshot.records.map((record) => record.createdAt)).toEqual([
      "2026-08-05T00:00:00.000Z",
      "2026-08-03T00:00:00.000Z",
      "2026-08-01T00:00:00.000Z",
    ]);
  });
});
