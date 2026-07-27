import { describe, expect, test } from "bun:test";
import { MANAGED_SHARE_RECORD_VERSION, ManagedShare } from "./managed-share.js";

const valid = {
  id: "Abcdefghijklmnopqrstu_",
  deleteCode: "ABCD-EFGH-JKLM",
  createdAt: "2026-07-22T00:00:00.000Z",
  expiresAt: "2026-08-22T00:00:00.000Z",
  label: "JURI vs KEN",
};

describe("managed share", () => {
  test("保存形式へversionを付け、未知fieldを持たない値へ正規化する", () => {
    expect(ManagedShare.store(valid)).toEqual({
      ...valid,
      version: MANAGED_SHARE_RECORD_VERSION,
    });
    expect(
      ManagedShare.parse({
        ...valid,
        version: MANAGED_SHARE_RECORD_VERSION,
        ignored: "value",
      }),
    ).toEqual({ ...valid, version: MANAGED_SHARE_RECORD_VERSION });
  });

  test("不正な保存値を全fieldで拒否する", () => {
    const stored = { ...valid, version: MANAGED_SHARE_RECORD_VERSION };
    const functionRecord = Object.assign(() => undefined, stored);
    const invalid: unknown[] = [
      null,
      [],
      functionRecord,
      { ...stored, version: 2 },
      { ...stored, id: "invalid" },
      { ...stored, id: { toString: () => valid.id } },
      { ...stored, deleteCode: "invalid-code" },
      { ...stored, deleteCode: { toString: () => valid.deleteCode } },
      { ...stored, label: "" },
      { ...stored, label: 42 },
      { ...stored, label: "x".repeat(81) },
      { ...stored, createdAt: 123 },
      { ...stored, createdAt: { toString: () => valid.createdAt } },
      {
        ...stored,
        createdAt: {
          toString: () => {
            throw new Error("must not coerce dates");
          },
        },
      },
      { ...stored, createdAt: "2026-07-22" },
      { ...stored, createdAt: "not-a-dateZ" },
      { ...stored, createdAt: "2026-07-22T00:00:00Z" },
      { ...stored, expiresAt: 123 },
      { ...stored, expiresAt: "not-a-date" },
    ];

    for (const value of invalid)
      expect(ManagedShare.parse(value)).toBeUndefined();
  });

  test("80文字のlabelを受理し、81文字から拒否する", () => {
    expect(
      ManagedShare.parse({
        ...valid,
        version: MANAGED_SHARE_RECORD_VERSION,
        label: "x".repeat(80),
      })?.label,
    ).toHaveLength(80);
    expect(
      ManagedShare.parse({
        ...valid,
        version: MANAGED_SHARE_RECORD_VERSION,
        label: "x".repeat(81),
      }),
    ).toBeUndefined();
  });

  test("期限時刻を含めて期限切れと判定する", () => {
    expect(
      ManagedShare.isExpired(valid, new Date("2026-08-21T23:59:59.999Z")),
    ).toBe(false);
    expect(
      ManagedShare.isExpired(valid, new Date("2026-08-22T00:00:00.000Z")),
    ).toBe(true);
  });
});
