import { describe, expect, test } from "bun:test";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  syntheticAdviceReport,
  syntheticTacticStats,
} from "~/test-support/analysis.js";
import type {
  AnalysisHistoryRepository,
  ResultsServices,
} from "../../application/ports.js";
import type { AnalysisHistoryRecord } from "../../domain/history.js";
import { ResultsServicesProvider } from "../ResultsServicesProvider.js";
import { MatchupHistorySection } from "./MatchupHistorySection.js";

class MemoryHistoryRepository implements AnalysisHistoryRepository {
  savingEnabled = false;
  failDelete = false;

  constructor(readonly records: AnalysisHistoryRecord[]) {}

  async save(record: AnalysisHistoryRecord): Promise<void> {
    this.records.push(record);
  }

  async load(): Promise<AnalysisHistoryRecord[]> {
    return [...this.records];
  }

  async delete(id: string): Promise<void> {
    if (this.failDelete) throw new Error("delete failed");
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

const report = syntheticAdviceReport({
  ruleset_version: 6,
  tactic_stats: syntheticTacticStats({ anti_air_opportunities: 1 }),
});
const context = {
  ownSide: "p1" as const,
  p1: { character: "JURI" },
  p2: { character: "KEN" },
};

function historyRecord(
  id: string,
  rulesetVersion: number,
  ownCharacter = "JURI",
  opponentCharacter = "KEN",
): AnalysisHistoryRecord {
  return {
    id,
    createdAt: "2026-08-03T10:00:00.000Z",
    rulesetVersion,
    ownCharacter,
    opponentCharacter,
    rounds: 2,
    tactics: report.tactic_stats,
  };
}

function renderHistory(repository: AnalysisHistoryRepository) {
  const services: ResultsServices = {
    history: repository,
    debugFrameInspector: {} as ResultsServices["debugFrameInspector"],
    debugFrameSourceFactory: {} as ResultsServices["debugFrameSourceFactory"],
  };
  return render(
    <ResultsServicesProvider services={services}>
      <MatchupHistorySection
        context={context}
        file={
          new File(["video"], "never-stored.mp4", {
            type: "video/mp4",
            lastModified: 1,
          })
        }
        report={report}
      />
    </ResultsServicesProvider>,
  );
}

describe("MatchupHistorySection privacy controls", () => {
  test("保存設定、全判定版の件数、個別削除と全削除を管理する", async () => {
    const user = userEvent.setup();
    const repository = new MemoryHistoryRepository([
      historyRecord("v2:current", 6),
      historyRecord("v2:legacy", 5, "AKI", "RYU"),
    ]);
    renderHistory(repository);

    const toggle = await screen.findByRole("checkbox", {
      name: "今後の解析履歴を保存する",
    });
    expect(toggle).not.toBeChecked();
    expect(
      screen.getByText(/動画とファイル名は保存しません/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/共有URLや削除コードには影響しません/),
    ).toBeInTheDocument();

    await user.click(toggle);
    expect(repository.savingEnabled).toBe(true);
    expect(await screen.findByRole("status")).toHaveTextContent(
      "今後の解析履歴を保存します",
    );

    await user.click(screen.getByText("保存済み履歴を管理（全判定版 2件）"));
    expect(
      screen.getByText(/現在の判定版は1件、旧判定版は1件/),
    ).toBeInTheDocument();

    await user.click(
      screen.getByRole("button", { name: /JURI vs KEN.*判定版 6.*を削除/ }),
    );
    expect(screen.getByText("この1件を削除しますか？")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "削除する" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "解析履歴を1件削除しました",
    );
    expect(repository.records.map((record) => record.id)).toEqual([
      "v2:legacy",
    ]);

    await user.click(
      screen.getByRole("button", { name: "解析履歴をすべて削除" }),
    );
    expect(
      screen.getByText(/旧判定版を含む全1件を削除しますか/),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "すべて削除する" }));
    await waitFor(() => expect(repository.records).toHaveLength(0));
    expect(screen.getByText(/全判定版 0件/)).toBeInTheDocument();
    expect(repository.savingEnabled).toBe(true);
  });

  test("削除失敗をalertで通知し、対象を保持する", async () => {
    const user = userEvent.setup();
    const repository = new MemoryHistoryRepository([
      historyRecord("v2:current", 6),
    ]);
    repository.failDelete = true;
    renderHistory(repository);

    await user.click(
      await screen.findByText("保存済み履歴を管理（全判定版 1件）"),
    );
    await user.click(
      screen.getByRole("button", { name: /JURI vs KEN.*判定版 6.*を削除/ }),
    );
    await user.click(screen.getByRole("button", { name: "削除する" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "解析履歴を削除できませんでした",
    );
    expect(repository.records).toHaveLength(1);
  });

  test("破壊操作の確認へfocusを移し、キャンセル後は起点へ戻す", async () => {
    const user = userEvent.setup();
    const repository = new MemoryHistoryRepository([
      historyRecord("v2:first", 6),
      historyRecord("v2:second", 6),
    ]);
    renderHistory(repository);

    await user.click(
      await screen.findByText("保存済み履歴を管理（全判定版 2件）"),
    );
    const deleteButtons = screen.getAllByRole("button", {
      name: /JURI vs KEN.*判定版 6.*を削除/,
    });
    expect(
      new Set(deleteButtons.map((button) => button.getAttribute("aria-label")))
        .size,
    ).toBe(2);

    deleteButtons[0]?.focus();
    await user.keyboard("{Enter}");
    const cancelDelete = screen.getByRole("button", { name: "キャンセル" });
    expect(screen.getByRole("button", { name: "削除する" })).toHaveFocus();
    await user.click(cancelDelete);
    expect(deleteButtons[0]).toHaveFocus();

    const clearButton = screen.getByRole("button", {
      name: "解析履歴をすべて削除",
    });
    clearButton.focus();
    await user.keyboard("{Enter}");
    const clearCancel = screen.getByRole("button", { name: "キャンセル" });
    expect(
      screen.getByRole("button", { name: "すべて削除する" }),
    ).toHaveFocus();
    await user.click(clearCancel);
    expect(clearButton).toHaveFocus();
  });
});
