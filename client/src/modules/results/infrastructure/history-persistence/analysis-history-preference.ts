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
    const normalized =
      value === null || value === ENABLED_VALUE
        ? ENABLED_VALUE
        : DISABLED_VALUE;
    // Reading can succeed even when browser policy or quota makes storage
    // read-only. Persist the canonical value before treating the preference as
    // usable so an enabled default never bypasses the fail-closed guarantee.
    storage.setItem(STORAGE_KEY, normalized);
    return { enabled: normalized === ENABLED_VALUE, persistent: true };
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
