import type { PublishedAnalysisRepository } from "./published-analysis";
import { PublishedAnalysisRepository as PgPublishedAnalysisRepository } from "./published-analysis/postgres";
import type { PublishedAnalysisCreateEventRepository } from "./published-analysis-create-event";
import { PublishedAnalysisCreateEventRepository as PgPublishedAnalysisCreateEventRepository } from "./published-analysis-create-event/postgres";
import type { PublishedAnalysisLifecycleRepository } from "./published-analysis-lifecycle";
import { PublishedAnalysisLifecycleRepository as PgPublishedAnalysisLifecycleRepository } from "./published-analysis-lifecycle/postgres";
import type { PublishedAnalysisStorageUsageRepository } from "./published-analysis-storage-usage";
import { PublishedAnalysisStorageUsageRepository as PgPublishedAnalysisStorageUsageRepository } from "./published-analysis-storage-usage/postgres";
import type { TransactionLockRepository } from "./transaction-lock";
import { TransactionLockRepository as PgTransactionLockRepository } from "./transaction-lock/postgres";

export type Repos = {
  publishedAnalysis: PublishedAnalysisRepository;
  publishedAnalysisLifecycle: PublishedAnalysisLifecycleRepository;
  publishedAnalysisCreateEvent: PublishedAnalysisCreateEventRepository;
  publishedAnalysisStorageUsage: PublishedAnalysisStorageUsageRepository;
  transactionLock: TransactionLockRepository;
};

export function createRawRepos(): Repos {
  return {
    publishedAnalysis: new PgPublishedAnalysisRepository(),
    publishedAnalysisLifecycle: new PgPublishedAnalysisLifecycleRepository(),
    publishedAnalysisCreateEvent:
      new PgPublishedAnalysisCreateEventRepository(),
    publishedAnalysisStorageUsage:
      new PgPublishedAnalysisStorageUsageRepository(),
    transactionLock: new PgTransactionLockRepository(),
  };
}
