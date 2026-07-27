import { describe, expect, test } from "bun:test";
import {
  deleteCredentialCandidates,
  generateDeleteCode,
  isGeneratedDeleteCode,
  normalizeDeleteCredential,
} from "./delete-code";
import { isValidDeletePassword } from "./share";

describe("share delete code", () => {
  test("60 bitの乱数を読みやすい12文字のコードにする", () => {
    let requestedLength = 0;
    const code = generateDeleteCode((values) => {
      requestedLength = values.length;
      values.set(Array.from({ length: values.length }, (_, index) => index));
      return values;
    });

    expect(requestedLength).toBe(12);
    expect(code).toBe("2345-6789-ABCD");
    expect(isGeneratedDeleteCode(code)).toBe(true);
    expect(isValidDeletePassword(code)).toBe(true);
    expect(code).not.toMatch(/[01IO]/u);
  });

  test("発行済みコードだけを大文字へ正規化する", () => {
    expect(normalizeDeleteCredential("abcd-efgh-jklm")).toBe("ABCD-EFGH-JKLM");
    expect(normalizeDeleteCredential(" legacy password ")).toBe(
      " legacy password ",
    );
    expect(normalizeDeleteCredential("abcdefghijkl")).toBe("abcdefghijkl");
    expect(deleteCredentialCandidates("abcd-efgh-jklm")).toEqual([
      "abcd-efgh-jklm",
      "ABCD-EFGH-JKLM",
    ]);
    expect(deleteCredentialCandidates(" legacy password ")).toEqual([
      " legacy password ",
    ]);
    expect(isGeneratedDeleteCode("ABCD-EFGH-JKLM")).toBe(true);
    expect(isGeneratedDeleteCode("ABCI-EFGH-JKLM")).toBe(false);
    expect(isGeneratedDeleteCode("xABCD-EFGH-JKLM")).toBe(false);
    expect(isGeneratedDeleteCode("ABCD-EFGH-JKLMx")).toBe(false);
  });
});
