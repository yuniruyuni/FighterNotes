import { useCallback, useEffect, useRef, useState } from "react";
import type {
  AnalysisResult,
  AnalysisSide,
} from "~/modules/analysis/contracts.js";
import type { DebugFrameNavigationAction } from "../../domain/debug-frame-navigation.js";
import { useResultsServices } from "../ResultsServicesProvider.js";
import { navigationActionForKey } from "./debug-frame-shortcuts.js";
import {
  type DebugOverlayVisibility,
  initialDebugOverlayVisibility,
} from "./debug-viewer-model.js";
import {
  createDebugViewerSession,
  type DebugViewerSession,
} from "./debug-viewer-session.js";

interface DebugViewerOptions {
  active: boolean;
  file: File;
  result: AnalysisResult;
  side: AnalysisSide;
}

export function useDebugViewer(options: DebugViewerOptions) {
  const { debugFrameInspector, debugFrameSourceFactory } = useResultsServices();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const session = useRef<DebugViewerSession | null>(null);
  const generation = useRef(0);
  const [visibility, setVisibility] = useState(initialDebugOverlayVisibility);
  const visibilityRef = useRef(visibility);
  const [frameInfo, setFrameInfo] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const navigate = useCallback((action: DebugFrameNavigationAction) => {
    const viewer = session.current;
    if (!viewer) return;
    const requestGeneration = generation.current;
    void viewer.navigate(action).catch((cause) => {
      if (generation.current === requestGeneration) {
        setError(errorMessage(cause));
      }
    });
  }, []);

  const setOverlayVisibility = useCallback(
    (key: keyof DebugOverlayVisibility, enabled: boolean) => {
      const next = { ...visibilityRef.current, [key]: enabled };
      visibilityRef.current = next;
      setVisibility(next);
      const viewer = session.current;
      if (!viewer) return;
      const requestGeneration = generation.current;
      void viewer.setVisibility(next).catch((cause) => {
        if (generation.current === requestGeneration) {
          setError(errorMessage(cause));
        }
      });
    },
    [],
  );

  const saveCurrentFrame = useCallback(() => {
    session.current?.saveCurrentFrame();
  }, []);

  useEffect(() => {
    if (!options.active) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (isFormControl(event.target)) return;
      const action = navigationActionForKey(event.key, {
        ctrl: event.ctrlKey,
        shift: event.shiftKey,
      });
      if (action) {
        navigate(action);
      } else if (event.key === "s") {
        saveCurrentFrame();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [navigate, options.active, saveCurrentFrame]);

  useEffect(() => {
    const canvas = canvasRef.current;
    const currentGeneration = generation.current + 1;
    generation.current = currentGeneration;
    if (!options.active || !canvas) {
      const viewer = session.current;
      session.current = null;
      viewer?.destroy();
      setLoading(false);
      return;
    }

    const controller = new AbortController();
    let viewer: DebugViewerSession | null = null;
    const isCurrent = () =>
      generation.current === currentGeneration && !controller.signal.aborted;
    setLoading(true);
    setError("");
    setFrameInfo("");
    void (async () => {
      try {
        const data = {
          file: options.file,
          timeline: options.result.timeline,
          hpFeatures: options.result.hpFeatures,
          trackedInputs: options.result.trackedInputs,
          attackInfo: options.result.attackInfo,
          frameCount: options.result.frameCount,
          frameTimestamps: options.result.frameTimestamps,
          sampleData: options.result.sampleData,
          videoArrayBuffer: options.result.sampleData
            ? await readFileBuffer(options.file, controller.signal)
            : null,
          codecConfig: options.result.codecConfig,
          frameToSampleIndex: options.result.frameToSampleIdx,
        };
        const initialVisibility = visibilityRef.current;
        const created = await createDebugViewerSession({
          canvas,
          data,
          ownSide: options.side,
          signal: controller.signal,
          visibility: initialVisibility,
          frameSourceFactory: debugFrameSourceFactory,
          frameInspector: debugFrameInspector,
          onFrameInfo(label) {
            if (isCurrent()) setFrameInfo(label);
          },
          onError(cause) {
            if (isCurrent()) setError(errorMessage(cause));
          },
        });
        if (!isCurrent()) {
          created.destroy();
          return;
        }
        viewer = created;
        session.current = viewer;
        if (visibilityRef.current !== initialVisibility) {
          await viewer.setVisibility(visibilityRef.current);
        }
      } catch (cause) {
        if (isCurrent()) setError(errorMessage(cause));
      } finally {
        if (isCurrent()) setLoading(false);
      }
    })();

    return () => {
      if (generation.current === currentGeneration) generation.current += 1;
      if (session.current === viewer) session.current = null;
      controller.abort(
        new DOMException("認識デバッグを終了しました", "AbortError"),
      );
      viewer?.destroy();
    };
  }, [
    debugFrameInspector,
    debugFrameSourceFactory,
    options.active,
    options.file,
    options.result,
    options.side,
  ]);

  return {
    canvasRef,
    frameInfo,
    visibility,
    setOverlayVisibility,
    navigate,
    loading,
    error,
  };
}

function isFormControl(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLInputElement || target instanceof HTMLSelectElement
  );
}

function errorMessage(cause: unknown): string {
  return `エラー: ${cause instanceof Error ? cause.message : String(cause)}`;
}

function readFileBuffer(file: File, signal: AbortSignal): Promise<ArrayBuffer> {
  return new Promise<ArrayBuffer>((resolve, reject) => {
    const reader = new FileReader();
    let settled = false;
    const cleanup = () => {
      signal.removeEventListener("abort", onAbort);
      reader.onload = null;
      reader.onerror = null;
      reader.onabort = null;
    };
    const settle = (callback: () => void) => {
      if (settled) return;
      settled = true;
      cleanup();
      callback();
    };
    const onAbort = () => {
      if (reader.readyState === FileReader.LOADING) reader.abort();
      settle(() => reject(abortReason(signal)));
    };
    reader.onload = () => {
      const result = reader.result;
      settle(() => {
        if (result instanceof ArrayBuffer) resolve(result);
        else reject(new Error("デバッグ動画を読み込めませんでした"));
      });
    };
    reader.onerror = () =>
      settle(() =>
        reject(reader.error ?? new Error("デバッグ動画を読み込めませんでした")),
      );
    reader.onabort = onAbort;
    signal.addEventListener("abort", onAbort, { once: true });
    if (signal.aborted) {
      onAbort();
      return;
    }
    try {
      reader.readAsArrayBuffer(file);
    } catch (cause) {
      settle(() => reject(cause));
    }
  });
}

function abortReason(signal: AbortSignal): Error {
  return signal.reason instanceof Error
    ? signal.reason
    : new DOMException("認識デバッグを終了しました", "AbortError");
}
