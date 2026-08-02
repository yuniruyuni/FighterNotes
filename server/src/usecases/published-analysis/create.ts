import { fail } from "../../models/common/fail";
import type {
  CreatedPublishedAnalysis,
  DeletePassword,
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
import type { ReadContext, WriteContext } from "../context";
import type { Usecase } from "../runner";
import { usecase } from "../runner";
import { SecurityCapacityError } from "../services";

export interface CreatePublishedAnalysisResult {
  id: ShareId;
  expiresAt: Date;
}

interface CreateRequest {
  id: ShareId;
  content: PublishedAnalysisContent;
  deletePasswordHash: DeletePasswordHash;
}

interface ValidatedCreateRequest {
  id: ShareId;
  content: PublishedAnalysisContent;
  deletePassword: DeletePassword;
}

export function createPublishedAnalysisUsecase(
  candidate: unknown,
  rawDeletePassword: unknown,
  retentionDays: number,
  limits: ShareCreateLimits,
): Usecase<CreatePublishedAnalysisResult> {
  return usecase<
    ValidatedCreateRequest,
    ValidatedCreateRequest,
    CreateRequest,
    CreateRequest,
    CreateRequest,
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
        deletePassword,
      };
    },
    read: async (ctx, request) => {
      const quotaFailure = await checkCreateQuota(ctx, limits);
      return quotaFailure ?? request;
    },
    process: async (ctx, request) => {
      try {
        return {
          id: request.id,
          content: request.content,
          deletePasswordHash:
            await ctx.services.publishedAnalysisSecurity.hashDeletePassword(
              request.deletePassword,
            ),
        };
      } catch (error) {
        if (!(error instanceof SecurityCapacityError)) throw error;
        ctx.logger.warn("Published analysis security capacity reached", {
          operation: "hash",
        });
        return fail("RESOURCE_LIMIT", "Published analysis service is busy");
      }
    },
    finish: async (ctx, request) => {
      await ctx.repos.transactionLock.acquire(PUBLISHED_ANALYSIS_CREATE_LOCK);
      const quotaFailure = await checkCreateQuota(ctx, limits);
      if (quotaFailure) return quotaFailure;

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

async function checkCreateQuota(
  ctx: ReadContext | WriteContext,
  limits: ShareCreateLimits,
) {
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
  if (quota.allowed) return null;
  ctx.logger.warn("Published analysis create quota reached", {
    reason: quota.reason,
  });
  return fail("RESOURCE_LIMIT", "Published analysis capacity reached");
}
