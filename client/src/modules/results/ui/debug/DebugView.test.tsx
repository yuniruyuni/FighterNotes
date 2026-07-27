import { describe, expect, mock, test } from "bun:test";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { syntheticAnalysisResult } from "~/test-support/analysis.js";
import type { DebugFrameInspector } from "../../application/debug-frame-inspection.js";
import type { DebugFrameSource } from "../../application/debug-frame-source.js";
import type { ResultsServices } from "../../application/ports.js";
import { ResultsServicesProvider } from "../ResultsServicesProvider.js";
import { DebugView } from "./DebugView.js";

describe("DebugView navigation", () => {
  test("UI入力を意味のあるviewer操作へ変換する", async () => {
    const user = userEvent.setup();
    const seekFallback = mock(() => {});
    const destroy = mock(() => {});
    const source: DebugFrameSource = {
      fallbackSource: document.createElement("canvas"),
      usesExactFrames: false,
      initialize: async () => {},
      decode: async () => null,
      seekFallback,
      destroy,
    };
    const create = mock(() => source);
    const inspector: DebugFrameInspector = {
      initialize: async () => ({
        p1: parallelogram(),
        p2: parallelogram(),
      }),
      inspectMeter: unexpectedInspection,
      inspectHp: unexpectedInspection,
      inspectDrive: unexpectedInspection,
      inspectInput: unexpectedInspection,
    };
    const services: ResultsServices = {
      debugFrameInspector: inspector,
      debugFrameSourceFactory: { create },
      history: {
        save: async () => {},
        load: async () => [],
      },
    };
    const result = {
      ...syntheticAnalysisResult(),
      frameCount: 100,
      sampleData: null,
    };
    const getContext = Object.getOwnPropertyDescriptor(
      HTMLCanvasElement.prototype,
      "getContext",
    );
    Object.defineProperty(HTMLCanvasElement.prototype, "getContext", {
      configurable: true,
      value: () => ({}) as CanvasRenderingContext2D,
    });

    try {
      render(
        <ResultsServicesProvider services={services}>
          <DebugView
            active
            file={new File(["video"], "replay.mp4", { type: "video/mp4" })}
            result={result}
            side="p1"
          />
        </ResultsServicesProvider>,
      );
      await waitFor(() => expect(create).toHaveBeenCalledTimes(1));
      expect(seekFallback).toHaveBeenLastCalledWith(0);

      await user.click(screen.getByRole("button", { name: "60フレーム進む" }));
      expect(seekFallback).toHaveBeenLastCalledWith(60);

      fireEvent.keyDown(window, { key: "ArrowLeft", shiftKey: true });
      expect(seekFallback).toHaveBeenLastCalledWith(50);

      const hpToggle = screen.getByRole("checkbox", { name: "HP" });
      fireEvent.keyDown(hpToggle, { key: "ArrowLeft" });
      expect(seekFallback).toHaveBeenCalledTimes(3);

      const canvas = document.querySelector("canvas");
      expect(canvas).not.toBeNull();
      const toBlob = mock((callback: BlobCallback) => callback(null));
      Object.defineProperty(canvas!, "toBlob", { value: toBlob });
      fireEvent.keyDown(window, { key: "s" });
      expect(toBlob).toHaveBeenCalledTimes(1);
    } finally {
      if (getContext) {
        Object.defineProperty(
          HTMLCanvasElement.prototype,
          "getContext",
          getContext,
        );
      } else {
        Reflect.deleteProperty(HTMLCanvasElement.prototype, "getContext");
      }
    }
  });
});

function parallelogram() {
  return {
    top_left: { x: 0, y: 0 },
    top_right: { x: 1, y: 0 },
    bottom_right: { x: 1, y: 1 },
    bottom_left: { x: 0, y: 1 },
  };
}

function unexpectedInspection(): never {
  throw new Error("inspection should not run in this interaction test");
}
