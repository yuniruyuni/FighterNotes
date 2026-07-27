import type { Cursor, Page } from "../../../models/common";
import type { PublishedAnalysisLifecycle } from "../../../models/published-analysis";
import type { DbReadCtx, DbWriteCtx } from "../../common/capability";
import type { PublishedAnalysisLifecycleRepository as IPublishedAnalysisLifecycleRepository } from "../repository";
import { count } from "./count";
import { del } from "./delete";
import { list } from "./list";

export class PublishedAnalysisLifecycleRepository
  implements IPublishedAnalysisLifecycleRepository
{
  list(
    ctx: DbReadCtx,
    spec: PublishedAnalysisLifecycle.Spec,
    cursor: Cursor<PublishedAnalysisLifecycle.SortKey>,
  ): Promise<Page<PublishedAnalysisLifecycle>> {
    return list(ctx.db, spec, cursor);
  }

  count(
    ctx: DbReadCtx,
    spec: PublishedAnalysisLifecycle.Spec,
  ): Promise<number> {
    return count(ctx.db, spec);
  }

  delete(
    ctx: DbWriteCtx,
    spec: PublishedAnalysisLifecycle.Spec,
  ): Promise<number> {
    return del(ctx.db, spec);
  }
}
