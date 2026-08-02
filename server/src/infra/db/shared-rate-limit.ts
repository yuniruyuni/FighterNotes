import { createHash } from "node:crypto";
import type { QueryResultRow } from "pg";
import type {
  RateLimitDecision,
  RateLimitPruneResult,
  SharingRateLimit,
  SharingRateLimitBucket,
} from "../../usecases/services";
import type { Database } from "./database";
import { sql } from "./sql";

interface RateLimitRow extends QueryResultRow {
  allowed: boolean;
  retry_after_seconds: number | string;
}

interface RateLimitPruneRow extends QueryResultRow {
  deleted: number;
  has_more: boolean;
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

  async prune(before: Date, limit: number): Promise<RateLimitPruneResult> {
    if (!Number.isSafeInteger(limit) || limit < 1 || limit > 10_000) {
      throw new Error("Rate limit prune limit must be from 1 to 10000");
    }
    const row = await this.db.queryGet<RateLimitPruneRow>(sql`
      WITH candidates AS MATERIALIZED (
        SELECT bucket, client_key_hash, window_started_at
        FROM published_analysis_rate_limits
        WHERE window_started_at < ${before}
        ORDER BY window_started_at ASC, bucket ASC, client_key_hash ASC
        LIMIT ${limit + 1}
        FOR UPDATE SKIP LOCKED
      ), selected AS (
        SELECT bucket, client_key_hash
        FROM candidates
        ORDER BY window_started_at ASC, bucket ASC, client_key_hash ASC
        LIMIT ${limit}
      ), deleted AS (
        DELETE FROM published_analysis_rate_limits AS rate_limit
        USING selected
        WHERE rate_limit.bucket = selected.bucket
          AND rate_limit.client_key_hash = selected.client_key_hash
        RETURNING rate_limit.bucket
      )
      SELECT
        (SELECT count(*)::integer FROM deleted) AS deleted,
        (SELECT count(*) FROM candidates) > ${limit} AS has_more
    `);
    if (!row) throw new Error("Rate limit prune returned no row");
    return { deleted: row.deleted, hasMore: row.has_more };
  }
}
