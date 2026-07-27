import type { PublishedAnalysisStorageUsage } from "../../../models/published-analysis";
import type { DbReadCtx } from "../../common/capability";
import type { PublishedAnalysisStorageUsageRepository as IPublishedAnalysisStorageUsageRepository } from "../repository";
import { get } from "./get";

export class PublishedAnalysisStorageUsageRepository
  implements IPublishedAnalysisStorageUsageRepository
{
  get(
    ctx: DbReadCtx,
    spec: PublishedAnalysisStorageUsage.Spec,
  ): Promise<PublishedAnalysisStorageUsage | null> {
    return get(ctx.db, spec);
  }
}
