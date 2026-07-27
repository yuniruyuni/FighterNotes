import { describe, expect, test } from "bun:test";
import type { PublishedAnalysisCandidate } from "../domain/published-analysis";
import {
  createPublishedAnalysisShare,
  createShareErrorMessage,
  deletePublishedAnalysisShare,
  isValidDeletePassword,
  parseCreateResponse,
  type ShareTransport,
} from "./share-api";

const candidate = {} as PublishedAnalysisCandidate;
const id = "Abcdefghijklmnopqrstu_";
const deletePassword = "fighter-notes-delete-key";

class FakeTransport implements ShareTransport {
  calls: Array<{ path: string; input: unknown }> = [];

  constructor(private readonly response: unknown) {}

  async mutation(path: string, input: unknown): Promise<unknown> {
    this.calls.push({ path, input });
    return this.response;
  }
}

describe("share API", () => {
  test("作成候補をtRPCへ渡し、検証済み共有情報を返す", async () => {
    const transport = new FakeTransport({
      url: `https://fighter.example/s/${id}`,
      expiresAt: "2027-07-13T00:00:00.000Z",
    });

    const result = await createPublishedAnalysisShare(
      candidate,
      deletePassword,
      transport,
    );

    expect(transport.calls).toEqual([
      {
        path: "publishedAnalysis.create",
        input: { analysis: candidate, deletePassword },
      },
    ]);
    expect(result).toEqual({
      id,
      url: `https://fighter.example/s/${id}`,
      expiresAt: "2027-07-13T00:00:00.000Z",
    });
  });

  test("共有IDと削除キーだけを削除mutationへ渡す", async () => {
    const transport = new FakeTransport({ deleted: true });

    await deletePublishedAnalysisShare({ id }, deletePassword, transport);

    expect(transport.calls).toEqual([
      {
        path: "publishedAnalysis.delete",
        input: { id, deletePassword },
      },
    ]);
  });

  test("削除キーを12〜128文字に制限する", () => {
    expect(isValidDeletePassword(deletePassword)).toBe(true);
    expect(isValidDeletePassword("too-short")).toBe(false);
    expect(isValidDeletePassword(" ".repeat(12))).toBe(false);
    expect(isValidDeletePassword("x".repeat(129))).toBe(false);
  });

  test("不正なURLと期限を拒否する", () => {
    const base = {
      url: `https://fighter.example/s/${id}`,
      expiresAt: "2027-07-13T00:00:00.000Z",
    };
    expect(() =>
      parseCreateResponse({ ...base, url: "javascript:alert(1)" }),
    ).toThrow("共有URLが不正です");
    expect(() =>
      parseCreateResponse({ ...base, url: `${base.url}?x=1` }),
    ).toThrow("共有URLが不正です");
    expect(() =>
      parseCreateResponse({ ...base, expiresAt: "invalid" }),
    ).toThrow("共有期限が不正です");
  });

  test("作成失敗の原因に応じて利用者向け文言を分ける", () => {
    expect(
      createShareErrorMessage({ data: { code: "BAD_REQUEST" } }),
    ).toContain("共有形式に対応していません");
    expect(
      createShareErrorMessage({ data: { code: "TOO_MANY_REQUESTS" } }),
    ).toContain("1分ほど待って");
    expect(createShareErrorMessage(new Error("network"))).toContain(
      "時間を置いて",
    );
  });
});
