import type { Page } from "../../models/common";
import {
  PublishedAnalysisCreateEvent,
  PublishedAnalysisLifecycle,
} from "../../models/published-analysis";
import type { Usecase } from "../runner";
import { usecase } from "../runner";

export interface CleanupPublishedAnalysisBatchInput {
  readonly limit: number;
  readonly retentionCutoff: Date;
}

export interface CleanupPublishedAnalysisBatchResult {
  readonly deleted: number;
  readonly hasMore: boolean;
}

export function cleanupPublishedAnalysisBatchUsecase(
  input: CleanupPublishedAnalysisBatchInput,
): Usecase<CleanupPublishedAnalysisBatchResult> {
  return usecase<
    CleanupPublishedAnalysisBatchInput,
    Page<PublishedAnalysisLifecycle>,
    Page<PublishedAnalysisLifecycle>,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult,
    CleanupPublishedAnalysisBatchResult
  >({
    pre: () => input,
    read: (ctx, value) =>
      ctx.repos.publishedAnalysisLifecycle.list(
        PublishedAnalysisLifecycle.ExpiredAt(ctx.now).or(
          PublishedAnalysisLifecycle.CreatedAtOrBefore(value.retentionCutoff),
        ),
        {
          limit: value.limit,
          sort: PublishedAnalysisLifecycle.defaultSort,
        },
      ),
    write: async (ctx, page) => {
      const deleted = await ctx.repos.publishedAnalysisLifecycle.delete(
        PublishedAnalysisLifecycle.ByIds(...page.items.map((item) => item.id)),
      );
      return { deleted, hasMore: page.hasMore };
    },
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
