import { describe, expect, mock, test } from "bun:test";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { syntheticAnalysisResult } from "~/test-support/analysis.js";
import type { DebugFrameInspector } from "../../application/debug-frame-inspection.js";
import type {
  DebugFrameSource,
  DebugFrameSourceData,
} from "../../application/debug-frame-source.js";
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
    const inspector = debugInspector();
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
      expect(screen.getByRole("checkbox", { name: "SA" })).not.toBeNull();

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

  test("inactiveで初期decodeを破棄し、大容量bufferを解放して再初期化する", async () => {
    const decoded = deferred<VideoFrame | null>();
    const closeDecodedFrame = mock(() => {});
    const sourceData: DebugFrameSourceData[] = [];
    const destroys: Array<ReturnType<typeof mock>> = [];
    const seekFallback = mock(() => {});
    let sourceIndex = 0;
    const create = mock((data: DebugFrameSourceData): DebugFrameSource => {
      sourceData.push(data);
      const index = sourceIndex;
      sourceIndex += 1;
      const destroy = mock(() => {});
      destroys.push(destroy);
      return {
        fallbackSource: document.createElement("canvas"),
        usesExactFrames: index === 0,
        initialize: async () => {},
        decode: () => decoded.promise,
        seekFallback,
        destroy,
      };
    });
    const services = debugServices(create);
    const result = {
      ...syntheticAnalysisResult(),
      frameCount: 100,
      sampleData: [],
    };
    const file = new File([new Uint8Array(8 * 1024 * 1024)], "replay.mp4", {
      type: "video/mp4",
    });
    const drawImage = mock(() => {});
    const restoreCanvas = installCanvasContext({ drawImage });

    try {
      const view = render(
        <ResultsServicesProvider services={services}>
          <DebugView active file={file} result={result} side="p1" />
        </ResultsServicesProvider>,
      );
      await waitFor(() => expect(create).toHaveBeenCalledTimes(1));
      expect(sourceData[0].videoArrayBuffer?.byteLength).toBe(file.size);

      view.rerender(
        <ResultsServicesProvider services={services}>
          <DebugView active={false} file={file} result={result} side="p1" />
        </ResultsServicesProvider>,
      );
      expect(destroys[0]).toHaveBeenCalledTimes(1);
      expect(sourceData[0].videoArrayBuffer).toBeNull();

      view.rerender(
        <ResultsServicesProvider services={services}>
          <DebugView active file={file} result={result} side="p1" />
        </ResultsServicesProvider>,
      );
      await waitFor(() => expect(create).toHaveBeenCalledTimes(2));
      expect(seekFallback).toHaveBeenCalledWith(0);
      expect(sourceData[1].videoArrayBuffer?.byteLength).toBe(file.size);

      await act(async () => {
        decoded.resolve({ close: closeDecodedFrame } as unknown as VideoFrame);
        await Promise.resolve();
      });
      await waitFor(() => expect(closeDecodedFrame).toHaveBeenCalledTimes(1));
      expect(drawImage).not.toHaveBeenCalled();
      expect(destroys[0]).toHaveBeenCalledTimes(1);

      view.unmount();
      expect(destroys[1]).toHaveBeenCalledTimes(1);
      expect(sourceData[1].videoArrayBuffer).toBeNull();
    } finally {
      restoreCanvas();
    }
  });

  test("inactiveで進行中の動画buffer読込をabortし、遅延完了を無視する", async () => {
    const originalFileReader = Object.getOwnPropertyDescriptor(
      globalThis,
      "FileReader",
    );
    Object.defineProperty(globalThis, "FileReader", {
      configurable: true,
      writable: true,
      value: ControlledFileReader,
    });
    ControlledFileReader.instances = [];
    const destroy = mock(() => {});
    const create = mock(
      (): DebugFrameSource => ({
        fallbackSource: document.createElement("canvas"),
        usesExactFrames: false,
        initialize: async () => {},
        decode: async () => null,
        seekFallback: () => {},
        destroy,
      }),
    );
    const services = debugServices(create);
    const result = { ...syntheticAnalysisResult(), sampleData: [] };
    const file = new File(["video"], "replay.mp4", { type: "video/mp4" });
    const restoreCanvas = installCanvasContext({});

    try {
      const view = render(
        <ResultsServicesProvider services={services}>
          <DebugView active file={file} result={result} side="p1" />
        </ResultsServicesProvider>,
      );
      await waitFor(() =>
        expect(ControlledFileReader.instances).toHaveLength(1),
      );

      view.rerender(
        <ResultsServicesProvider services={services}>
          <DebugView active={false} file={file} result={result} side="p1" />
        </ResultsServicesProvider>,
      );
      expect(ControlledFileReader.instances[0].abortCount).toBe(1);
      ControlledFileReader.instances[0].resolve(
        new ArrayBuffer(8 * 1024 * 1024),
      );
      await Promise.resolve();
      expect(create).not.toHaveBeenCalled();
      expect(screen.queryByText(/エラー:/)).toBeNull();

      view.rerender(
        <ResultsServicesProvider services={services}>
          <DebugView active file={file} result={result} side="p1" />
        </ResultsServicesProvider>,
      );
      await waitFor(() =>
        expect(ControlledFileReader.instances).toHaveLength(2),
      );
      await act(async () => {
        ControlledFileReader.instances[1].resolve(
          new ArrayBuffer(8 * 1024 * 1024),
        );
        await Promise.resolve();
      });
      await waitFor(() => expect(create).toHaveBeenCalledTimes(1));

      view.unmount();
      expect(destroy).toHaveBeenCalledTimes(1);
    } finally {
      restoreCanvas();
      if (originalFileReader) {
        Object.defineProperty(globalThis, "FileReader", originalFileReader);
      }
    }
  });
});

