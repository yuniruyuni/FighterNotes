import { describe, expect, test } from "bun:test";
import type {
  AttackInfoObservation,
  HpFrameData,
  TrackedInputRow,
} from "~/modules/analysis/contracts.js";
import {
  type DebugViewerData,
  frameDataAt,
  initialDebugOverlayVisibility,
} from "./debug-viewer-model.js";

describe("debug viewer model", () => {
  test("overlayをすべて非表示の独立した状態で初期化する", () => {
    const first = initialDebugOverlayVisibility();
    const second = initialDebugOverlayVisibility();

    expect(first).toEqual({
      raw: false,
      hue: false,
      hp: false,
      drive: false,
      super: false,
      input: false,
      attackInfo: false,
    });
    expect(first).not.toBe(second);
  });
});

const inputRow = (dir: string) =>
  ({
    count: 1,
    dir,
    badges: "",
    auto: false,
    throw: false,
    repaired: false,
    uncertain: false,
  }) as TrackedInputRow;

function viewerData(overrides: Partial<DebugViewerData> = {}): DebugViewerData {
  return {
    file: new File([new Uint8Array(1)], "replay.mp4"),
    timeline: {
      left: { side: "left", segments: [] },
      right: { side: "right", segments: [] },
      video_map: { "3": [7, 421] },
    },
    hpFeatures: [],
    trackedInputs: null,
    attackInfo: [],
    frameCount: 10,
    frameTimestamps: [0, 1 / 60, 2 / 60, 3 / 60],
    sampleData: null,
    codecConfig: null,
    frameToSampleIndex: null,
    ...overrides,
  };
}

describe("frameDataAt", () => {
  test("表示中フレームの認識結果だけを取り出す", () => {
    const hp = { frame_index: 3, fps: 60, own_hp: 0.5 } as HpFrameData;
    const attack = { frame_index: 3 } as AttackInfoObservation;
    const data = viewerData({
      hpFeatures: [{} as HpFrameData, {} as HpFrameData, {} as HpFrameData, hp],
      trackedInputs: {
        p1: [inputRow("5"), inputRow("6"), inputRow("2"), inputRow("4")],
        p2: [inputRow("5"), inputRow("5"), inputRow("5"), inputRow("6")],
      },
      attackInfo: [{ frame_index: 1 } as AttackInfoObservation, attack],
    });

    expect(frameDataAt(data, 3)).toEqual({
      frame: 3,
      timeSeconds: 3 / 60,
      timeline: { segmentId: 7, gameFrame: 421 },
      hp,
      input: { p1: inputRow("4"), p2: inputRow("6") },
      attackInfo: attack,
    });
  });

  test("対応する記録が無いフレームはnullで埋める", () => {
    expect(frameDataAt(viewerData(), 9)).toEqual({
      frame: 9,
      timeSeconds: null,
      timeline: null,
      hp: null,
      input: { p1: null, p2: null },
      attackInfo: null,
    });
  });
});
