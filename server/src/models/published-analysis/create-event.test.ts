import { describe, expect, test } from "bun:test";
import { PublishedAnalysisCreateEvent, startOfUtcDay } from "./create-event";
import { parseShareId } from "./model";

const id = parseShareId("Abcdefghijklmnopqrstu_");
if (!id) throw new Error("invalid test share ID");

describe("PublishedAnalysisCreateEvent", () => {
  test("全query specificationを型付きdataへ変換する", () => {
    const start = new Date("2026-07-22T00:00:00.000Z");
    const cutoff = new Date("2026-07-23T00:00:00.000Z");

    expect(PublishedAnalysisCreateEvent.ByAnalysisId(id)).toMatchObject({
      type: "ByAnalysisId",
      analysisId: id,
    });
    expect(PublishedAnalysisCreateEvent.CreatedAtOrAfter(start)).toMatchObject({
      type: "CreatedAtOrAfter",
      start,
    });
    expect(PublishedAnalysisCreateEvent.CreatedBefore(cutoff)).toMatchObject({
      type: "CreatedBefore",
      cutoff,
    });
  });

  test("入力時刻が属するUTC日の開始を返す", () => {
    const source = new Date("2026-07-22T23:59:59.999-07:00");
    expect(startOfUtcDay(source).toISOString()).toBe(
      "2026-07-23T00:00:00.000Z",
    );
    expect(source.toISOString()).toBe("2026-07-23T06:59:59.999Z");
  });
});
