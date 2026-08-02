import type { RuntimeConfig } from "../../config";
import type { Database } from "../db";
import { PostgresSharingRateLimit } from "../db/shared-rate-limit";
import {
  createPublishedAnalysisSecurity,
  publishedAnalysisSecurity,
} from "./published-analysis";

export {
  createPublishedAnalysisSecurity,
  publishedAnalysisSecurity,
} from "./published-analysis";

export function createRuntimeServices(db?: Database, config?: RuntimeConfig) {
  return {
    publishedAnalysisSecurity: config
      ? createPublishedAnalysisSecurity(config.sharing.argon2)
      : publishedAnalysisSecurity,
    sharingRateLimit: db
      ? new PostgresSharingRateLimit(db)
      : {
          async consume(): Promise<never> {
            throw new Error("Shared rate limit requires a database");
          },
        },
  };
}
