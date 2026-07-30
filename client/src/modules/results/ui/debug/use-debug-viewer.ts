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
  const initInFlight = useRef(false);
  const disposed = useRef(false);
  const [visibility, setVisibility] = useState(initialDebugOverlayVisibility);
  const visibilityRef = useRef(visibility);
  const [frameInfo, setFrameInfo] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  const navigate = useCallback((action: DebugFrameNavigationAction) => {
    const viewer = session.current;
    if (!viewer) return;
    void viewer.navigate(action).catch((cause) => {
      setError(errorMessage(cause));
    });
  }, []);

  const setOverlayVisibility = useCallback(
    (key: keyof DebugOverlayVisibility, enabled: boolean) => {
      const next = { ...visibilityRef.current, [key]: enabled };
      visibilityRef.current = next;
      setVisibility(next);
      const viewer = session.current;
      if (!viewer) return;
      void viewer.setVisibility(next).catch((cause) => {
        setError(errorMessage(cause));
      });
    },
    [],
  );

  const saveCurrentFrame = useCallback(() => {
    session.current?.saveCurrentFrame();
  }, []);

  useEffect(() => {
    disposed.current = false;
    return () => {
      disposed.current = true;
      session.current?.destroy();
      session.current = null;
    };
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
    if (!options.active || !canvas || session.current || initInFlight.current) {
      return;
    }

    initInFlight.current = true;
    setLoading(true);
    setError("");
    void (async () => {
      try {
        const videoArrayBuffer = options.result.sampleData
          ? await options.file.arrayBuffer()
          : null;
        const initialVisibility = visibilityRef.current;
        const viewer = await createDebugViewerSession({
          canvas,
          data: {
            file: options.file,
            timeline: options.result.timeline,
            hpFeatures: options.result.hpFeatures,
            trackedInputs: options.result.trackedInputs,
            attackInfo: options.result.attackInfo,
            frameCount: options.result.frameCount,
            frameTimestamps: options.result.frameTimestamps,
            sampleData: options.result.sampleData,
            videoArrayBuffer,
            codecConfig: options.result.codecConfig,
            frameToSampleIndex: options.result.frameToSampleIdx,
          },
          ownSide: options.side,
          visibility: initialVisibility,
          frameSourceFactory: debugFrameSourceFactory,
          frameInspector: debugFrameInspector,
          onFrameInfo(label) {
            if (!disposed.current) setFrameInfo(label);
          },
          onError(cause) {
            if (!disposed.current) setError(errorMessage(cause));
          },
        });
        if (disposed.current) {
          viewer.destroy();
          return;
        }
        session.current = viewer;
        if (visibilityRef.current !== initialVisibility) {
          await viewer.setVisibility(visibilityRef.current);
        }
      } catch (cause) {
        if (!disposed.current) setError(errorMessage(cause));
      } finally {
        if (!disposed.current) setLoading(false);
        initInFlight.current = false;
      }
    })();
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
