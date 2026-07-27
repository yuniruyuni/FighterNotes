import { fail } from "../../models/common/fail";
import type {
  PublishedAnalysis,
  ShareId,
} from "../../models/published-analysis";
import {
  PublishedAnalysis as PublishedAnalysisModel,
  parseShareId,
} from "../../models/published-analysis";
import type { Usecase } from "../runner";
import { usecase } from "../runner";

export function getPublishedAnalysisUsecase(
  rawId: string,
): Usecase<PublishedAnalysis> {
  return usecase<
    ShareId,
    PublishedAnalysis,
    PublishedAnalysis,
    PublishedAnalysis,
    PublishedAnalysis,
    PublishedAnalysis,
    PublishedAnalysis
  >({
    pre: () =>
      parseShareId(rawId) ?? fail("NOT_FOUND", "Published analysis not found"),
    read: async (ctx, id) =>
      (await ctx.repos.publishedAnalysis.get(
        PublishedAnalysisModel.ById(id).and(
          PublishedAnalysisModel.ActiveAt(ctx.now),
        ),
      )) ?? fail("NOT_FOUND", "Published analysis not found"),
    result: (analysis) => analysis,
  });
}