function debugInspector(): DebugFrameInspector {
  return {
    initialize: async () => ({
      p1: parallelogram(),
      p2: parallelogram(),
    }),
    inspectMeter: unexpectedInspection,
    inspectHp: unexpectedInspection,
    inspectDrive: unexpectedInspection,
    inspectSuper: unexpectedInspection,
    inspectInput: unexpectedInspection,
    inspectAttackInfo: unexpectedInspection,
  };
}

function debugServices(
  create: ResultsServices["debugFrameSourceFactory"]["create"],
): ResultsServices {
  return {
    debugFrameInspector: debugInspector(),
    debugFrameSourceFactory: { create },
    history: {
      save: async () => {},
      load: async () => [],
    },
  };
}

function installCanvasContext(context: object): () => void {
  const descriptor = Object.getOwnPropertyDescriptor(
    HTMLCanvasElement.prototype,
    "getContext",
  );
  Object.defineProperty(HTMLCanvasElement.prototype, "getContext", {
    configurable: true,
    value: () => context as CanvasRenderingContext2D,
  });
  return () => {
    if (descriptor) {
      Object.defineProperty(
        HTMLCanvasElement.prototype,
        "getContext",
        descriptor,
      );
    } else {
      Reflect.deleteProperty(HTMLCanvasElement.prototype, "getContext");
    }
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

class ControlledFileReader {
  static readonly EMPTY = 0;
  static readonly LOADING = 1;
  static readonly DONE = 2;
  static instances: ControlledFileReader[] = [];
  readyState = ControlledFileReader.EMPTY;
  result: string | ArrayBuffer | null = null;
  error: DOMException | null = null;
  onload: ((event: ProgressEvent<FileReader>) => void) | null = null;
  onerror: ((event: ProgressEvent<FileReader>) => void) | null = null;
  onabort: ((event: ProgressEvent<FileReader>) => void) | null = null;
  abortCount = 0;

  constructor() {
    ControlledFileReader.instances.push(this);
  }

  readAsArrayBuffer(): void {
    this.readyState = ControlledFileReader.LOADING;
  }

  abort(): void {
    this.abortCount += 1;
    this.readyState = ControlledFileReader.DONE;
    this.onabort?.({} as ProgressEvent<FileReader>);
  }

  resolve(value: ArrayBuffer): void {
    this.readyState = ControlledFileReader.DONE;
    this.result = value;
    this.onload?.({} as ProgressEvent<FileReader>);
  }
}

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
