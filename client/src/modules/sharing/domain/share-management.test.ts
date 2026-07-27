import { describe, expect, test } from "bun:test";
import { isShareId } from "./share";
import {
  isShareManagementPath,
  managementPathId,
  ShareManagement,
  shareIdFromReference,
} from "./share-management";

const id = "Abcdefghijklmnopqrstu_";
const origin = "https://fighter.example";

describe("share management model", () => {
  test("共有IDを厳密に検証する", () => {
    expect(isShareId(id)).toBe(true);
    expect(isShareId(`${id}x`)).toBe(false);
    expect(isShareId("../not-a-share-id")).toBe(false);
  });

  test("管理一覧と個別管理pathを識別する", () => {
    expect(isShareManagementPath("/manage")).toBe(true);
    expect(isShareManagementPath(`/manage/${id}`)).toBe(true);
    expect(isShareManagementPath("/s/example")).toBe(false);
    expect(isShareManagementPath("/management")).toBe(false);
    expect(managementPathId(`/manage/${id}`)).toBe(id);
    expect(managementPathId("/manage/not-valid")).toBeUndefined();
    expect(managementPathId(`/manage/${id}/extra`)).toBeUndefined();
    expect(managementPathId(`/prefix/manage/${id}`)).toBeUndefined();
  });

  test("同一originの共有URLまたは厳密なIDだけを受理する", () => {
    expect(shareIdFromReference(id, origin)).toBe(id);
    expect(shareIdFromReference(`${origin}/s/${id}`, origin)).toBe(id);
    expect(
      shareIdFromReference(`https://attacker.example/s/${id}`, origin),
    ).toBeUndefined();
    expect(
      shareIdFromReference(`${origin}/s/${id}?unexpected=1`, origin),
    ).toBeUndefined();
    expect(
      shareIdFromReference(`${origin}/s/${id}#fragment`, origin),
    ).toBeUndefined();
    expect(
      shareIdFromReference(`https://user@fighter.example/s/${id}`, origin),
    ).toBeUndefined();
    expect(
      shareIdFromReference(`https://user:pass@fighter.example/s/${id}`, origin),
    ).toBeUndefined();
    expect(shareIdFromReference(`  ${id}  `, origin)).toBe(id);
    expect(shareIdFromReference("not-valid", origin)).toBeUndefined();
    expect(
      shareIdFromReference(`${origin}/prefix/s/${id}`, origin),
    ).toBeUndefined();
    expect(shareIdFromReference(`${origin}/x/${id}`, origin)).toBeUndefined();
    expect(
      shareIdFromReference(`${origin}/s/${id}/extra`, origin),
    ).toBeUndefined();
  });

  test("手動削除要求を副作用なしで検証する", () => {
    expect(
      ShareManagement.manualDeletionRequest(
        `${origin}/s/${id}`,
        "DELETE-CODE1",
        origin,
      ),
    ).toEqual({ valid: true, id, credential: "DELETE-CODE1" });
    expect(ShareManagement.manualDeletionRequest(id, "short", origin)).toEqual({
      valid: false,
      feedback: {
        message:
          "削除コードまたは以前設定した削除用パスワードを12文字以上128文字以下で入力してください。",
        tone: "error",
      },
    });
    expect(
      ShareManagement.manualDeletionRequest(
        "not-valid",
        "DELETE-CODE1",
        origin,
      ),
    ).toMatchObject({
      valid: false,
      feedback: { message: expect.stringContaining("共有URL"), tone: "error" },
    });
    expect(ShareManagement.emptyFeedback()).toEqual({ message: "", tone: "" });
    expect(
      ShareManagement.snapshotFeedback({ available: false, shares: [] }),
    ).toEqual({
      message:
        "このブラウザの保存領域を読み込めません。下のフォームから削除コードを入力してください。",
      tone: "error",
    });
    expect(
      ShareManagement.snapshotFeedback({ available: true, shares: [] }),
    ).toEqual({
      message: "このブラウザに保存された共有はありません。",
      tone: "",
    });
    expect(
      ShareManagement.snapshotFeedback({
        available: true,
        shares: [
          {
            id,
            deleteCode: "ABCD-EFGH-JKLM",
            createdAt: "2026-07-22T00:00:00.000Z",
            expiresAt: "2026-08-22T00:00:00.000Z",
            label: "JURI vs KEN",
          },
        ],
      }),
    ).toEqual({ message: "", tone: "" });
  });
});
