import { describe, expect, test } from "bun:test";
import {
  assertDeletePassword,
  isShareId,
  isValidDeletePassword,
  MAX_DELETE_PASSWORD_LENGTH,
  MIN_DELETE_PASSWORD_LENGTH,
  shareIdFromPath,
  shareIdFromUrl,
} from "./share.js";

const id = "Abcdefghijklmnopqrstu_";

describe("published analysis share", () => {
  test("共有IDを閉じた形式で検証する", () => {
    expect(isShareId(id)).toBe(true);
    expect(isShareId(`${id}x`)).toBe(false);
    expect(isShareId("../not-a-share-id")).toBe(false);
  });

  test("削除credentialの長さと非空白条件を検証する", () => {
    expect(isValidDeletePassword("x".repeat(MIN_DELETE_PASSWORD_LENGTH))).toBe(
      true,
    );
    expect(isValidDeletePassword("x".repeat(MAX_DELETE_PASSWORD_LENGTH))).toBe(
      true,
    );
    expect(
      isValidDeletePassword("x".repeat(MIN_DELETE_PASSWORD_LENGTH - 1)),
    ).toBe(false);
    expect(
      isValidDeletePassword("x".repeat(MAX_DELETE_PASSWORD_LENGTH + 1)),
    ).toBe(false);
    expect(isValidDeletePassword(" ".repeat(MIN_DELETE_PASSWORD_LENGTH))).toBe(
      false,
    );
    expect(() => assertDeletePassword("valid-password")).not.toThrow();
    expect(() => assertDeletePassword("short")).toThrow(
      /12文字以上128文字以下/,
    );
  });

  test("共有pathから厳密なIDだけを抽出する", () => {
    expect(shareIdFromPath(`/s/${id}`)).toBe(id);
    for (const path of [
      `/s/${id}/`,
      `/s/${id}/extra`,
      `/s/invalid`,
      `/x/${id}`,
      `/prefix/s/${id}`,
    ]) {
      expect(shareIdFromPath(path)).toBeUndefined();
    }
  });

  test("HTTP系の汚染されていない共有URLだけを受理する", () => {
    expect(shareIdFromUrl(new URL(`https://fighter.example/s/${id}`))).toBe(id);
    expect(shareIdFromUrl(new URL(`http://localhost:3001/s/${id}`))).toBe(id);
    for (const value of [
      `ftp://fighter.example/s/${id}`,
      `https://user@fighter.example/s/${id}`,
      `https://user:pass@fighter.example/s/${id}`,
      `https://fighter.example/s/${id}?query=1`,
      `https://fighter.example/s/${id}#fragment`,
      `https://fighter.example/s/${id}/extra`,
      `https://fighter.example/prefix/s/${id}`,
      "https://fighter.example/s/invalid",
    ]) {
      expect(shareIdFromUrl(new URL(value))).toBeUndefined();
    }
  });
});
