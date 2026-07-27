import { describe, expect, test } from "bun:test";
import { PublishedAnalysisStorageUsage } from "./storage-usage";

describe("PublishedAnalysisStorageUsage", () => {
  test("現在のstorage使用量を要求するspecを作る", () => {
    expect(PublishedAnalysisStorageUsage.Current()).toMatchObject({
      type: "Current",
    });
  });
});
