import type { TransactionLock } from "../../../models/common";
import type { DbWriteCtx } from "../../common/capability";
import type { TransactionLockRepository as ITransactionLockRepository } from "../repository";
import { acquire } from "./acquire";

export class TransactionLockRepository implements ITransactionLockRepository {
  acquire(ctx: DbWriteCtx, lock: TransactionLock): Promise<void> {
    return acquire(ctx.db, lock);
  }
}
