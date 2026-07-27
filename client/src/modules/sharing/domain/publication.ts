import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import type { PublishedAnalysisShare } from "./share.js";

export type PublicationTone = "pending" | "success" | "error" | "";
export type PublicationPhase =
  | "idle"
  | "creating"
  | "failed"
  | "deleting"
  | "deleted";

export interface PublicationSource {
  context: AnalysisContext;
  deleteCode: string;
  report: AdviceReport;
}

export interface PublicationState {
  source?: PublicationSource;
  published?: PublishedAnalysisShare;
  phase: PublicationPhase;
  storedLocally: boolean;
  status: string;
  tone: PublicationTone;
}

export type PublicationAction =
  | { type: "prepare"; source: PublicationSource }
  | { type: "creating"; source: PublicationSource }
  | {
      type: "created";
      published: PublishedAnalysisShare;
      storedLocally: boolean;
    }
  | { type: "failed"; message: string }
  | { type: "deleting" }
  | { type: "deleteFailed" }
  | { type: "deleted"; removedLocally: boolean }
  | { type: "feedback"; message: string; tone: PublicationTone }
  | { type: "reset" };

export const Publication = {
  initial(): PublicationState {
    return {
      phase: "idle",
      storedLocally: false,
      status: "",
      tone: "",
    };
  },

  canRetry(state: PublicationState): state is PublicationState & {
    source: PublicationSource;
  } {
    return Boolean(
      state.source &&
        !state.published &&
        state.phase !== "creating" &&
        state.phase !== "deleting",
    );
  },

  canDelete(state: PublicationState): state is PublicationState & {
    source: PublicationSource;
    published: PublishedAnalysisShare;
  } {
    return Boolean(state.source && state.published && state.phase === "idle");
  },

  reduce(state: PublicationState, action: PublicationAction): PublicationState {
    switch (action.type) {
      case "prepare":
        return {
          source: action.source,
          phase: "idle",
          storedLocally: false,
          status: "共有URLを準備しています。",
          tone: "pending",
        };
      case "creating":
        return {
          ...state,
          source: action.source,
          published: undefined,
          phase: "creating",
          storedLocally: false,
          status: "共有URLを準備しています。",
          tone: "pending",
        };
      case "created":
        return {
          ...state,
          published: action.published,
          phase: "idle",
          storedLocally: action.storedLocally,
          status:
            "公開URLを準備しました。この端末では動画付きの解析画面を表示しています。",
          tone: "success",
        };
      case "failed":
        return {
          ...state,
          published: undefined,
          phase: "failed",
          status: action.message,
          tone: "error",
        };
      case "deleting":
        return {
          ...state,
          phase: "deleting",
          status: "共有結果を削除しています。",
          tone: "pending",
        };
      case "deleteFailed":
        return {
          ...state,
          phase: "idle",
          status: "共有結果を削除できませんでした。",
          tone: "error",
        };
      case "deleted":
        return {
          ...state,
          published: undefined,
          phase: "deleted",
          storedLocally: false,
          status: action.removedLocally
            ? "共有結果を削除しました。新しいアクセスには約15秒以内に反映されます。"
            : "共有結果を削除しました。この端末の管理一覧に表示が残る場合があります。",
          tone: action.removedLocally ? "success" : "error",
        };
      case "feedback":
        return { ...state, status: action.message, tone: action.tone };
      case "reset":
        return Publication.initial();
    }
  },
};
