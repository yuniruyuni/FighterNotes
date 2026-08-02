import { describe, expect, mock, test } from "bun:test";
import type { AnalysisContext } from "~/modules/analysis/contracts.js";
import {
  syntheticAdviceReport,
  syntheticTacticStats,
} from "~/test-support/analysis.js";
import { ShareProjectionError } from "../domain/published-analysis.js";
import type { PublishedAnalysisShare } from "../domain/share.js";
import type { SharingServices } from "./ports.js";
import {
  createPublication,
  deletePublication,
  discardPublication,
  preparePublication,
  publicationErrorMessage,
  publicationLabel,
  renewPublication,
} from "./publication-service.js";

const context: AnalysisContext = {
  ownSide: "p1",
  p1: { character: "JURI" },
  p2: { character: "KEN" },
};
const report = syntheticAdviceReport();
const published: PublishedAnalysisShare = {
  id: "Abcdefghijklmnopqrstu_",
  url: "https://fighter.example/s/Abcdefghijklmnopqrstu_",
  expiresAt: "2026-08-22T00:00:00.000Z",
};

function createServices() {
  const create = mock(async () => published);
  const remove = mock(() => true);
  const save = mock(() => true);
  const deleteShare = mock(async () => undefined);
  const errorMessage = mock(() => "gateway error");
  const generateDeleteCode = mock(() => "ABCD-EFGH-JKLM");
  const services: SharingServices = {
    gateway: { create, delete: deleteShare, errorMessage },
    managedShares: {
      save,
      load: () => ({ available: true, shares: [] }),
      remove,
      subscribe: () => () => undefined,
    },
    capabilities: {
      copyText: async () => undefined,
      canShare: () => false,
      share: async () => undefined,
      confirm: () => true,
      origin: () => "https://fighter.example",
      isCancelledShare: () => false,
    },
    generateDeleteCode,
    now: () => new Date("2026-07-22T00:00:00.000Z"),
  };
  return {
    services,
    create,
    deleteShare,
    errorMessage,
    generateDeleteCode,
    remove,
    save,
  };
}

describe("publication service", () => {
  test("共有元を準備し、再作成時は削除codeだけを更新する", () => {
    const { services, generateDeleteCode } = createServices();
    const source = preparePublication(report, context, services);
    const renewed = renewPublication(source, services);

    expect(source).toEqual({ report, context, deleteCode: "ABCD-EFGH-JKLM" });
    expect(renewed).toEqual(source);
    expect(renewed).not.toBe(source);
    expect(generateDeleteCode).toHaveBeenCalledTimes(2);
  });

  test("共有候補を作成し、管理に必要な最小情報を保存する", async () => {
    const { services, create, save } = createServices();
    const source = { report, context, deleteCode: "ABCD-EFGH-JKLM" };

    await expect(createPublication(source, services)).resolves.toEqual({
      published,
      storedLocally: true,
    });
    expect(create).toHaveBeenCalledWith(
      expect.objectContaining({
        ownCharacter: "JURI",
        opponentCharacter: "KEN",
      }),
      source.deleteCode,
    );
    expect(save).toHaveBeenCalledWith(
      {
        id: published.id,
        deleteCode: source.deleteCode,
        createdAt: "2026-07-22T00:00:00.000Z",
        expiresAt: published.expiresAt,
        label: "JURI vs KEN",
      },
      new Date("2026-07-22T00:00:00.000Z"),
    );
  });

  test("ruleset v9はavailability付きSA/CA集計をgatewayへ送る", async () => {
    const { services, create, save } = createServices();
    const current = syntheticAdviceReport({
      ruleset_version: 9,
      tactic_stats: syntheticTacticStats({
        super_art_stats_complete: false,
        opponent_super_art_stats_complete: false,
        sa1_used: 0,
        sa2_used: 0,
        sa3_used: 0,
        ca_used: 0,
        opponent_sa1_used: 0,
        opponent_sa2_used: 0,
        opponent_sa3_used: 0,
        opponent_ca_used: 0,
      }),
    });

    await expect(
      createPublication(
        { report: current, context, deleteCode: "ABCD-EFGH-JKLM" },
        services,
      ),
    ).resolves.toEqual({ published, storedLocally: true });
    expect(create).toHaveBeenCalledWith(
      expect.objectContaining({
        rulesetVersion: 9,
        superArts: {
          own: { availability: "unavailable" },
          opponent: { availability: "unavailable" },
        },
      }),
      "ABCD-EFGH-JKLM",
    );
    expect(save).toHaveBeenCalledTimes(1);
  });

  test("不要な共有を削除し、local保存状態に応じて管理recordも消す", async () => {
    const first = createServices();
    const source = { report, context, deleteCode: "ABCD-EFGH-JKLM" };
    await discardPublication(published, source, first.services);
    expect(first.deleteShare).toHaveBeenCalledWith(
      published,
      source.deleteCode,
    );

    await expect(
      deletePublication(published, source, false, first.services),
    ).resolves.toBe(true);
    expect(first.remove).not.toHaveBeenCalled();

    const second = createServices();
    second.remove.mockImplementation(() => false);
    await expect(
      deletePublication(published, source, true, second.services),
    ).resolves.toBe(false);
    expect(second.remove).toHaveBeenCalledWith(published.id);
  });

  test("labelとerrorをdomain情報から利用者向け文言へ変換する", () => {
    const { services, errorMessage } = createServices();
    expect(publicationLabel(context)).toBe("JURI vs KEN");
    expect(
      publicationLabel({ ownSide: "p2", p1: {}, p2: { character: "RYU" } }),
    ).toBe("RYU vs 未指定");

    const projectionError = new ShareProjectionError("character required");
    expect(publicationErrorMessage(projectionError, services)).toBe(
      "character required",
    );
    expect(publicationErrorMessage(new Error("network"), services)).toBe(
      "gateway error",
    );
    expect(errorMessage).toHaveBeenCalledTimes(1);
  });
});
