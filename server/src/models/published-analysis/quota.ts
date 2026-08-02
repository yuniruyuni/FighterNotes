import type { TransactionLock } from "../common";

export interface ShareCreateLimits {
  readonly dailyCreates: number;
  readonly activeRows: number;
  readonly storageBytes: number;
}

export interface PublishedAnalysisQuotaUsage {
  readonly dailyCreates: number;
  readonly activeRows: number;
  readonly storageBytes: number;
}

export type ShareQuotaDecision =
  | { readonly allowed: true }
  | {
      readonly allowed: false;
      readonly reason: "daily" | "active" | "storage";
    };

// Quota accounting stores the exact serialized logical size for new rows and
// conservatively accounts legacy rows at this closed-schema maximum.
export const MAX_ANALYSIS_STORAGE_RESERVATION_BYTES = 8 * 1024;

export const PUBLISHED_ANALYSIS_CREATE_LOCK: TransactionLock = Object.freeze({
  namespace: 1_179_537_442,
  id: 1,
});

export function evaluatePublishedAnalysisCreateQuota(
  usage: PublishedAnalysisQuotaUsage,
  limits: ShareCreateLimits,
): ShareQuotaDecision {
  if (usage.dailyCreates >= limits.dailyCreates) {
    return { allowed: false, reason: "daily" };
  }
  if (usage.activeRows >= limits.activeRows) {
    return { allowed: false, reason: "active" };
  }
  if (
    usage.storageBytes + MAX_ANALYSIS_STORAGE_RESERVATION_BYTES >
    limits.storageBytes
  ) {
    return { allowed: false, reason: "storage" };
  }
  return { allowed: true };
}
