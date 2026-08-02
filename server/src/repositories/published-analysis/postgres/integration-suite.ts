import { describe, expect, test } from "bun:test";
import type { PgDatabase } from "../../../infra/db";
import type {
  PublishedAnalysis,
  ShareId,
} from "../../../models/published-analysis";
import {
  createPublishedAnalysisContent,
  PublishedAnalysis as PublishedAnalysisSpec,
} from "../../../models/published-analysis";
import { createDbReadCtx, createDbWriteCtx } from "../../common/capability";
import {
  candidate,
  persistableAnalysis,
  v9Candidate,
} from "../../test-support/published-analysis";
import { PublishedAnalysisRepository } from ".";

export function registerPublishedAnalysisRepositoryIntegrationTests(
  database: () => PgDatabase,
): void {
  describe("PublishedAnalysisRepository create -> get", () => {
    const repository = new PublishedAnalysisRepository();

    test("作成した公開モデルの全フィールドを復元する", async () => {
      const persisted = persistableAnalysis();
      await repository.create(createDbWriteCtx(database()), persisted);

      const restored = await repository.get(
        createDbReadCtx(database()),
        PublishedAnalysisSpec.ById(persisted.id).and(
          PublishedAnalysisSpec.ActiveAt(persisted.createdAt),
        ),
      );
      const expected: PublishedAnalysis = {
        id: persisted.id,
        content: persisted.content,
        createdAt: persisted.createdAt,
        expiresAt: persisted.expiresAt,
      };
      expect(restored).toEqual(expected);
    });

    test("ruleset v9のSA/CA availabilityと両者集計を復元する", async () => {
      const persisted = persistableAnalysis({
        id: "Cbcdefghijklmnopqrstu_" as ShareId,
        rulesetVersion: 9,
      });
      await repository.create(createDbWriteCtx(database()), persisted);

      const restored = await repository.get(
        createDbReadCtx(database()),
        PublishedAnalysisSpec.ById(persisted.id),
      );
      expect(restored?.content.superArts).toEqual(persisted.content.superArts);
    });

    test("ruleset v9のpartialをcomplete=false side行から復元する", async () => {
      const input = v9Candidate();
      const observed = input.superArts;
      if (!observed || observed.own.availability === "unavailable") {
        throw new Error("fixture is invalid");
      }
      input.superArts = {
        own: { ...observed.own, availability: "partial" },
        opponent: { availability: "unavailable" },
      };
      const content = createPublishedAnalysisContent(input);
      if (!content.ok) throw new Error("fixture is invalid");
      const persisted = {
        ...persistableAnalysis({
          id: "Dbcdefghijklmnopqrstu_" as ShareId,
          rulesetVersion: 9,
        }),
        content: content.value,
      };
      await repository.create(createDbWriteCtx(database()), persisted);

      const restored = await repository.get(
        createDbReadCtx(database()),
        PublishedAnalysisSpec.ById(persisted.id),
      );
      expect(restored?.content.superArts).toEqual(content.value.superArts);
    });

    test("期限境界では作成済みモデルを返さない", async () => {
      const persisted = persistableAnalysis();
      await repository.create(createDbWriteCtx(database()), persisted);

      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisSpec.ById(persisted.id).and(
            PublishedAnalysisSpec.ActiveAt(persisted.expiresAt),
          ),
        ),
      ).toBeNull();
    });

    test("findingsが空でも作成した公開モデルを復元する", async () => {
      const content = createPublishedAnalysisContent({
        ...candidate(),
        findings: [],
      });
      if (!content.ok) throw new Error("fixture is invalid");
      const persisted = {
        ...persistableAnalysis({
          id: "Bbcdefghijklmnopqrstu_" as ShareId,
        }),
        content: content.value,
      };

      await repository.create(createDbWriteCtx(database()), persisted);

      expect(
        await repository.get(
          createDbReadCtx(database()),
          PublishedAnalysisSpec.ById(persisted.id),
        ),
      ).toEqual({
        id: persisted.id,
        content: persisted.content,
        createdAt: persisted.createdAt,
        expiresAt: persisted.expiresAt,
      });
    });
  });
}
