import { describe, expect, test } from "bun:test";
import type { PgDatabase } from "../../../infra/db";
import { sql } from "../../../infra/db";
import { PublishedAnalysisStorageUsage } from "../../../models/published-analysis";
import { createDbReadCtx, createDbWriteCtx } from "../../common/capability";
import { PublishedAnalysisRepository } from "../../published-analysis/postgres";
import { persistableAnalysis } from "../../test-support/published-analysis";
import { PublishedAnalysisStorageUsageRepository } from ".";

export function registerPublishedAnalysisStorageUsageRepositoryIntegrationTests(
  database: () => PgDatabase,
): void {
  describe("PublishedAnalysisStorageUsageRepository get", () => {
    const analysisRepository = new PublishedAnalysisRepository();
    const repository = new PublishedAnalysisStorageUsageRepository();

    test("作成後の4テーブルの実サイズと同じ集約値を返す", async () => {
      await analysisRepository.create(
        createDbWriteCtx(database()),
        persistableAnalysis(),
      );
      const row = await database().queryGet<{ bytes: string }>(sql`
        SELECT (
          pg_total_relation_size('published_analyses'::regclass) +
          pg_total_relation_size('published_analysis_findings'::regclass) +
          pg_total_relation_size('published_analysis_tactics'::regclass) +
          pg_total_relation_size(
            'published_analysis_create_events'::regclass
          )
        )::bigint AS bytes
      `);

      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisStorageUsage.Current(),
        ),
      ).toEqual({ bytes: Number(row?.bytes) });
    });
  });
}
