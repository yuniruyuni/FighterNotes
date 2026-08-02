import { createHash } from "node:crypto";
import type { QueryResultRow } from "pg";
import type {
  RateLimitDecision,
  SharingRateLimit,
  SharingRateLimitBucket,
} from "../../usecases/services";
import type { Database } from "./database";
import { sql } from "./sql";

interface RateLimitRow extends QueryResultRow {
  allowed: boolean;
  retry_after_seconds: number | string;
}

export class PostgresSharingRateLimit implements SharingRateLimit {
  constructor(private readonly db: Database) {}

  async consume(
    bucket: SharingRateLimitBucket,
    clientKey: string,
    limit: number,
  ): Promise<RateLimitDecision> {
    const clientKeyHash = createHash("sha256").update(clientKey).digest("hex");
    const row = await this.db.queryGet<RateLimitRow>(sql`
      WITH consumed AS (
        INSERT INTO published_analysis_rate_limits (
          bucket, client_key_hash, window_started_at, request_count
        ) VALUES (
          ${bucket}, ${clientKeyHash}, clock_timestamp(), 1
        )
        ON CONFLICT (bucket, client_key_hash) DO UPDATE SET
          request_count = CASE
            WHEN published_analysis_rate_limits.window_started_at
              + INTERVAL '1 minute' <= EXCLUDED.window_started_at
            THEN 1
            ELSE LEAST(
              published_analysis_rate_limits.request_count + 1,
              ${limit + 1}
            )
          END,
          window_started_at = CASE
            WHEN published_analysis_rate_limits.window_started_at
              + INTERVAL '1 minute' <= EXCLUDED.window_started_at
            THEN EXCLUDED.window_started_at
            ELSE published_analysis_rate_limits.window_started_at
          END
        RETURNING request_count, window_started_at
      )
      SELECT
        request_count <= ${limit} AS allowed,
        CASE WHEN request_count <= ${limit} THEN 0 ELSE GREATEST(
          1,
          CEIL(EXTRACT(EPOCH FROM (
            window_started_at + INTERVAL '1 minute' - clock_timestamp()
          )))
        ) END AS retry_after_seconds
      FROM consumed
    `);
    if (!row) throw new Error("Shared rate limit returned no row");
    return {
      allowed: row.allowed,
      retryAfterSeconds: Number(row.retry_after_seconds),
    };
  }
}
