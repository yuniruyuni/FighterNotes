import type { PublishedAnalysisCreateEvent } from "../../../models/published-analysis";
import type { DbReadCtx, DbWriteCtx } from "../../common/capability";
import type { PublishedAnalysisCreateEventRepository as IPublishedAnalysisCreateEventRepository } from "../repository";
import { count } from "./count";
import { create } from "./create";
import { del } from "./delete";
import { get } from "./get";

export class PublishedAnalysisCreateEventRepository
  implements IPublishedAnalysisCreateEventRepository
{
  get(
    ctx: DbReadCtx,
    spec: PublishedAnalysisCreateEvent.Spec,
  ): Promise<PublishedAnalysisCreateEvent | null> {
    return get(ctx.db, spec);
  }

  count(
    ctx: DbReadCtx,
    spec: PublishedAnalysisCreateEvent.Spec,
  ): Promise<number> {
    return count(ctx.db, spec);
  }

  create(ctx: DbWriteCtx, event: PublishedAnalysisCreateEvent): Promise<void> {
    return create(ctx.db, event);
  }

  delete(
    ctx: DbWriteCtx,
    spec: PublishedAnalysisCreateEvent.Spec,
  ): Promise<number> {
    return del(ctx.db, spec);
  }
}
