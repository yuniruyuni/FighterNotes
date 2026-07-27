import { describe, expect, test } from "bun:test";
import type { DeletePasswordHash } from "../../models/published-analysis";
import {
  parseDeletePassword,
  parseShareId,
} from "../../models/published-analysis";
import { publishedAnalysisSecurity } from "./published-analysis";

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
});
