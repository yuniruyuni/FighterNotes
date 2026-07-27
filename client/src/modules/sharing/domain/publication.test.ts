import { describe, expect, test } from "bun:test";
import { Publication, type PublicationSource } from "./publication.js";

const source = {
  context: { ownSide: "p1", p1: {}, p2: {} },
  deleteCode: "ABCD-EFGH-JKLM",
  report: {},
} as PublicationSource;

const published = {
  id: "Abcdefghijklmnopqrstu_",
  url: "https://fighter.example/s/Abcdefghijklmnopqrstu_",
  expiresAt: "2026-08-17T00:00:00.000Z",
};

describe("publication reducer", () => {
  test("初期状態と再試行・削除の実行条件を判定する", () => {
    const initial = Publication.initial();
    expect(initial).toEqual({
      phase: "idle",
      storedLocally: false,
      status: "",
      tone: "",
    });
    expect(Publication.canRetry(initial)).toBe(false);
    expect(Publication.canDelete(initial)).toBe(false);

    const prepared = Publication.reduce(initial, { type: "prepare", source });
    expect(prepared).toEqual({
      source,
      phase: "idle",
      storedLocally: false,
      status: "共有URLを準備しています。",
      tone: "pending",
    });
    expect(Publication.canRetry(prepared)).toBe(true);
    expect(Publication.canRetry({ ...prepared, phase: "creating" })).toBe(
      false,
    );
    expect(Publication.canRetry({ ...prepared, phase: "deleting" })).toBe(
      false,
    );

    const created = Publication.reduce(prepared, {
      type: "created",
      published,
      storedLocally: false,
    });
    expect(Publication.canRetry(created)).toBe(false);
    expect(Publication.canDelete(created)).toBe(true);
    expect(Publication.canDelete({ ...created, phase: "deleting" })).toBe(
      false,
    );
    expect(Publication.canDelete({ ...created, source: undefined })).toBe(
      false,
    );
    expect(Publication.canDelete({ ...created, published: undefined })).toBe(
      false,
    );
  });

  test("作成中から公開済みへ遷移する", () => {
    const creating = Publication.reduce(Publication.initial(), {
      type: "creating",
      source,
    });
    const created = Publication.reduce(creating, {
      type: "created",
      published,
      storedLocally: true,
    });

    expect(creating).toEqual({
      source,
      published: undefined,
      phase: "creating",
      storedLocally: false,
      status: "共有URLを準備しています。",
      tone: "pending",
    });
    expect(created).toEqual({
      source,
      published,
      phase: "idle",
      storedLocally: true,
      status:
        "公開URLを準備しました。この端末では動画付きの解析画面を表示しています。",
      tone: "success",
    });
  });

  test("削除後も再公開に使う解析結果を保持する", () => {
    const creating = Publication.reduce(Publication.initial(), {
      type: "creating",
      source,
    });
    const deleted = Publication.reduce(creating, {
      type: "deleted",
      removedLocally: true,
    });

    expect(deleted.phase).toBe("deleted");
    expect(deleted.source).toBe(source);
    expect(deleted.published).toBeUndefined();
    expect(deleted.storedLocally).toBe(false);
    expect(deleted.status).toBe(
      "共有結果を削除しました。新しいアクセスには約15秒以内に反映されます。",
    );
    expect(deleted.tone).toBe("success");
  });

  test("失敗・削除・feedback・resetの全actionを反映する", () => {
    const created = Publication.reduce(
      Publication.reduce(Publication.initial(), { type: "prepare", source }),
      { type: "created", published, storedLocally: true },
    );
    const failed = Publication.reduce(created, {
      type: "failed",
      message: "作成失敗",
    });
    expect(failed).toMatchObject({
      phase: "failed",
      published: undefined,
      status: "作成失敗",
      tone: "error",
    });

    const deleting = Publication.reduce(created, { type: "deleting" });
    expect(deleting).toMatchObject({
      phase: "deleting",
      status: "共有結果を削除しています。",
      tone: "pending",
    });
    expect(Publication.canRetry(deleting)).toBe(false);

    const deleteFailed = Publication.reduce(deleting, { type: "deleteFailed" });
    expect(deleteFailed).toMatchObject({
      phase: "idle",
      status: "共有結果を削除できませんでした。",
      tone: "error",
    });

    const retained = Publication.reduce(created, {
      type: "deleted",
      removedLocally: false,
    });
    expect(retained).toMatchObject({
      phase: "deleted",
      storedLocally: false,
      status:
        "共有結果を削除しました。この端末の管理一覧に表示が残る場合があります。",
      tone: "error",
    });

    const feedback = Publication.reduce(retained, {
      type: "feedback",
      message: "copied",
      tone: "success",
    });
    expect(feedback).toMatchObject({ status: "copied", tone: "success" });
    expect(Publication.reduce(feedback, { type: "reset" })).toEqual(
      Publication.initial(),
    );
  });
});
