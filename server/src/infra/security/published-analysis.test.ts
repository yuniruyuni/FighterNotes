import { describe, expect, test } from "bun:test";
import type { DeletePasswordHash } from "../../models/published-analysis";
import {
  parseDeletePassword,
  parseShareId,
} from "../../models/published-analysis";
import { SecurityCapacityError } from "../../usecases/services";
import {
  createPublishedAnalysisSecurity,
  publishedAnalysisSecurity,
} from "./published-analysis";

describe("publishedAnalysisSecurity", () => {
  test("128-bit IDとsalt付きArgon2idハッシュを生成する", async () => {
    const password = parseDeletePassword("fighter-notes-delete-key");
    if (!password) throw new Error("invalid fixture");
    const firstId = publishedAnalysisSecurity.generateShareId();
    const secondId = publishedAnalysisSecurity.generateShareId();
    const hash = await publishedAnalysisSecurity.hashDeletePassword(password);

    expect(parseShareId(firstId)).toBe(firstId);
    expect(secondId).not.toBe(firstId);
    expect(hash).toStartWith("$argon2id$");
    expect(
      await publishedAnalysisSecurity.verifyDeletePassword(password, hash),
    ).toBe(true);
  });

  test("誤ったパスワードと壊れたハッシュを不一致として扱う", async () => {
    const password = parseDeletePassword("fighter-notes-delete-key");
    const wrong = parseDeletePassword("wrong-delete-password");
    if (!password || !wrong) throw new Error("invalid fixture");
    const hash = await publishedAnalysisSecurity.hashDeletePassword(password);

    expect(
      await publishedAnalysisSecurity.verifyDeletePassword(wrong, hash),
    ).toBe(false);
    expect(
      await publishedAnalysisSecurity.verifyDeletePassword(
        password,
        "not-a-password-hash" as DeletePasswordHash,
      ),
    ).toBe(false);
  });

  test("Argon2 hashとverifyの合計同時実行数を制限する", async () => {
    const password = parseDeletePassword("fighter-notes-delete-key");
    if (!password) throw new Error("invalid fixture");
    let active = 0;
    let maximumActive = 0;
    const operation = async () => {
      active += 1;
      maximumActive = Math.max(maximumActive, active);
      await Bun.sleep(5);
      active -= 1;
      return "fixture-hash" as DeletePasswordHash;
    };
    const security = createPublishedAnalysisSecurity({
      concurrency: 2,
      queueLimit: 8,
      waitMillis: 100,
      hashPassword: operation,
      verifyPassword: async () => {
        await operation();
        return true;
      },
    });

    await Promise.all([
      security.hashDeletePassword(password),
      security.verifyDeletePassword(password, "fixture" as DeletePasswordHash),
      security.hashDeletePassword(password),
      security.verifyDeletePassword(password, "fixture" as DeletePasswordHash),
      security.hashDeletePassword(password),
      security.hashDeletePassword(password),
    ]);
    expect(maximumActive).toBe(2);
  });

  test("待機時間とqueueを超えたArgon2要求をfail closedにする", async () => {
    const password = parseDeletePassword("fighter-notes-delete-key");
    if (!password) throw new Error("invalid fixture");
    let release: (() => void) | undefined;
    const blocked = new Promise<void>((resolve) => {
      release = resolve;
    });
    const security = createPublishedAnalysisSecurity({
      concurrency: 1,
      queueLimit: 1,
      waitMillis: 5,
      hashPassword: async () => {
        await blocked;
        return "fixture-hash" as DeletePasswordHash;
      },
    });

    const active = security.hashDeletePassword(password);
    await Bun.sleep(0);
    const waiting = security.hashDeletePassword(password);
    const overflow = security.hashDeletePassword(password);
    await expect(overflow).rejects.toBeInstanceOf(SecurityCapacityError);
    await expect(waiting).rejects.toBeInstanceOf(SecurityCapacityError);
    release?.();
    await active;
  });
});
