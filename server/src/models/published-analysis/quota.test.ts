import { describe, expect, test } from "bun:test";
import {
  evaluatePublishedAnalysisCreateQuota,
  MAX_ANALYSIS_STORAGE_RESERVATION_BYTES,
  PUBLISHED_ANALYSIS_CREATE_LOCK,
} from "./quota";

const limits = {
  dailyCreates: 100,
  activeRows: 1_000,
  storageBytes: 10_000_000,
};

describe("evaluatePublishedAnalysisCreateQuota", () => {
  test("storage reservationとtransaction lockを安定した値で共有する", () => {
    expect(MAX_ANALYSIS_STORAGE_RESERVATION_BYTES).toBe(262_144);
    expect(PUBLISHED_ANALYSIS_CREATE_LOCK).toEqual({
      namespace: 1_179_537_442,
      id: 1,
    });
    expect(Object.isFrozen(PUBLISHED_ANALYSIS_CREATE_LOCK)).toBe(true);
  });

  test("各hard limitに到達したcreateを拒否する", () => {
    expect(
      evaluatePublishedAnalysisCreateQuota(
        { dailyCreates: 100, activeRows: 0, storageBytes: 0 },
        limits,
      ),
    ).toEqual({ allowed: false, reason: "daily" });
    expect(
      evaluatePublishedAnalysisCreateQuota(
        { dailyCreates: 0, activeRows: 1_000, storageBytes: 0 },
        limits,
      ),
    ).toEqual({ allowed: false, reason: "active" });
    expect(
      evaluatePublishedAnalysisCreateQuota(
        {
          dailyCreates: 0,
          activeRows: 0,
          storageBytes:
            limits.storageBytes - MAX_ANALYSIS_STORAGE_RESERVATION_BYTES + 1,
        },
        limits,
      ),
    ).toEqual({ allowed: false, reason: "storage" });
  });

  test("全limitにreservation込みのheadroomがあれば許可する", () => {
    expect(
      evaluatePublishedAnalysisCreateQuota(
        {
          dailyCreates: 99,
          activeRows: 999,
          storageBytes:
            limits.storageBytes - MAX_ANALYSIS_STORAGE_RESERVATION_BYTES,
        },
        limits,
      ),
    ).toEqual({ allowed: true });
  });
});
