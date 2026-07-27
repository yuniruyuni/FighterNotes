import { fail } from "../../models/common/fail";
import type {
  DeletePassword,
  DeletePasswordHash,
  ShareId,
} from "../../models/published-analysis";
import {
  PublishedAnalysisLifecycle,
  parseDeletePassword,
  parseShareId,
} from "../../models/published-analysis";
import type { Usecase } from "../runner";
import { usecase } from "../runner";

interface DeleteRequest {
  id: ShareId;
  deletePassword: DeletePassword;
}

interface StoredDeleteRequest extends DeleteRequest {
  deletePasswordHash: DeletePasswordHash;
}

export function deletePublishedAnalysisUsecase(
  rawId: string,
  rawDeletePassword: string,
): Usecase<{ deleted: true }> {
  return usecase<
    DeleteRequest,
    StoredDeleteRequest,
    ShareId,
    ShareId,
    ShareId,
    { deleted: true },
    { deleted: true }
  >({
    pre: () => {
      const id = parseShareId(rawId);
      const deletePassword = parseDeletePassword(rawDeletePassword);
      if (!id || !deletePassword) {
        return fail("NOT_FOUND", "Published analysis not found");
      }
      return { id, deletePassword };
    },
    read: async (ctx, request) => {
      const page = await ctx.repos.publishedAnalysisLifecycle.list(
        PublishedAnalysisLifecycle.ById(request.id),
        { limit: 1, sort: PublishedAnalysisLifecycle.defaultSort },
      );
      const stored = page.items[0];
      if (!stored?.deletePasswordHash) {
        return fail("NOT_FOUND", "Published analysis not found");
      }
      return {
        ...request,
        deletePasswordHash: stored.deletePasswordHash,
      };
    },
    process: async (ctx, request) => {
      if (
        !(await ctx.services.publishedAnalysisSecurity.verifyDeletePassword(
          request.deletePassword,
          request.deletePasswordHash,
        ))
      ) {
        return fail("NOT_FOUND", "Published analysis not found");
      }
      return request.id;
    },
    finish: async (ctx, id) => {
      const deleted = await ctx.repos.publishedAnalysisLifecycle.delete(
        PublishedAnalysisLifecycle.ById(id),
      );
      if (deleted !== 1) {
        return fail("NOT_FOUND", "Published analysis not found");
      }
      ctx.logger.info("Published analysis deleted");
      return { deleted: true } as const;
    },
    result: (result) => result,
  });
}
