import { describe, expect, test } from "bun:test";
import { type PgDatabase, sql } from "../../../infra/db";
import {
  PublishedAnalysisLifecycle,
  PublishedAnalysisStorageUsage,
} from "../../../models/published-analysis";
import { createDbReadCtx, createDbWriteCtx } from "../../common/capability";
import { PublishedAnalysisRepository } from "../../published-analysis/postgres";
import { PublishedAnalysisLifecycleRepository } from "../../published-analysis-lifecycle/postgres";
import { persistableAnalysis } from "../../test-support/published-analysis";
import { PublishedAnalysisStorageUsageRepository } from ".";

export function registerPublishedAnalysisStorageUsageRepositoryIntegrationTests(
  database: () => PgDatabase,
): void {
  describe("PublishedAnalysisStorageUsageRepository get", () => {
    const analysisRepository = new PublishedAnalysisRepository();
    const lifecycleRepository = new PublishedAnalysisLifecycleRepository();
    const repository = new PublishedAnalysisStorageUsageRepository();

    test("logical payloadを集計しDELETE直後にVACUUMなしで回復する", async () => {
      const persisted = persistableAnalysis();
      await analysisRepository.create(createDbWriteCtx(database()), persisted);

      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisStorageUsage.Current(),
        ),
      ).toEqual({ bytes: persisted.logicalSizeBytes });

      expect(
        await lifecycleRepository.delete(
          createDbWriteCtx(database()),
          PublishedAnalysisLifecycle.ById(persisted.id),
        ),
      ).toBe(1);
      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisStorageUsage.Current(),
        ),
      ).toEqual({ bytes: 0 });

      // A rollback revision does not send the new column. The schema default
      // must keep that old INSERT compatible and account it conservatively.
      await database().transaction(async (tx) => {
        await tx.queryRun(sql.raw("SET LOCAL ROLE fighter_app"));
        await tx.queryRun(sql`
          INSERT INTO published_analyses (
            id, schema_version, ruleset_version, presentation_revision,
            own_character, opponent_character,
            rounds_detected, rounds_won, rounds_lost, rounds_unresolved,
            created_at, expires_at
          ) VALUES (
            ${"Bbcdefghijklmnopqrstu_"}, 1, 3, 1, ${"LUKE"}, ${"CHUN_LI"},
            0, 0, 0, 0,
            ${new Date("2026-07-13T00:00:00.000Z")},
            ${new Date("2026-08-13T00:00:00.000Z")}
          )
        `);
      });
      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisStorageUsage.Current(),
        ),
      ).toEqual({ bytes: 8 * 1024 });
    });
  });
}
