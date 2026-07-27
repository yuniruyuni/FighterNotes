import { describe, expect, test } from "bun:test";
import { PublishedAnalysisLifecycle } from "./lifecycle";
import { type PublishedAnalysis, parseShareId } from "./model";

const id = parseShareId("Abcdefghijklmnopqrstu_");
if (!id) throw new Error("invalid test share ID");

describe("PublishedAnalysisLifecycle", () => {
  test("全query specificationを型付きdataへ変換する", () => {
    const at = new Date("2026-07-22T00:00:00.000Z");
    const secondId = "Zbcdefghijklmnopqrstu_" as PublishedAnalysis["id"];

    expect(PublishedAnalysisLifecycle.ById(id)).toMatchObject({
      type: "ById",
      id,
    });
    expect(PublishedAnalysisLifecycle.ByIds(id, secondId)).toMatchObject({
      type: "ByIds",
      ids: [id, secondId],
    });
    expect(PublishedAnalysisLifecycle.ActiveAt(at)).toMatchObject({
      type: "ActiveAt",
      at,
    });
    expect(PublishedAnalysisLifecycle.ExpiredAt(at)).toMatchObject({
      type: "ExpiredAt",
      at,
    });
    expect(PublishedAnalysisLifecycle.CreatedAtOrBefore(at)).toMatchObject({
      type: "CreatedAtOrBefore",
      cutoff: at,
    });
  });

  test("指定sort keyを安定した文字列cursorへ変換する", () => {
    const lifecycle: PublishedAnalysisLifecycle = {
      id,
      deletePasswordHash: null,
      createdAt: new Date("2026-07-20T00:00:00.000Z"),
      expiresAt: new Date("2026-08-20T00:00:00.000Z"),
    };

    expect(
      PublishedAnalysisLifecycle.cursor(lifecycle, [
        "createdAt",
        "expiresAt",
        "id",
      ]),
    ).toEqual({
      createdAt: "2026-07-20T00:00:00.000Z",
      expiresAt: "2026-08-20T00:00:00.000Z",
      id,
    });
    expect(PublishedAnalysisLifecycle.defaultSort).toEqual({
      keys: ["expiresAt", "id"],
      order: "asc",
    });
  });
});
