import type { Database } from "../infra/db/database";
import type { ILogger } from "../infra/logger/types";
import type { Repos } from "../repositories";
import type {
  DbRead,
  DbWrite,
  FullRepos,
  Service,
} from "../repositories/common/capability";
import type { RuntimeServices } from "./services";

export interface Context {
  now: Date;
  logger: ILogger;
  db: Database;
  rawRepos: Repos;
  repos: FullRepos<Repos>;
  config: RuntimeConfig;
  services: RuntimeServices;
}

type DbReadRepos<T> = { [K in keyof T]: DbRead<T[K]> };
type DbWriteRepos<T> = { [K in keyof T]: DbWrite<T[K]> };
type ServiceRepos<T> = { [K in keyof T]: Service<T[K]> };

export interface PreContext {
  now: Date;
  logger: ILogger;
  services: RuntimeServices;
}

export interface ReadContext {
  now: Date;
  logger: ILogger;
  repos: DbReadRepos<Repos>;
}

export interface ProcessContext {
  now: Date;
  logger: ILogger;
  services: RuntimeServices;
}

export interface WriteContext {
  now: Date;
  logger: ILogger;
  repos: DbWriteRepos<Repos>;
}

export interface PostContext {
  now: Date;
  logger: ILogger;
  repos: ServiceRepos<Repos>;
}

export interface FinishContext {
  now: Date;
  logger: ILogger;
  repos: DbWriteRepos<Repos>;
}

import type { RuntimeConfig } from "../config";
