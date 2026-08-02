import { PublishedAnalysisCreateEvent } from "../../models/published-analysis";
import type { Usecase } from "../runner";
import { usecase } from "../runner";
import type { RateLimitPruneResult } from "../services";

export interface CleanupExpiredPublishedAnalysisBatchInput {
  readonly at: Date;
  readonly limit: number;
}

export interface CleanupRetainedPublishedAnalysisBatchInput {
  readonly retentionCutoff: Date;
  readonly limit: number;
}

export interface CleanupPublishedAnalysisBatchResult {
  readonly deleted: number;
  readonly hasMore: boolean;
}

export interface PruneSharingRateLimitsBatchInput {
  readonly before: Date;
  readonly limit: number;
}

export function cleanupExpiredPublishedAnalysisBatchUsecase(
  input: CleanupExpiredPublishedAnalysisBatchInput,
): Usecase<CleanupPublishedAnalysisBatchResult> {
  return usecase<
    CleanupExpiredPublishedAnalysisBatchInput,
    CleanupExpiredPublishedAnalysisBatchInput,
    CleanupExpiredPublishedAnalysisBatchInput,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult
  >({
    pre: () => input,
    write: (ctx, value) =>
      ctx.repos.publishedAnalysisLifecycle.deleteExpiredBatch(
        value.at,
        value.limit,
      ),
  });
}

export function cleanupRetainedPublishedAnalysisBatchUsecase(
  input: CleanupRetainedPublishedAnalysisBatchInput,
): Usecase<CleanupPublishedAnalysisBatchResult> {
  return usecase<
    CleanupRetainedPublishedAnalysisBatchInput,
    CleanupRetainedPublishedAnalysisBatchInput,
    CleanupRetainedPublishedAnalysisBatchInput,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult
  >({
    pre: () => input,
    write: (ctx, value) =>
      ctx.repos.publishedAnalysisLifecycle.deleteCreatedAtOrBeforeBatch(
        value.retentionCutoff,
        value.limit,
      ),
  });
}

export function pruneSharingRateLimitsBatchUsecase(
  input: PruneSharingRateLimitsBatchInput,
): Usecase<RateLimitPruneResult> {
  return usecase<
    PruneSharingRateLimitsBatchInput,
    PruneSharingRateLimitsBatchInput,
    RateLimitPruneResult,
    RateLimitPruneResult,
    RateLimitPruneResult,
    RateLimitPruneResult,
    RateLimitPruneResult
  >({
    pre: () => input,
    process: (ctx, value) =>
      ctx.services.sharingRateLimit.prune(value.before, value.limit),
  });
}

export function prunePublishedAnalysisCreateEventsUsecase(
  cutoff: Date,
): Usecase<number> {
  return usecase<Date, Date, Date, number, number, number, number>({
    pre: () => cutoff,
    write: (ctx, value) =>
      ctx.repos.publishedAnalysisCreateEvent.delete(
        PublishedAnalysisCreateEvent.CreatedBefore(value),
      ),
  });
}
