import { describe, expect, test } from "bun:test";
import type { AdviceCard } from "~/modules/analysis/contracts.js";
import { WorkspaceNavigation } from "./workspace-navigation.js";

const card = (evidence: AdviceCard["evidence"]): AdviceCard =>
  ({ id: "anti_air", title: "対空", evidence }) as AdviceCard;

describe("WorkspaceNavigation", () => {
  test("summaryを初期表示し、summaryとdebugへ切り替える", () => {
    const initial = WorkspaceNavigation.initial();
    expect(initial).toEqual({
      view: "summary",
      selected: "summary",
      scene: null,
      nextSceneKey: 1,
    });
    expect(
      WorkspaceNavigation.reduce(
        { ...initial, view: "video", selected: "card-1" },
        { type: "summary" },
      ),
    ).toMatchObject({ view: "summary", selected: "summary" });
    expect(
      WorkspaceNavigation.reduce(initial, { type: "debug" }),
    ).toMatchObject({ view: "debug", selected: "debug" });
  });

  test("openSceneを直接使って選択状態を保ったままsceneを開く", () => {
    const initial = { ...WorkspaceNavigation.initial(), selected: "card-4" };
    expect(
      WorkspaceNavigation.openScene(initial, {
        frame: 240,
        card: null,
        label: "確認",
      }),
    ).toMatchObject({
      view: "video",
      selected: "card-4",
      nextSceneKey: 2,
      scene: { key: 1, frame: 240, label: "確認" },
    });
  });

  test("React reducerとしてthisなしで詳細シーンを開く", () => {
    const reducer = WorkspaceNavigation.reduce;

    expect(
      reducer(WorkspaceNavigation.initial(), {
        type: "card",
        index: 1,
        card: card([{ frame: 90, end_frame: 120, label: "詳細" }]),
      }),
    ).toMatchObject({
      view: "video",
      selected: "card-1",
      scene: { frame: 90, endFrame: 120, key: 1 },
    });
  });

  test("証拠のあるカードは連番付きシーンを開く", () => {
    const first = WorkspaceNavigation.reduce(WorkspaceNavigation.initial(), {
      type: "card",
      index: 2,
      card: card([{ frame: 120, end_frame: 180, label: "場面" }]),
    });
    const second = WorkspaceNavigation.reduce(first, {
      type: "scene",
      scene: { frame: 300, card: null },
    });

    expect(first).toMatchObject({
      view: "video",
      selected: "card-2",
      scene: { frame: 120, endFrame: 180, key: 1 },
    });
    expect(second.scene?.key).toBe(2);
  });

  test("証拠のないカードはサマリーに留まる", () => {
    expect(
      WorkspaceNavigation.reduce(WorkspaceNavigation.initial(), {
        type: "card",
        index: 0,
        card: card([]),
      }),
    ).toMatchObject({ view: "summary", selected: "card-0", scene: null });
  });
});
