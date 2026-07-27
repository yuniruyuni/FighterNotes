import type {
  PersistablePublishedAnalysis,
  PublishedAnalysis,
} from "../../../models/published-analysis";
import type { DbReadCtx, DbWriteCtx } from "../../common/capability";
import type { PublishedAnalysisRepository as IPublishedAnalysisRepository } from "../repository";
import { create } from "./create";
import { get } from "./get";

export class PublishedAnalysisRepository
  implements IPublishedAnalysisRepository
{
  get(
    ctx: DbReadCtx,
    spec: PublishedAnalysis.Spec,
  ): Promise<PublishedAnalysis | null> {
    return get(ctx.db, spec);
  }

  create(
    ctx: DbWriteCtx,
    analysis: PersistablePublishedAnalysis,
  ): Promise<void> {
    return create(ctx.db, analysis);
  }
}
