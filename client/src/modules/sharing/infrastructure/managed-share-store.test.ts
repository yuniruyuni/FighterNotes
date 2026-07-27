import { describe, expect, test } from "bun:test";
import type { ManagedShare } from "../domain/managed-share";
import {
  loadManagedShares,
  type ManagedShareStorage,
  removeManagedShare,
  saveManagedShare,
} from "./managed-share-store";

const firstId = "Abcdefghijklmnopqrstu_";
const secondId = "Zbcdefghijklmnopqrstu_";
const now = new Date("2026-07-18T00:00:00.000Z");

class MemoryStorage implements ManagedShareStorage {
  readonly items = new Map<string, string>();

  get length(): number {
    return this.items.size;
  }

  getItem(key: string): string | null {
    return this.items.get(key) ?? null;
  }

  key(index: number): string | null {
    return [...this.items.keys()][index] ?? null;
  }

  removeItem(key: string): void {
    this.items.delete(key);
  }

  setItem(key: string, value: string): void {
    this.items.set(key, value);
  }
}

function managedShare(overrides: Partial<ManagedShare> = {}): ManagedShare {
  return {
    id: firstId,
    deleteCode: "2345-6789-ABCD",
    createdAt: "2026-07-16T00:00:00.000Z",
    expiresAt: "2026-08-15T00:00:00.000Z",
    label: "RYU vs KEN",
    ...overrides,
  };
}

describe("managed share store", () => {
  test("共有の削除に必要な最小情報だけを保存し、新しい順に返す", () => {
    const storage = new MemoryStorage();
    expect(saveManagedShare(managedShare(), storage, now)).toBe(true);
    expect(
      saveManagedShare(
        managedShare({
          id: secondId,
          createdAt: "2026-07-17T00:00:00.000Z",
          label: "KEN vs RYU",
        }),
        storage,
        now,
      ),
    ).toBe(true);

    const snapshot = loadManagedShares(storage, now);
    expect(snapshot.available).toBe(true);
    expect(snapshot.shares.map((share) => share.id)).toEqual([
      secondId,
      firstId,
    ]);
    const serialized = [...storage.items.values()].join("\n");
    expect(serialized).not.toContain("url");
    expect(serialized).not.toContain("report");
    expect(serialized).not.toContain("video");
    const stored = JSON.parse([...storage.items.values()][0] ?? "null");
    expect(Object.keys(stored).sort()).toEqual([
      "createdAt",
      "deleteCode",
      "expiresAt",
      "id",
      "label",
      "version",
    ]);
  });

  test("期限切れ・壊れたrecordを取り除き、他用途のkeyは保持する", () => {
    const storage = new MemoryStorage();
    saveManagedShare(
      managedShare({ expiresAt: "2026-07-16T00:00:00.000Z" }),
      storage,
      new Date("2026-07-15T00:00:00.000Z"),
    );
    storage.setItem("fighter-notes:managed-share:v1:broken", "not-json");
    storage.setItem("another-feature", "keep");

    const snapshot = loadManagedShares(
      storage,
      new Date("2026-07-16T00:00:00.000Z"),
    );
    expect(snapshot).toEqual({ available: true, shares: [] });
    expect(storage.getItem("another-feature")).toBe("keep");
    expect(storage.length).toBe(1);
  });

  test("削除成功後は該当recordだけを忘れる", () => {
    const storage = new MemoryStorage();
    saveManagedShare(managedShare(), storage, now);

    expect(removeManagedShare(firstId, storage)).toBe(true);
    expect(loadManagedShares(storage, now).shares).toEqual([]);
  });

  test("不正なrecordや利用不能な保存領域を安全に扱う", () => {
    const storage = new MemoryStorage();
    expect(
      saveManagedShare(managedShare({ deleteCode: "too-short" }), storage, now),
    ).toBe(false);
    expect(
      saveManagedShare(
        managedShare({ deleteCode: "abcdefghijkl" }),
        storage,
        now,
      ),
    ).toBe(false);

    const unavailable = {
      get length(): number {
        throw new Error("blocked");
      },
      getItem: () => null,
      key: () => null,
      removeItem: () => undefined,
      setItem: () => {
        throw new Error("blocked");
      },
    } satisfies ManagedShareStorage;
    expect(loadManagedShares(unavailable)).toEqual({
      available: false,
      shares: [],
    });
    expect(saveManagedShare(managedShare(), unavailable, now)).toBe(false);
  });
});
