import type { AnalysisContext, AnalysisSide } from "./context.js";
import type { AdviceReport } from "./report.js";
import type { AnalysisResult } from "./result.js";

export type AnalysisPhase =
  | "setup"
  | "analyzing"
  | "canceling"
  | "canceled"
  | "ready";

export interface AnalysisSessionState {
  file: File | null;
  side: AnalysisSide | "";
  ownCharacter: string;
  opponentCharacter: string;
  phase: AnalysisPhase;
  progress: number;
  status: string;
  error: string;
  result: AnalysisResult | null;
  report: AdviceReport | null;
  context: AnalysisContext | null;
}

export interface CompletedAnalysis {
  file: File;
  result: AnalysisResult;
  report: AdviceReport;
  context: AnalysisContext;
}

export type AnalysisSessionAction =
  | { type: "file"; file: File | null }
  | { type: "side"; side: AnalysisSide }
  | { type: "ownCharacter"; character: string }
  | { type: "opponentCharacter"; character: string }
  | { type: "start" }
  | { type: "progress"; progress: number; status: string }
  | { type: "cancel" }
  | { type: "canceled" }
  | {
      type: "complete";
      result: AnalysisResult;
      report: AdviceReport;
      context: AnalysisContext;
    }
  | { type: "fail"; error: string }
  | { type: "reset" };

export const AnalysisSession = {
  initial(): AnalysisSessionState {
    return {
      file: null,
      side: "",
      ownCharacter: "",
      opponentCharacter: "",
      phase: "setup",
      progress: 0,
      status: "",
      error: "",
      result: null,
      report: null,
      context: null,
    };
  },

  canStart(state: AnalysisSessionState): boolean {
    return Boolean(
      state.phase !== "analyzing" &&
        state.phase !== "canceling" &&
        state.file &&
        state.side &&
        state.ownCharacter &&
        state.opponentCharacter,
    );
  },

  reduce(
    state: AnalysisSessionState,
    action: AnalysisSessionAction,
  ): AnalysisSessionState {
    switch (action.type) {
      case "file":
        return { ...state, file: action.file, side: "", error: "" };
      case "side":
        return { ...state, side: action.side };
      case "ownCharacter":
        return { ...state, ownCharacter: action.character, error: "" };
      case "opponentCharacter":
        return { ...state, opponentCharacter: action.character, error: "" };
      case "start":
        return {
          ...state,
          phase: "analyzing",
          progress: 0,
          status: "準備中…",
          error: "",
          result: null,
          report: null,
          context: null,
        };
      case "progress":
        if (state.phase !== "analyzing") return state;
        return {
          ...state,
          progress: Math.round(action.progress * 100),
          status: action.status,
        };
      case "cancel":
        if (state.phase !== "analyzing") return state;
        return {
          ...state,
          phase: "canceling",
          status: "解析を中止しています…",
          error: "",
        };
      case "canceled":
        if (state.phase !== "canceling" && state.phase !== "analyzing") {
          return state;
        }
        return {
          ...state,
          phase: "canceled",
          progress: 0,
          status: "解析を中止しました。設定を確認して再試行できます。",
          error: "",
          result: null,
          report: null,
          context: null,
        };
      case "complete":
        if (state.phase !== "analyzing") return state;
        return {
          ...state,
          phase: "ready",
          progress: 100,
          status: "",
          error: "",
          result: action.result,
          report: action.report,
          context: action.context,
        };
      case "fail":
        return {
          ...state,
          phase: "setup",
          status: "",
          error: action.error,
        };
      case "reset":
        return {
          ...AnalysisSession.initial(),
          file: state.file,
          ownCharacter: state.ownCharacter,
          opponentCharacter: state.opponentCharacter,
        };
    }
  },
};
