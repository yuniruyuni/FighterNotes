import type {
  DeletePassword,
  DeletePasswordHash,
  ShareId,
} from "../models/published-analysis";

export interface PublishedAnalysisSecurity {
  generateShareId(): ShareId;
  hashDeletePassword(password: DeletePassword): Promise<DeletePasswordHash>;
  verifyDeletePassword(
    password: DeletePassword,
    hash: DeletePasswordHash,
  ): Promise<boolean>;
}

export class SecurityCapacityError extends Error {
  constructor() {
    super("Security capacity unavailable");
    this.name = "SecurityCapacityError";
  }
}

export type SharingRateLimitBucket = "create" | "delete" | "public_read";

export interface RateLimitDecision {
  readonly allowed: boolean;
  readonly retryAfterSeconds: number;
}

export interface SharingRateLimit {
  consume(
    bucket: SharingRateLimitBucket,
    clientKey: string,
    limit: number,
  ): Promise<RateLimitDecision>;
}

export interface RuntimeServices {
  publishedAnalysisSecurity: PublishedAnalysisSecurity;
  sharingRateLimit: SharingRateLimit;
}
