import { describe, expect, test } from "bun:test";
import type { PgDatabase } from "../../../infra/db";
import {
  PublishedAnalysisCreateEvent,
  type ShareId,
} from "../../../models/published-analysis";
import { createDbReadCtx, createDbWriteCtx } from "../../common/capability";
import { FIXTURE_ID } from "../../test-support/published-analysis";
import { PublishedAnalysisCreateEventRepository } from ".";

export function registerPublishedAnalysisCreateEventRepositoryIntegrationTests(
  database: () => PgDatabase,
): void {
  describe("PublishedAnalysisCreateEventRepository create -> get", () => {
    const repository = new PublishedAnalysisCreateEventRepository();

    test("作成したeventの全フィールドを復元する", async () => {
      const event: PublishedAnalysisCreateEvent = {
        analysisId: FIXTURE_ID,
        createdAt: new Date("2026-07-14T12:34:56.789Z"),
      };
      await repository.create(createDbWriteCtx(database()), event);

      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisCreateEvent.ByAnalysisId(event.analysisId),
        ),
      ).toEqual(event);
    });

    test("時刻Specの境界でcountし、対象だけをdeleteする", async () => {
      const events: PublishedAnalysisCreateEvent[] = [
        {
          analysisId: "Bbcdefghijklmnopqrstu_" as ShareId,
          createdAt: new Date("2026-07-14T00:00:00.000Z"),
        },
        {
          analysisId: "Cbcdefghijklmnopqrstu_" as ShareId,
          createdAt: new Date("2026-07-14T01:00:00.000Z"),
        },
        {
          analysisId: "Dbcdefghijklmnopqrstu_" as ShareId,
          createdAt: new Date("2026-07-14T02:00:00.000Z"),
        },
      ];
      for (const event of events) {
        await repository.create(createDbWriteCtx(database()), event);
      }

      const boundary = events[1].createdAt;
      expect(
        await repository.count(
          createDbReadCtx(database()),
          PublishedAnalysisCreateEvent.CreatedAtOrAfter(boundary),
        ),
      ).toBe(2);
      expect(
        await repository.count(
          createDbReadCtx(database()),
          PublishedAnalysisCreateEvent.CreatedBefore(boundary),
        ),
      ).toBe(1);
      expect(
        await repository.delete(
          createDbWriteCtx(database()),
          PublishedAnalysisCreateEvent.CreatedBefore(boundary),
        ),
      ).toBe(1);
      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisCreateEvent.ByAnalysisId(events[0].analysisId),
        ),
      ).toBeNull();
      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisCreateEvent.ByAnalysisId(events[1].analysisId),
        ),
      ).toEqual(events[1]);
    });
  });
}
