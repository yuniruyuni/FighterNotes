import type { TransactionLock } from "../../models/common";
import type { DbWriteCtx } from "../common/capability";

// Transaction locks are an infrastructure command rather than persisted CRUD.
// Keeping this repository generic prevents domain repositories from hiding
// orchestration and quota policy inside ad-hoc methods.
export interface TransactionLockRepository {
  acquire(ctx: DbWriteCtx, lock: TransactionLock): Promise<void>;
}
