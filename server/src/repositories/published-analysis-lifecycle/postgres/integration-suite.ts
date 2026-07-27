import { describe, expect, test } from "bun:test";
import type { PgDatabase } from "../../../infra/db";
import {
  type PublishedAnalysisLifecycle,
  PublishedAnalysisLifecycle as PublishedAnalysisLifecycleSpec,
  type ShareId,
} from "../../../models/published-analysis";
import { createDbReadCtx, createDbWriteCtx } from "../../common/capability";
import { PublishedAnalysisRepository } from "../../published-analysis/postgres";
import { persistableAnalysis } from "../../test-support/published-analysis";
import { PublishedAnalysisLifecycleRepository } from ".";

export function registerPublishedAnalysisLifecycleRepositoryIntegrationTests(
  database: () => PgDatabase,
): void {
  describe("PublishedAnalysisLifecycleRepository create projection -> list", () => {
    const analysisRepository = new PublishedAnalysisRepository();
    const repository = new PublishedAnalysisLifecycleRepository();

    test("作成したanalysisからlifecycleの全フィールドを復元する", async () => {
      const persisted = persistableAnalysis();
      await analysisRepository.create(createDbWriteCtx(database()), persisted);

      const page = await repository.list(
        createDbReadCtx(database()),
        PublishedAnalysisLifecycleSpec.ById(persisted.id),
        { limit: 1, sort: PublishedAnalysisLifecycleSpec.defaultSort },
      );
      const expected: PublishedAnalysisLifecycle = {
        id: persisted.id,
        deletePasswordHash: persisted.deletePasswordHash,
        createdAt: persisted.createdAt,
        expiresAt: persisted.expiresAt,
      };
      expect(page.items).toEqual([expected]);
      expect(page.hasMore).toBe(false);
      expect(page.nextCursor).toBeUndefined();
    });

    test("各Specの境界でcountする", async () => {
      const analyses = lifecycleFixtures();
      for (const analysis of analyses) {
        await analysisRepository.create(createDbWriteCtx(database()), analysis);
      }

      expect(
        await repository.count(
          createDbReadCtx(database()),
          PublishedAnalysisLifecycleSpec.ByIds(),
        ),
      ).toBe(0);
      expect(
        await repository.count(
          createDbReadCtx(database()),
          PublishedAnalysisLifecycleSpec.ByIds(analyses[0].id, analyses[2].id),
        ),
      ).toBe(2);
      expect(
        await repository.count(
          createDbReadCtx(database()),
          PublishedAnalysisLifecycleSpec.ActiveAt(analyses[2].expiresAt),
        ),
      ).toBe(1);
      expect(
        await repository.count(
          createDbReadCtx(database()),
          PublishedAnalysisLifecycleSpec.ExpiredAt(analyses[2].expiresAt),
        ),
      ).toBe(2);
      expect(
        await repository.count(
          createDbReadCtx(database()),
          PublishedAnalysisLifecycleSpec.CreatedAtOrBefore(
            analyses[1].createdAt,
          ),
        ),
      ).toBe(2);
    });

    test("既定ソートでlimitを適用して次カーソルを返す", async () => {
      const analyses = lifecycleFixtures();
      for (const analysis of analyses) {
        await analysisRepository.create(createDbWriteCtx(database()), analysis);
      }

      const page = await repository.list(
        createDbReadCtx(database()),
        PublishedAnalysisLifecycleSpec.ByIds(
          ...analyses.map((analysis) => analysis.id),
        ),
        { limit: 1 },
      );

      expect(page).toEqual({
        items: [lifecycleOf(analyses[1])],
        hasMore: true,
        nextCursor: {
          expiresAt: analyses[1].expiresAt.toISOString(),
          id: analyses[1].id,
        },
      });
    });

    test("指定ソートで全件を並べ、続きがないことを返す", async () => {
      const analyses = lifecycleFixtures();
      for (const analysis of analyses) {
        await analysisRepository.create(createDbWriteCtx(database()), analysis);
      }

      const page = await repository.list(
        createDbReadCtx(database()),
        PublishedAnalysisLifecycleSpec.ByIds(
          ...analyses.map((analysis) => analysis.id),
        ),
        { limit: 3, sort: { keys: ["createdAt"], order: "desc" } },
      );

      expect(page).toEqual({
        items: [...analyses].reverse().map(lifecycleOf),
        hasMore: false,
        nextCursor: undefined,
      });
    });
  });
}

function lifecycleFixtures() {
  return [
    persistableAnalysis({
      id: "Bbcdefghijklmnopqrstu_" as ShareId,
      now: new Date("2026-07-01T00:00:00.000Z"),
      retentionDays: 10,
    }),
    persistableAnalysis({
      id: "Cbcdefghijklmnopqrstu_" as ShareId,
      now: new Date("2026-07-02T00:00:00.000Z"),
      retentionDays: 1,
    }),
    persistableAnalysis({
      id: "Dbcdefghijklmnopqrstu_" as ShareId,
      now: new Date("2026-07-03T00:00:00.000Z"),
      retentionDays: 5,
    }),
  ];
}

function lifecycleOf(
  analysis: ReturnType<typeof persistableAnalysis>,
): PublishedAnalysisLifecycle {
  return {
    id: analysis.id,
    deletePasswordHash: analysis.deletePasswordHash,
    createdAt: analysis.createdAt,
    expiresAt: analysis.expiresAt,
  };
}
