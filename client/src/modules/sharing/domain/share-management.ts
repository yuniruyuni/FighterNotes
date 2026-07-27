import type { ManagedShareSnapshot } from "./managed-share.js";
import { isShareId, isValidDeletePassword } from "./share.js";

export type FeedbackTone = "pending" | "success" | "error" | "";

export interface Feedback {
  message: string;
  tone: FeedbackTone;
}

export type ManualDeletionRequest =
  | { valid: true; id: string; credential: string }
  | { valid: false; feedback: Feedback };

export const ShareManagement = {
  emptyFeedback(): Feedback {
    return { message: "", tone: "" };
  },

  snapshotFeedback(snapshot: ManagedShareSnapshot): Feedback {
    if (!snapshot.available) {
      return {
        message:
          "このブラウザの保存領域を読み込めません。下のフォームから削除コードを入力してください。",
        tone: "error",
      };
    }
    if (snapshot.shares.length === 0) {
      return {
        message: "このブラウザに保存された共有はありません。",
        tone: "",
      };
    }
    return { message: "", tone: "" };
  },

  manualDeletionRequest(
    reference: string,
    credential: string,
    expectedOrigin: string,
  ): ManualDeletionRequest {
    const id = shareIdFromReference(reference, expectedOrigin);
    if (!id) {
      return {
        valid: false,
        feedback: {
          message: "このサイトの共有URLまたは共有IDを入力してください。",
          tone: "error",
        },
      };
    }
    if (!isValidDeletePassword(credential)) {
      return {
        valid: false,
        feedback: {
          message:
            "削除コードまたは以前設定した削除用パスワードを12文字以上128文字以下で入力してください。",
          tone: "error",
        },
      };
    }
    return { valid: true, id, credential };
  },
};

export function isShareManagementPath(pathname: string): boolean {
  return pathname === "/manage" || pathname.startsWith("/manage/");
}

export function managementPathId(pathname: string): string | undefined {
  const match = /^\/manage\/([^/]+)$/.exec(pathname);
  return match?.[1] && isShareId(match[1]) ? match[1] : undefined;
}

export function shareIdFromReference(
  value: string,
  expectedOrigin: string,
): string | undefined {
  const trimmed = value.trim();
  if (isShareId(trimmed)) return trimmed;

  if (!URL.canParse(trimmed)) return undefined;
  const url = new URL(trimmed);
  if (
    url.origin !== expectedOrigin ||
    url.username ||
    url.password ||
    url.search ||
    url.hash
  ) {
    return undefined;
  }
  const match = /^\/s\/([^/]+)$/.exec(url.pathname);
  return match?.[1] && isShareId(match[1]) ? match[1] : undefined;
}
