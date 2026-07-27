import type { PublishedAnalysisCreateEvent } from "../../models/published-analysis";
import type { DbReadCtx, DbWriteCtx } from "../common/capability";

export interface PublishedAnalysisCreateEventRepository {
  get(
    ctx: DbReadCtx,
    spec: PublishedAnalysisCreateEvent.Spec,
  ): Promise<PublishedAnalysisCreateEvent | null>;
  count(
    ctx: DbReadCtx,
    spec: PublishedAnalysisCreateEvent.Spec,
  ): Promise<number>;
  create(ctx: DbWriteCtx, event: PublishedAnalysisCreateEvent): Promise<void>;
  delete(
    ctx: DbWriteCtx,
    spec: PublishedAnalysisCreateEvent.Spec,
  ): Promise<number>;
}
