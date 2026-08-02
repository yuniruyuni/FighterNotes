import type { Cursor, Page } from "../../models/common";
import type { PublishedAnalysisLifecycle } from "../../models/published-analysis";
import type { DbReadCtx, DbWriteCtx } from "../common/capability";

export interface LifecycleDeleteBatchResult {
  readonly deleted: number;
  readonly hasMore: boolean;
}

export interface PublishedAnalysisLifecycleRepository {
  list(
    ctx: DbReadCtx,
    spec: PublishedAnalysisLifecycle.Spec,
    cursor: Cursor<PublishedAnalysisLifecycle.SortKey>,
  ): Promise<Page<PublishedAnalysisLifecycle>>;
  count(ctx: DbReadCtx, spec: PublishedAnalysisLifecycle.Spec): Promise<number>;
  delete(
    ctx: DbWriteCtx,
    spec: PublishedAnalysisLifecycle.Spec,
  ): Promise<number>;
  deleteBatch(
    ctx: DbWriteCtx,
    spec: PublishedAnalysisLifecycle.Spec,
    limit: number,
  ): Promise<LifecycleDeleteBatchResult>;
}
