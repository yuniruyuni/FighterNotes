import { useCallback, useEffect, useRef, useState } from "react";
import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import {
  loadMatchupHistory,
  recordAndLoadMatchupHistory,
} from "../../application/matchup-history.js";
import type { AnalysisHistorySavingPreference } from "../../application/ports.js";
import type {
  AnalysisHistoryRecord,
  MatchupSummary,
} from "../../domain/history.js";
import { useResultsServices } from "../ResultsServicesProvider.js";

interface MatchupHistoryNotice {
  kind: "status" | "error";
  message: string;
}

export interface MatchupHistoryState {
  phase: "loading" | "ready" | "error";
  summaries: MatchupSummary[];
  records: AnalysisHistoryRecord[];
  saving: AnalysisHistorySavingPreference;
  busy: "preference" | "delete" | "clear" | null;
  notice: MatchupHistoryNotice | null;
  setSavingEnabled(enabled: boolean): Promise<boolean>;
  deleteRecord(id: string): Promise<boolean>;
  clearHistory(): Promise<boolean>;
}

const UNKNOWN_PREFERENCE: AnalysisHistorySavingPreference = {
  enabled: false,
  persistent: false,
};

type MatchupHistoryData = Pick<
  MatchupHistoryState,
  "phase" | "summaries" | "records" | "saving" | "busy" | "notice"
>;

export function useMatchupHistory(
  file: File,
  context: AnalysisContext,
  report: AdviceReport,
): MatchupHistoryState {
  const { history } = useResultsServices();
  const mounted = useRef(false);
  const generation = useRef(0);
  const [state, setState] = useState<MatchupHistoryData>({
    phase: "loading",
    summaries: [],
    records: [],
    saving: UNKNOWN_PREFERENCE,
    busy: null,
    notice: null,
  });

  useEffect(() => {
    mounted.current = true;
    const currentGeneration = ++generation.current;
    setState({
      phase: "loading",
      summaries: [],
      records: [],
      saving: UNKNOWN_PREFERENCE,
      busy: null,
      notice: null,
    });
    void (async () => {
      try {
        const snapshot = await recordAndLoadMatchupHistory(
          file,
          context,
          report,
          history,
        );
        if (mounted.current && generation.current === currentGeneration) {
          setState({
            phase: "ready",
            ...snapshot,
            busy: null,
            notice: null,
          });
        }
      } catch {
        if (mounted.current && generation.current === currentGeneration) {
          setState((current) => ({
            ...current,
            phase: "error",
            busy: null,
            notice: {
              kind: "error",
              message: "対戦履歴を読み込めませんでした。",
            },
          }));
        }
      }
    })();
    return () => {
      mounted.current = false;
    };
  }, [context, file, history, report]);

  const setSavingEnabled = useCallback(
    async (enabled: boolean): Promise<boolean> => {
      const currentGeneration = generation.current;
      setState((current) => ({
        ...current,
        busy: "preference",
        notice: null,
      }));
      try {
        await history.setSavingEnabled(enabled);
        const saving = await history.getSavingPreference();
        if (mounted.current && generation.current === currentGeneration) {
          setState((current) => ({
            ...current,
            saving,
            busy: null,
            notice: {
              kind: "status",
              message: saving.enabled
                ? "今後の解析履歴を保存します。"
                : "今後の解析履歴を保存しません。既存の履歴は残っています。",
            },
          }));
        }
        return true;
      } catch {
        if (mounted.current && generation.current === currentGeneration) {
          setState((current) => ({
            ...current,
            busy: null,
            notice: {
              kind: "error",
              message: "解析履歴の保存設定を変更できませんでした。",
            },
          }));
        }
        return false;
      }
    },
    [history],
  );

  const deleteRecord = useCallback(
    async (id: string): Promise<boolean> => {
      const currentGeneration = generation.current;
      setState((current) => ({ ...current, busy: "delete", notice: null }));
      try {
        await history.delete(id);
        const snapshot = await loadMatchupHistory(
          report.ruleset_version,
          history,
        );
        if (mounted.current && generation.current === currentGeneration) {
          setState({
            phase: "ready",
            ...snapshot,
            busy: null,
            notice: {
              kind: "status",
              message: "解析履歴を1件削除しました。",
            },
          });
        }
        return true;
      } catch {
        if (mounted.current && generation.current === currentGeneration) {
          setState((current) => ({
            ...current,
            busy: null,
            notice: {
              kind: "error",
              message: "解析履歴を削除できませんでした。",
            },
          }));
        }
        return false;
      }
    },
    [history, report.ruleset_version],
  );

  const clearHistory = useCallback(async (): Promise<boolean> => {
    const currentGeneration = generation.current;
    setState((current) => ({ ...current, busy: "clear", notice: null }));
    try {
      await history.clear();
      const snapshot = await loadMatchupHistory(
        report.ruleset_version,
        history,
      );
      if (mounted.current && generation.current === currentGeneration) {
        setState({
          phase: "ready",
          ...snapshot,
          busy: null,
          notice: {
            kind: "status",
            message: "このブラウザの解析履歴をすべて削除しました。",
          },
        });
      }
      return true;
    } catch {
      if (mounted.current && generation.current === currentGeneration) {
        setState((current) => ({
          ...current,
          busy: null,
          notice: {
            kind: "error",
            message: "解析履歴をすべて削除できませんでした。",
          },
        }));
      }
      return false;
    }
  }, [history, report.ruleset_version]);

  return {
    ...state,
    setSavingEnabled,
    deleteRecord,
    clearHistory,
  };
}
