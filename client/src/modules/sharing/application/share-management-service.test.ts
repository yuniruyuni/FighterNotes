import { describe, expect, mock, test } from "bun:test";
import type { ManagedShare } from "../domain/managed-share.js";
import type { SharingServices } from "./ports.js";
import {
  deletePublicationWithCredential,
  deleteStoredPublication,
} from "./share-management-service.js";

function servicesFor(
  deleteShare: SharingServices["gateway"]["delete"],
  remove = mock(() => true),
): SharingServices {
  return {
    gateway: {
      create: async () => {
        throw new Error("unused");
      },
      delete: deleteShare,
      errorMessage: () => "error",
    },
    managedShares: {
      save: () => false,
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
    generateDeleteCode: () => "ABCD-EFGH-JKLM",
    now: () => new Date("2026-07-22T00:00:00.000Z"),
  };
}

const share: ManagedShare = {
  id: "Abcdefghijklmnopqrstu_",
  deleteCode: "ABCD-EFGH-JKLM",
  createdAt: "2026-07-22T00:00:00.000Z",
  expiresAt: "2026-08-22T00:00:00.000Z",
  label: "JURI vs KEN",
};

describe("share management service", () => {
  test("保存済み共有をremoteとlocalから削除する", async () => {
    const deleteShare = mock(async () => undefined);
    const remove = mock(() => true);

    await expect(
      deleteStoredPublication(share, servicesFor(deleteShare, remove)),
    ).resolves.toBe(true);
    expect(deleteShare).toHaveBeenCalledWith(
      { id: share.id },
      share.deleteCode,
    );
    expect(remove).toHaveBeenCalledWith(share.id);
  });

  test("小文字の発行codeは元入力失敗後に正規化候補で再試行する", async () => {
    const calls: string[] = [];
    const deleteShare = mock(
      async (_share: { id: string }, credential: string) => {
        calls.push(credential);
        if (calls.length === 1) throw new Error("first failed");
      },
    );

    await deletePublicationWithCredential(
      share.id,
      "abcd-efgh-jklm",
      servicesFor(deleteShare),
    );
    expect(calls).toEqual(["abcd-efgh-jklm", "ABCD-EFGH-JKLM"]);
    expect(deleteShare).toHaveBeenNthCalledWith(
      1,
      { id: share.id },
      "abcd-efgh-jklm",
    );
    expect(deleteShare).toHaveBeenNthCalledWith(
      2,
      { id: share.id },
      "ABCD-EFGH-JKLM",
    );
  });

  test("全credential候補が失敗したら最後のerrorを返す", async () => {
    const lastError = new Error("delete failed");
    const deleteShare = mock(async () => {
      throw lastError;
    });

    await expect(
      deletePublicationWithCredential(
        share.id,
        "legacy-password",
        servicesFor(deleteShare),
      ),
    ).rejects.toBe(lastError);
    expect(deleteShare).toHaveBeenCalledTimes(1);
  });
});
