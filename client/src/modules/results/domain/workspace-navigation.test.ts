import { describe, expect, test } from "bun:test";
import type { AdviceCard } from "~/modules/analysis/contracts.js";
import { WorkspaceNavigation } from "./workspace-navigation.js";

const card = (evidence: AdviceCard["evidence"]): AdviceCard =>
  ({ id: "anti_air", title: "対空", evidence }) as AdviceCard;

describe("WorkspaceNavigation", () => {
  test("summaryを初期表示し、summary・video・debugへ切り替える", () => {
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
    expect(
      WorkspaceNavigation.reduce(initial, { type: "video" }),
    ).toMatchObject({ view: "video", selected: "video" });
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

  test("sceneの表示元を明示して現在のnavigation項目を更新する", () => {
    expect(
      WorkspaceNavigation.reduce(WorkspaceNavigation.initial(), {
        type: "scene",
        selected: "video",
        scene: { frame: 240, card: null, label: "ラウンド開始" },
      }),
    ).toMatchObject({
      view: "video",
      selected: "video",
      scene: { frame: 240, label: "ラウンド開始" },
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

describe("WorkspaceNavigation.location", () => {
  test("history へ残せるよう、cardをidだけにしたscene位置を作る", () => {
    const opened = WorkspaceNavigation.reduce(WorkspaceNavigation.initial(), {
      type: "card",
      index: 1,
      card: card([{ frame: 90, end_frame: 120, label: "詳細" }]),
    });

    expect(WorkspaceNavigation.location(opened)).toEqual({
      view: "video",
      selected: "card-1",
      scene: {
        frame: 90,
        cardId: "anti_air",
        label: undefined,
        endFrame: 120,
      },
    });
  });

  test("cardのないsceneとsceneのない位置をそれぞれ残す", () => {
    const labeled = WorkspaceNavigation.openScene(
      WorkspaceNavigation.initial(),
      { frame: 240, card: null, label: "ラウンド 1 開始" },
    );

    expect(WorkspaceNavigation.location(labeled).scene).toEqual({
      frame: 240,
      cardId: null,
      label: "ラウンド 1 開始",
      endFrame: undefined,
    });
    expect(
      WorkspaceNavigation.location(
        WorkspaceNavigation.reduce(labeled, { type: "debug" }),
      ),
    ).toEqual({ view: "debug", selected: "debug", scene: null });
  });
});

describe("WorkspaceNavigation.sameLocation", () => {
  const base = {
    view: "video",
    selected: "card-1",
    scene: { frame: 90, cardId: "anti_air", label: "詳細", endFrame: 120 },
  } as const;

  test("同じ位置を同一と見なす", () => {
    expect(
      WorkspaceNavigation.sameLocation(base, {
        ...base,
        scene: { ...base.scene },
      }),
    ).toBe(true);
    expect(
      WorkspaceNavigation.sameLocation(
        { view: "summary", selected: "summary", scene: null },
        { view: "summary", selected: "summary", scene: null },
      ),
    ).toBe(true);
  });

  test("view・選択項目・sceneのいずれかが違えば別の位置とする", () => {
    const differences = [
      { ...base, view: "summary" as const },
      { ...base, selected: "video" },
      { ...base, scene: { ...base.scene, frame: 91 } },
      { ...base, scene: { ...base.scene, cardId: "big_hits" } },
      { ...base, scene: { ...base.scene, label: "別の場面" } },
      { ...base, scene: { ...base.scene, endFrame: 121 } },
      { ...base, scene: null },
    ];

    for (const difference of differences) {
      expect(WorkspaceNavigation.sameLocation(base, difference)).toBe(false);
      expect(WorkspaceNavigation.sameLocation(difference, base)).toBe(false);
    }
  });
});

describe("WorkspaceNavigation.restore", () => {
  const cards = [card([{ frame: 90, end_frame: 120, label: "詳細" }])];

  test("位置のないentryは初期位置へ戻す", () => {
    const opened = WorkspaceNavigation.reduce(WorkspaceNavigation.initial(), {
      type: "card",
      index: 0,
      card: cards[0] as AdviceCard,
    });

    expect(WorkspaceNavigation.restore(opened, null, cards)).toMatchObject({
      view: "summary",
      selected: "summary",
      scene: null,
      nextSceneKey: 2,
    });
  });

  test("sceneのない位置は表示中のsceneを閉じて戻す", () => {
    const opened = WorkspaceNavigation.reduce(WorkspaceNavigation.initial(), {
      type: "card",
      index: 0,
      card: cards[0] as AdviceCard,
    });

    expect(
      WorkspaceNavigation.restore(
        opened,
        { view: "debug", selected: "debug", scene: null },
        cards,
      ),
    ).toMatchObject({ view: "debug", selected: "debug", scene: null });
  });

  test("scene位置はcard idから復元し、新しいkeyで再生位置を取り直させる", () => {
    expect(
      WorkspaceNavigation.restore(
        { ...WorkspaceNavigation.initial(), nextSceneKey: 4 },
        {
          view: "video",
          selected: "card-0",
          scene: {
            frame: 90,
            cardId: "anti_air",
            label: "詳細",
            endFrame: 120,
          },
        },
        cards,
      ),
    ).toEqual({
      view: "video",
      selected: "card-0",
      scene: {
        frame: 90,
        card: cards[0] as AdviceCard,
        label: "詳細",
        endFrame: 120,
        key: 4,
      },
      nextSceneKey: 5,
    });
  });

  test("該当するcardがないscene位置はcardなしで復元する", () => {
    expect(
      WorkspaceNavigation.restore(
        WorkspaceNavigation.initial(),
        {
          view: "video",
          selected: "video",
          scene: { frame: 240, cardId: "removed" },
        },
        cards,
      ).scene,
    ).toMatchObject({ frame: 240, card: null, key: 1 });
  });
});
