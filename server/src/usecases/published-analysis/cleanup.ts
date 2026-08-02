import {
  PublishedAnalysisCreateEvent,
  PublishedAnalysisLifecycle,
} from "../../models/published-analysis";
import type { Usecase } from "../runner";
import { usecase } from "../runner";
import type { RateLimitPruneResult } from "../services";

export interface CleanupPublishedAnalysisBatchInput {
  readonly limit: number;
  readonly retentionCutoff: Date;
}

export interface CleanupPublishedAnalysisBatchResult {
  readonly deleted: number;
  readonly hasMore: boolean;
}

export interface PruneSharingRateLimitsBatchInput {
  readonly before: Date;
  readonly limit: number;
}

export function cleanupPublishedAnalysisBatchUsecase(
  input: CleanupPublishedAnalysisBatchInput,
): Usecase<CleanupPublishedAnalysisBatchResult> {
  return usecase<
    CleanupPublishedAnalysisBatchInput,
    CleanupPublishedAnalysisBatchInput,
    CleanupPublishedAnalysisBatchInput,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult
  >({
    pre: () => input,
    write: (ctx, value) =>
      ctx.repos.publishedAnalysisLifecycle.deleteBatch(
        PublishedAnalysisLifecycle.ExpiredAt(ctx.now).or(
          PublishedAnalysisLifecycle.CreatedAtOrBefore(value.retentionCutoff),
        ),
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
