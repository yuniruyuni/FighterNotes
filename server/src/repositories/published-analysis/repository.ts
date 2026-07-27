import type {
  PersistablePublishedAnalysis,
  PublishedAnalysis,
} from "../../models/published-analysis";
import type { DbReadCtx, DbWriteCtx } from "../common/capability";

export interface PublishedAnalysisRepository {
  get(
    ctx: DbReadCtx,
    spec: PublishedAnalysis.Spec,
  ): Promise<PublishedAnalysis | null>;
  create(
    ctx: DbWriteCtx,
    analysis: PersistablePublishedAnalysis,
  ): Promise<void>;
}
