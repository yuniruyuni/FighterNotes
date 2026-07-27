import { createTRPCUntypedClient, httpLink } from "@trpc/client";
import type { PublishedAnalysisGateway } from "../application/ports.js";
import type { PublishedAnalysisCandidate } from "../domain/published-analysis.js";
import {
  assertDeletePassword,
  isShareId,
  isValidDeletePassword,
  MAX_DELETE_PASSWORD_LENGTH,
  MIN_DELETE_PASSWORD_LENGTH,
  type PublishedAnalysisShare,
  shareIdFromUrl,
} from "../domain/share.js";

export {
  isValidDeletePassword,
  MAX_DELETE_PASSWORD_LENGTH,
  MIN_DELETE_PASSWORD_LENGTH,
  type PublishedAnalysisShare,
};

export interface ShareTransport {
  mutation(path: string, input: unknown): Promise<unknown>;
}

let defaultTransport: ShareTransport | undefined;

export async function createPublishedAnalysisShare(
  candidate: PublishedAnalysisCandidate,
  deletePassword: string,
  transport = getDefaultTransport(),
): Promise<PublishedAnalysisShare> {
  assertDeletePassword(deletePassword);
  return parseCreateResponse(
    await transport.mutation("publishedAnalysis.create", {
      analysis: candidate,
      deletePassword,
    }),
  );
}

export async function deletePublishedAnalysisShare(
  share: Pick<PublishedAnalysisShare, "id">,
  deletePassword: string,
  transport = getDefaultTransport(),
): Promise<void> {
  if (!isShareId(share.id)) {
    throw new Error("共有IDが不正です。");
  }
  assertDeletePassword(deletePassword);
  const response = await transport.mutation("publishedAnalysis.delete", {
    id: share.id,
    deletePassword,
  });
  if (!isRecord(response) || response.deleted !== true) {
    throw new Error("共有結果の削除応答が不正です。");
  }
}

export function parseCreateResponse(value: unknown): PublishedAnalysisShare {
  if (
    !isRecord(value) ||
    typeof value.url !== "string" ||
    typeof value.expiresAt !== "string"
  ) {
    throw new Error("共有結果の作成応答が不正です。");
  }

  let url: URL;
  try {
    url = new URL(value.url);
  } catch {
    throw new Error("共有URLが不正です。");
  }
  const id = shareIdFromUrl(url);
  if (!id) {
    throw new Error("共有URLが不正です。");
  }
  const expiresAt = new Date(value.expiresAt);
  if (!Number.isFinite(expiresAt.getTime())) {
    throw new Error("共有期限が不正です。");
  }

  return {
    id,
    url: url.toString(),
    expiresAt: expiresAt.toISOString(),
  };
}

export function createShareErrorMessage(error: unknown): string {
  const code =
    isRecord(error) && isRecord(error.data) ? error.data.code : undefined;
  if (code === "BAD_REQUEST") {
    return "この分析結果は現在の共有形式に対応していません。ページを再読み込みして再解析してください。";
  }
  if (code === "TOO_MANY_REQUESTS") {
    return "共有リンクの作成回数が上限に達しました。1分ほど待って再度お試しください。";
  }
  return "共有リンクを作成できませんでした。時間を置いて再度お試しください。";
}

export const browserPublishedAnalysisGateway: PublishedAnalysisGateway = {
  create: createPublishedAnalysisShare,
  delete: deletePublishedAnalysisShare,
  errorMessage: createShareErrorMessage,
};

function getDefaultTransport(): ShareTransport {
  defaultTransport ??= createTRPCUntypedClient({
    links: [httpLink({ url: "/api/trpc" })],
  });
  return defaultTransport;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
