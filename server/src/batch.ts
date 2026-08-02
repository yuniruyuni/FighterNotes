import type { CleanupSettings } from "./config";
import type { Fail } from "./models/common/fail";
import { fail } from "./models/common/fail";
import type { Result } from "./models/common/result";
import type { Context } from "./usecases/context";
import {
  cleanupExpiredPublishedAnalysisBatchUsecase,
  cleanupRetainedPublishedAnalysisBatchUsecase,
  prunePublishedAnalysisCreateEventsUsecase,
  pruneSharingRateLimitsBatchUsecase,
} from "./usecases/published-analysis";

const BATCH_ARGUMENT_PREFIX = "--batch=";
const DAY_MILLISECONDS = 24 * 60 * 60 * 1_000;
const QUOTA_EVENT_RETENTION_DAYS = 2;
const RATE_LIMIT_RETENTION_MILLISECONDS = 2 * 60 * 1_000;

export type BatchCommand = "cleanup";

export interface CleanupPublishedAnalysesResult {
  readonly expired: number;
  readonly rateLimits: number;
  readonly quotaEvents: number;
  readonly batches: number;
}

export function parseBatchCommand(
  argv: readonly string[],
): BatchCommand | null {
  const arguments_ = argv.filter((value) =>
    value.startsWith(BATCH_ARGUMENT_PREFIX),
  );
  if (arguments_.length === 0) return null;
  if (arguments_.length > 1) {
    throw new Error("Only one --batch argument may be specified");
  }

  const command = arguments_[0]?.slice(BATCH_ARGUMENT_PREFIX.length);
  if (command === "cleanup") return command;
  throw new Error(`Unknown batch command: ${command || "(empty)"}`);
}

export async function runBatchCommand(
  command: BatchCommand,
  ctx: Context,
): Promise<boolean> {
  switch (command) {
    case "cleanup": {
      const result = await runPublishedAnalysisCleanup(ctx, ctx.config.cleanup);
      if (!result.ok) {
        ctx.logger.error("Fighter cleanup failed", {
          code: result.error.code,
          message: result.error.message,
        });
        return false;
      }
      ctx.logger.info(
        `Fighter cleanup completed: expired=${result.value.expired}; ` +
          `rate_limits=${result.value.rateLimits}; ` +
          `quota_events=${result.value.quotaEvents}; batches=${result.value.batches}`,
      );
      return true;
    }
  }
}

export async function runPublishedAnalysisCleanup(
  ctx: Context,
  settings: CleanupSettings,
): Promise<Result<CleanupPublishedAnalysesResult, Fail>> {
  const retentionCutoff = new Date(
    ctx.now.getTime() - settings.retentionDays * DAY_MILLISECONDS,
  );
  const expired = await cleanupParentRows(ctx, settings, retentionCutoff);
  if (!expired.ok) return expired;
  const rateLimits = await pruneRateLimits(ctx, settings);
  if (!rateLimits.ok) return rateLimits;
  const quotaEvents = await prunePublishedAnalysisCreateEventsUsecase(
    quotaEventCutoff(ctx.now),
  ).run(ctx);
  if (!quotaEvents.ok) return quotaEvents;

  return {
    ok: true,
    value: {
      expired: expired.value.deleted,
      rateLimits: rateLimits.value,
      quotaEvents: quotaEvents.value,
      batches: expired.value.batches,
    },
  };
}

interface ParentCleanupResult {
  readonly deleted: number;
  readonly batches: number;
}

async function cleanupParentRows(
  ctx: Context,
  settings: CleanupSettings,
  retentionCutoff: Date,
): Promise<Result<ParentCleanupResult, Fail>> {
  const phases = [
    (limit: number) =>
      cleanupExpiredPublishedAnalysisBatchUsecase({
        at: ctx.now,
        limit,
      }).run(ctx),
    (limit: number) =>
      cleanupRetainedPublishedAnalysisBatchUsecase({
        retentionCutoff,
        limit,
      }).run(ctx),
  ];
  let deleted = 0;
  let batches = 0;

  for (const runBatch of phases) {
    for (;;) {
      if (batches >= settings.maxBatches) {
        return parentCleanupLimitFailure(ctx, settings);
      }
      batches += 1;
      const batch = await runBatch(settings.batchSize);
      if (!batch.ok) return batch;
      deleted += batch.value.deleted;
      if (!batch.value.hasMore) break;
    }
  }
  return { ok: true, value: { deleted, batches } };
}

function parentCleanupLimitFailure(
  ctx: Context,
  settings: CleanupSettings,
): Result<never, Fail> {
  ctx.logger.warn("Cleanup stopped at the configured batch safety limit", {
    batchSize: settings.batchSize,
    maxBatches: settings.maxBatches,
  });
  return {
    ok: false,
    error: fail(
      "RESOURCE_LIMIT",
      "Cleanup stopped at the configured batch safety limit",
    ),
  };
}

async function pruneRateLimits(
  ctx: Context,
  settings: CleanupSettings,
): Promise<Result<number, Fail>> {
  const before = new Date(
    ctx.now.getTime() - RATE_LIMIT_RETENTION_MILLISECONDS,
  );
  let deleted = 0;
  for (let batch = 1; batch <= settings.maxBatches; batch += 1) {
    const result = await pruneSharingRateLimitsBatchUsecase({
      before,
      limit: settings.batchSize,
    }).run(ctx);
    if (!result.ok) return result;
    deleted += result.value.deleted;
    if (!result.value.hasMore) return { ok: true, value: deleted };
  }
  ctx.logger.warn("Cleanup stopped at the configured batch safety limit", {
    resource: "rate_limits",
    batchSize: settings.batchSize,
    maxBatches: settings.maxBatches,
  });
  return {
    ok: false,
    error: fail(
      "RESOURCE_LIMIT",
      "Cleanup stopped at the configured batch safety limit",
    ),
  };
}

function quotaEventCutoff(now: Date): Date {
  return new Date(
    Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate()) -
      QUOTA_EVENT_RETENTION_DAYS * DAY_MILLISECONDS,
  );
}
