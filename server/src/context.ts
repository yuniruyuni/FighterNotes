import {
  RuntimeConfig,
  type RuntimeConfig as RuntimeConfigData,
} from "./config";
import type { Database } from "./infra/db/database";
import type { ILogger } from "./infra/logger/types";
import { createRuntimeServices } from "./infra/security";
import { CallbackClientImpl } from "./presentation/callback/impl";
import { createRawRepos } from "./repositories";
import { bindAllRepos, createFullCtx } from "./repositories/common/capability";
import type { Context } from "./usecases/context";
import type { RuntimeServices } from "./usecases/services";

interface ContextOptions {
  config?: RuntimeConfigData;
  services?: RuntimeServices;
  now?: Date;
}

export function createContext(
  db: Database,
  logger: ILogger,
  options: ContextOptions = {},
): Context {
  // Template repository と同じ layer 境界を保つため、具体的な callback がない段階でも残す。
  // 1. CallbackClient を repos 構築前に作成
  const callbackClient = new CallbackClientImpl();

  // 2. rawRepos 構築（将来 callbackClient を必要なリポジトリに注入する）
  const rawRepos = createRawRepos();
  const repos = bindAllRepos(rawRepos, createFullCtx(db));

  const config = options.config ?? RuntimeConfig.fromEnvironment();
  const ctx: Context = {
    now: options.now ?? new Date(),
    logger,
    db,
    rawRepos,
    repos,
    config,
    services: options.services ?? createRuntimeServices(db, config),
  };

  // 3. Context 構築後に遅延初期化
  callbackClient.initialize(ctx);

  return ctx;
}
