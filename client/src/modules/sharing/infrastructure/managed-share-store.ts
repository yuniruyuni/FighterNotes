import type { ManagedShareRepository } from "../application/ports.js";
import {
  ManagedShare,
  type ManagedShare as ManagedShareRecord,
  type ManagedShareSnapshot,
} from "../domain/managed-share.js";
import { isShareId } from "../domain/share.js";

const STORAGE_PREFIX = "fighter-notes:managed-share:v1:";

export interface ManagedShareStorage {
  readonly length: number;
  getItem(key: string): string | null;
  key(index: number): string | null;
  removeItem(key: string): void;
  setItem(key: string, value: string): void;
}

export function saveManagedShare(
  share: ManagedShareRecord,
  storage = browserStorage(),
  now = new Date(),
): boolean {
  const record = ManagedShare.store(share);
  if (!record || !storage || ManagedShare.isExpired(record, now)) {
    return false;
  }

  try {
    if (!loadManagedShares(storage, now).available) return false;
    storage.setItem(storageKey(record.id), JSON.stringify(record));
    return true;
  } catch {
    return false;
  }
}

export function loadManagedShares(
  storage = browserStorage(),
  now = new Date(),
): ManagedShareSnapshot {
  if (!storage) return { available: false, shares: [] };

  try {
    const shares: ManagedShareRecord[] = [];
    const keys = managedKeys(storage);
    for (const key of keys) {
      const stored = readRecord(storage.getItem(key));
      const expectedId = key.slice(STORAGE_PREFIX.length);
      if (
        !stored ||
        stored.id !== expectedId ||
        ManagedShare.isExpired(stored, now)
      ) {
        storage.removeItem(key);
        continue;
      }
      shares.push(stored);
    }
    shares.sort((left, right) => right.createdAt.localeCompare(left.createdAt));
    return { available: true, shares };
  } catch {
    return { available: false, shares: [] };
  }
}

export function removeManagedShare(
  id: string,
  storage = browserStorage(),
): boolean {
  if (!storage || !isShareId(id)) return false;
  try {
    storage.removeItem(storageKey(id));
    return true;
  } catch {
    return false;
  }
}

export function isManagedShareStorageKey(key: string | null): boolean {
  return key === null || key.startsWith(STORAGE_PREFIX);
}

export const browserManagedShareRepository: ManagedShareRepository = {
  save: (share, now) => saveManagedShare(share, undefined, now),
  load: (now) => loadManagedShares(undefined, now),
  remove: removeManagedShare,
  subscribe(listener) {
    const handleStorage = (event: StorageEvent) => {
      if (isManagedShareStorageKey(event.key)) listener();
    };
    window.addEventListener("storage", handleStorage);
    return () => window.removeEventListener("storage", handleStorage);
  },
};

function managedKeys(storage: ManagedShareStorage): string[] {
  const keys: string[] = [];
  for (let index = 0; index < storage.length; index += 1) {
    const key = storage.key(index);
    if (key?.startsWith(STORAGE_PREFIX)) keys.push(key);
  }
  return keys;
}

function readRecord(value: string | null) {
  if (!value) return undefined;
  try {
    return ManagedShare.parse(JSON.parse(value));
  } catch {
    return undefined;
  }
}

function storageKey(id: string): string {
  return `${STORAGE_PREFIX}${id}`;
}

function browserStorage(): ManagedShareStorage | undefined {
  try {
    return typeof localStorage === "undefined" ? undefined : localStorage;
  } catch {
    return undefined;
  }
}
