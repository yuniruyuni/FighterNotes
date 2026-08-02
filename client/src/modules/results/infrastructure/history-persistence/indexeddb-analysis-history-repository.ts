import type { AnalysisHistoryRepository } from "../../application/ports.js";
import {
  ANALYSIS_HISTORY_ID_PREFIX,
  type AnalysisHistoryRecord,
} from "../../domain/history.js";
import {
  loadAnalysisHistorySavingPreference,
  saveAnalysisHistorySavingPreference,
} from "./analysis-history-preference.js";

const DATABASE_NAME = "fighter-notes";
const DATABASE_VERSION = 2;
const STORE_NAME = "analysis-history";
const MAX_RECORDS = 200;

function createOpaqueHistoryId(): string {
  const random = crypto.getRandomValues(new Uint8Array(32));
  return `${ANALYSIS_HISTORY_ID_PREFIX}${Array.from(random, (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("")}`;
}

function replaceLegacyHistoryIds(store: IDBObjectStore): void {
  const request = store.openCursor();
  request.onsuccess = () => {
    const cursor = request.result;
    if (!cursor) return;
    const record = cursor.value as AnalysisHistoryRecord;
    if (!record.id.startsWith(ANALYSIS_HISTORY_ID_PREFIX)) {
      cursor.delete();
      store.put({ ...record, id: createOpaqueHistoryId() });
    }
    cursor.continue();
  };
}

function openDatabase(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DATABASE_NAME, DATABASE_VERSION);
    request.onupgradeneeded = (event) => {
      const database = request.result;
      if (!database.objectStoreNames.contains(STORE_NAME)) {
        database.createObjectStore(STORE_NAME, { keyPath: "id" });
      } else if (event.oldVersion < DATABASE_VERSION) {
        const transaction = request.transaction;
        if (transaction) {
          replaceLegacyHistoryIds(transaction.objectStore(STORE_NAME));
        }
      }
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

function requestResult<T>(request: IDBRequest<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

function transactionDone(transaction: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    transaction.oncomplete = () => resolve();
    transaction.onerror = () => reject(transaction.error);
    transaction.onabort = () => reject(transaction.error);
  });
}

export async function saveAnalysisHistoryRecord(
  record: AnalysisHistoryRecord,
): Promise<void> {
  if (
    typeof indexedDB === "undefined" ||
    !loadAnalysisHistorySavingPreference().enabled
  ) {
    return;
  }
  const database = await openDatabase();
  try {
    const transaction = database.transaction(STORE_NAME, "readwrite");
    transaction.objectStore(STORE_NAME).put(record);
    await transactionDone(transaction);

    const records = await loadAnalysisHistory(database);
    if (records.length > MAX_RECORDS) {
      const stale = records
        .sort((left, right) => right.createdAt.localeCompare(left.createdAt))
        .slice(MAX_RECORDS);
      const prune = database.transaction(STORE_NAME, "readwrite");
      const store = prune.objectStore(STORE_NAME);
      for (const item of stale) store.delete(item.id);
      await transactionDone(prune);
    }
  } finally {
    database.close();
  }
}

export const browserAnalysisHistoryRepository: AnalysisHistoryRepository = {
  save: saveAnalysisHistoryRecord,
  load: loadAnalysisHistory,
  delete: deleteAnalysisHistoryRecord,
  clear: clearAnalysisHistory,
  getSavingPreference: async () => loadAnalysisHistorySavingPreference(),
  setSavingEnabled: async (enabled) =>
    saveAnalysisHistorySavingPreference(enabled),
};

export async function loadAnalysisHistory(
  existingDatabase?: IDBDatabase,
): Promise<AnalysisHistoryRecord[]> {
  if (typeof indexedDB === "undefined") return [];
  const database = existingDatabase ?? (await openDatabase());
  try {
    const transaction = database.transaction(STORE_NAME, "readonly");
    return await requestResult(
      transaction.objectStore(STORE_NAME).getAll() as IDBRequest<
        AnalysisHistoryRecord[]
      >,
    );
  } finally {
    if (!existingDatabase) database.close();
  }
}

export async function deleteAnalysisHistoryRecord(id: string): Promise<void> {
  if (typeof indexedDB === "undefined") return;
  const database = await openDatabase();
  try {
    const transaction = database.transaction(STORE_NAME, "readwrite");
    transaction.objectStore(STORE_NAME).delete(id);
    await transactionDone(transaction);
  } finally {
    database.close();
  }
}

export async function clearAnalysisHistory(): Promise<void> {
  if (typeof indexedDB === "undefined") return;
  const database = await openDatabase();
  try {
    const transaction = database.transaction(STORE_NAME, "readwrite");
    transaction.objectStore(STORE_NAME).clear();
    await transactionDone(transaction);
  } finally {
    database.close();
  }
}
