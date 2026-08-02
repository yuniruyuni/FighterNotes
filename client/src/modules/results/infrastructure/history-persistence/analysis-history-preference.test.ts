import { describe, expect, test } from "bun:test";
import {
  type AnalysisHistoryPreferenceStorage,
  loadAnalysisHistorySavingPreference,
  saveAnalysisHistorySavingPreference,
} from "./analysis-history-preference.js";

class MemoryStorage implements AnalysisHistoryPreferenceStorage {
  value: string | null = null;

  getItem(): string | null {
    return this.value;
  }

  setItem(_key: string, value: string): void {
    this.value = value;
  }
}

describe("analysis history saving preference", () => {
  test("初期値を有効にし、無効化と再有効化をreload相当で復元する", () => {
    const storage = new MemoryStorage();
    expect(loadAnalysisHistorySavingPreference(storage)).toEqual({
      enabled: true,
      persistent: true,
    });

    saveAnalysisHistorySavingPreference(false, storage);
    expect(loadAnalysisHistorySavingPreference(storage)).toEqual({
      enabled: false,
      persistent: true,
    });

    saveAnalysisHistorySavingPreference(true, storage);
    expect(loadAnalysisHistorySavingPreference(storage)).toEqual({
      enabled: true,
      persistent: true,
    });
  });

  test("壊れた値と利用不能なstorageではprivacy側へfail closedする", () => {
    const corrupted = new MemoryStorage();
    corrupted.value = "unexpected";
    expect(loadAnalysisHistorySavingPreference(corrupted)).toEqual({
      enabled: false,
      persistent: true,
    });
    expect(loadAnalysisHistorySavingPreference(null)).toEqual({
      enabled: false,
      persistent: false,
    });

    const unavailable: AnalysisHistoryPreferenceStorage = {
      getItem: () => {
        throw new Error("blocked");
      },
      setItem: () => {
        throw new Error("blocked");
      },
    };
    expect(loadAnalysisHistorySavingPreference(unavailable)).toEqual({
      enabled: false,
      persistent: false,
    });
    expect(() =>
      saveAnalysisHistorySavingPreference(false, unavailable),
    ).toThrow("解析履歴の保存設定をこのブラウザに保存できません。");
  });
});
