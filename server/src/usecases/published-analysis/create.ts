import { fail } from "../../models/common/fail";
import type {
  CreatedPublishedAnalysis,
  DeletePasswordHash,
  PublishedAnalysisContent,
  ShareCreateLimits,
  ShareId,
} from "../../models/published-analysis";
import {
  createPersistablePublishedAnalysis,
  createPublishedAnalysisContent,
  evaluatePublishedAnalysisCreateQuota,
  PUBLISHED_ANALYSIS_CREATE_LOCK,
  PublishedAnalysisCreateEvent,
  PublishedAnalysisLifecycle,
  PublishedAnalysisStorageUsage,
  parseDeletePassword,
  startOfUtcDay,
} from "../../models/published-analysis";
import type { Usecase } from "../runner";
import { usecase } from "../runner";

export interface CreatePublishedAnalysisResult {
  id: ShareId;
  expiresAt: Date;
}

interface CreateRequest {
  id: ShareId;
  content: PublishedAnalysisContent;
  deletePasswordHash: DeletePasswordHash;
}

export function createPublishedAnalysisUsecase(
  candidate: unknown,
  rawDeletePassword: unknown,
  retentionDays: number,
  limits: ShareCreateLimits,
): Usecase<CreatePublishedAnalysisResult> {
  return usecase<
    CreateRequest,
    CreateRequest,
    CreateRequest,
    CreatedPublishedAnalysis,
    CreatedPublishedAnalysis,
    CreatedPublishedAnalysis,
    CreatePublishedAnalysisResult
  >({
    pre: async (ctx) => {
      const content = createPublishedAnalysisContent(candidate);
      if (!content.ok) return content.error;
      const deletePassword =
        typeof rawDeletePassword === "string"
          ? parseDeletePassword(rawDeletePassword)
          : null;
      if (!deletePassword) {
        return fail("INVALID_INPUT", "Invalid deletion password");
      }
      return {
        id: ctx.services.publishedAnalysisSecurity.generateShareId(),
        content: content.value,
        deletePasswordHash:
          await ctx.services.publishedAnalysisSecurity.hashDeletePassword(
            deletePassword,
          ),
      };
    },
    write: async (ctx, request) => {
      await ctx.repos.transactionLock.acquire(PUBLISHED_ANALYSIS_CREATE_LOCK);

      const dailyCreates = await ctx.repos.publishedAnalysisCreateEvent.count(
        PublishedAnalysisCreateEvent.CreatedAtOrAfter(startOfUtcDay(ctx.now)),
      );
      const activeRows = await ctx.repos.publishedAnalysisLifecycle.count(
        PublishedAnalysisLifecycle.ActiveAt(ctx.now),
      );
      const storage = await ctx.repos.publishedAnalysisStorageUsage.get(
        PublishedAnalysisStorageUsage.Current(),
      );
      if (!storage) {
        throw new Error("Published analysis storage usage returned no row");
      }

      const quota = evaluatePublishedAnalysisCreateQuota(
        { dailyCreates, activeRows, storageBytes: storage.bytes },
        limits,
      );
      if (!quota.allowed) {
        ctx.logger.warn("Published analysis create quota reached", {
          reason: quota.reason,
        });
        return fail("RESOURCE_LIMIT", "Published analysis capacity reached");
      }

      const created = createPersistablePublishedAnalysis({
        id: request.id,
        content: request.content,
        deletePasswordHash: request.deletePasswordHash,
        now: ctx.now,
        retentionDays,
      });

      await ctx.repos.publishedAnalysisCreateEvent.create({
        analysisId: created.analysis.id,
        createdAt: created.analysis.createdAt,
      });
      await ctx.repos.publishedAnalysis.create(created.analysis);
      ctx.logger.info("Published analysis created");
      return created;
    },
    result: (created) => ({
      id: created.analysis.id,
      expiresAt: created.analysis.expiresAt,
    }),
  });
}
