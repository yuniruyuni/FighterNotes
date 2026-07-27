import type { CleanupSettings } from "./config";
import type { Fail } from "./models/common/fail";
import { fail } from "./models/common/fail";
import type { Result } from "./models/common/result";
import type { Context } from "./usecases/context";
import {
  cleanupPublishedAnalysisBatchUsecase,
  prunePublishedAnalysisCreateEventsUsecase,
} from "./usecases/published-analysis";

const BATCH_ARGUMENT_PREFIX = "--batch=";
const DAY_MILLISECONDS = 24 * 60 * 60 * 1_000;
const QUOTA_EVENT_RETENTION_DAYS = 2;

export type BatchCommand = "cleanup";

export interface CleanupPublishedAnalysesResult {
  readonly expired: number;
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
  let expired = 0;

  for (let batches = 1; batches <= settings.maxBatches; batches++) {
    const batch = await cleanupPublishedAnalysisBatchUsecase({
      limit: settings.batchSize,
      retentionCutoff,
    }).run(ctx);
    if (!batch.ok) return batch;

    expired += batch.value.deleted;
    if (!batch.value.hasMore) {
      const quotaEvents = await prunePublishedAnalysisCreateEventsUsecase(
        quotaEventCutoff(ctx.now),
      ).run(ctx);
      if (!quotaEvents.ok) return quotaEvents;

      return {
        ok: true,
        value: { expired, quotaEvents: quotaEvents.value, batches },
      };
    }
  }

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

function quotaEventCutoff(now: Date): Date {
  return new Date(
    Date.UTC(now.getUTCFullYear(), now.getUTCMonth(), now.getUTCDate()) -
      QUOTA_EVENT_RETENTION_DAYS * DAY_MILLISECONDS,
  );
}
