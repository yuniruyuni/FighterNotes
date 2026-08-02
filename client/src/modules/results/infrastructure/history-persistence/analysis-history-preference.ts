import type { AnalysisHistorySavingPreference } from "../../application/ports.js";

const STORAGE_KEY = "fighter-notes:analysis-history:saving-enabled:v1";
const ENABLED_VALUE = "enabled";
const DISABLED_VALUE = "disabled";

export interface AnalysisHistoryPreferenceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export function loadAnalysisHistorySavingPreference(
  storage:
    | AnalysisHistoryPreferenceStorage
    | null
    | undefined = browserStorage(),
): AnalysisHistorySavingPreference {
  if (!storage) return { enabled: false, persistent: false };
  try {
    const value = storage.getItem(STORAGE_KEY);
    if (value === null || value === ENABLED_VALUE) {
      return { enabled: true, persistent: true };
    }
    if (value === DISABLED_VALUE) {
      return { enabled: false, persistent: true };
    }
    return { enabled: false, persistent: true };
  } catch {
    return { enabled: false, persistent: false };
  }
}

export function saveAnalysisHistorySavingPreference(
  enabled: boolean,
  storage:
    | AnalysisHistoryPreferenceStorage
    | null
    | undefined = browserStorage(),
): void {
  if (!storage) {
    throw new Error("解析履歴の保存設定をこのブラウザに保存できません。");
  }
  try {
    storage.setItem(STORAGE_KEY, enabled ? ENABLED_VALUE : DISABLED_VALUE);
  } catch {
    throw new Error("解析履歴の保存設定をこのブラウザに保存できません。");
  }
}

function browserStorage(): AnalysisHistoryPreferenceStorage | undefined {
  try {
    return typeof localStorage === "undefined" ? undefined : localStorage;
  } catch {
    return undefined;
  }
}
