import type { PublishedAnalysisStorageUsage } from "../../models/published-analysis";
import type { DbReadCtx } from "../common/capability";

export interface PublishedAnalysisStorageUsageRepository {
  get(
    ctx: DbReadCtx,
    spec: PublishedAnalysisStorageUsage.Spec,
  ): Promise<PublishedAnalysisStorageUsage | null>;
}
