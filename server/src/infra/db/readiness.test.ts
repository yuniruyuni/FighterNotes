import { describe, expect, test } from "bun:test";
import { RuntimeConfig } from "../../config";
import type { ILogger } from "../logger/types";
import type { Database } from "./database";
import { initDatabase } from "./index";
import { createDatabaseReadiness } from "./readiness";
import type { SQLFragment } from "./sql";

function logger(entries: unknown[][]): ILogger {
  const instance: ILogger = {
    debug() {},
    info(message, ...args) {
      entries.push(["info", message, ...args]);
    },
    warn(message, ...args) {
      entries.push(["warn", message, ...args]);
    },
    error(message, ...args) {
      entries.push(["error", message, ...args]);
    },
    child() {
      return instance;
    },
  };
  return instance;
}

function transactionDatabase(compatible: boolean): {
  readonly db: Database;
  readonly queries: SQLFragment[];
} {
  const queries: SQLFragment[] = [];
  const tx = databaseStub({
    async queryGet<T>(fragment: SQLFragment): Promise<T | null> {
      queries.push(fragment);
      if (fragment.query.includes("set_config")) {
        return { statement_timeout: "750ms" } as T;
      }
      return { compatible } as T;
    },
  });
  return {
    queries,
    db: databaseStub({
      async readTransaction<T>(fn: (database: Database) => Promise<T>) {
        return fn(tx);
      },
    }),
  };
}

function databaseStub(overrides: Partial<Database> = {}): Database {
  const unused = async (): Promise<never> => {
    throw new Error("unexpected database operation");
  };
  return {
    queryGet: unused,
    queryAll: unused,
    queryRun: unused,
    transaction: unused,
    readTransaction: unused,
    close: async () => undefined,
    ...overrides,
  } as Database;
}

describe("database readiness", () => {
  test("uses a read-only compatibility query with a short statement timeout", async () => {
    const fixture = transactionDatabase(true);
    const readiness = createDatabaseReadiness(fixture.db, logger([]));
    expect(await readiness.check()).toBe(true);
    expect(fixture.queries).toHaveLength(2);
    expect(fixture.queries[0]?.params).toEqual(["750ms"]);
    const compatibilityQuery = fixture.queries[1]?.query ?? "";
    expect(compatibilityQuery).toContain("transaction_read_only");
    expect(compatibilityQuery).toContain("required_columns");
    expect(compatibilityQuery).toContain(
      "published_analyses_schema_version_check",
    );
    expect(compatibilityQuery).toContain(
      "published_analyses_own_character_check",
    );
    expect(compatibilityQuery).toContain(
      "published_analyses_opponent_character_check",
    );
    const compatibilityParams = fixture.queries[1]?.params ?? [];
    expect(compatibilityParams).toContainEqual(
      expect.stringContaining(
        "own_character = ANY (ARRAY['A_K_I'::text, 'AKUMA'::text",
      ),
    );
    expect(compatibilityParams).toContainEqual(
      expect.stringContaining(
        "'TERRY'::text, 'YASMINE'::text, 'ZANGIEF'::text",
      ),
    );
    expect(compatibilityParams).toContainEqual(
      expect.stringContaining("opponent_character = ANY"),
    );
    expect(compatibilityQuery).toContain("logical_size_bytes");
    expect(compatibilityQuery).toContain("format_type");
    expect(compatibilityQuery).toContain("required_defaults");
    expect(compatibilityQuery).toContain("required_constraints");
    expect(compatibilityQuery).toContain("required_indexes");
    expect(compatibilityQuery).toContain("indisvalid");
    expect(compatibilityQuery).toContain("ON DELETE CASCADE");
    expect(compatibilityQuery).toContain("published_analysis_rate_limits");
    expect(compatibilityQuery).toContain("published_analysis_rate_limits_pkey");
    expect(compatibilityQuery).toContain("published_analysis_super_arts");
    expect(compatibilityQuery).toContain("published_analysis_own_super_arts");
    expect(compatibilityQuery).toContain(
      "published_analysis_opponent_super_arts",
    );
    expect(compatibilityQuery).toContain("ARRAY[3, 4, 5, 6, 7, 8, 9]");
    expect(compatibilityQuery).toContain("'DELETE'");
    expect(compatibilityQuery).toContain("has_table_privilege");
    expect(compatibilityQuery).toContain("has_column_privilege");
  });

  test("fails closed for missing schema or grants", async () => {
    const fixture = transactionDatabase(false);
    const entries: unknown[][] = [];
    const readiness = createDatabaseReadiness(fixture.db, logger(entries));
    expect(await readiness.check()).toBe(false);
    expect(entries).toEqual([
      ["warn", "Database readiness check failed", { reason: "incompatible" }],
    ]);
  });

  test("does not disclose tunnel or credential errors", async () => {
    for (const secretError of [
      "connect ECONNREFUSED db-tunnel.internal:5432",
      "password authentication failed: super-secret-password",
    ]) {
      const entries: unknown[][] = [];
      const db = databaseStub({
        async readTransaction() {
          throw new Error(secretError);
        },
      });
      const readiness = createDatabaseReadiness(db, logger(entries));
      expect(await readiness.check()).toBe(false);
      const serialized = JSON.stringify(entries);
      expect(serialized).not.toContain(secretError);
      expect(serialized).not.toContain("super-secret-password");
      expect(entries).toEqual([
        ["warn", "Database readiness check failed", { reason: "unavailable" }],
      ]);
    }
  });

  test("bounds waiting and coalesces concurrent probes", async () => {
    let calls = 0;
    const db = databaseStub({
      async readTransaction() {
        calls += 1;
        return new Promise<never>(() => undefined);
      },
    });
    const readiness = createDatabaseReadiness(db, logger([]), {
      timeoutMs: 5,
    });
    expect(await Promise.all([readiness.check(), readiness.check()])).toEqual([
      false,
      false,
    ]);
    expect(calls).toBe(1);
  });

  test("pool initialization log does not claim a successful connection", async () => {
    const entries: unknown[][] = [];
    const config = RuntimeConfig.fromEnvironment({}).database;
    const db = initDatabase(logger(entries), config);
    await db.close();
    expect(entries).toEqual([["info", "Database connection pool initialized"]]);
  });
});
