import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useReducer,
  useRef,
} from "react";
import { flushSync } from "react-dom";
import type { AnalysisServices } from "../application/ports.js";
import { runAnalysis } from "../application/run-analysis.js";
import {
  AnalysisCanceledError,
  isAnalysisCanceled,
} from "../domain/analysis-cancellation.js";
import {
  AnalysisSession,
  type AnalysisSessionState,
  type CompletedAnalysis,
} from "../domain/analysis-session.js";
import type { AnalysisSide } from "../domain/context.js";
import type { AnalysisRuntimeReadiness } from "../domain/runtime.js";
import { videoPreflightFailure } from "../domain/video-preflight.js";
import {
  AnalysisProgressReporter,
  analysisProgressStage,
} from "./analysis-progress-reporter.js";

interface AnalysisSessionValue {
  state: AnalysisSessionState;
  runtime: AnalysisRuntimeReadiness;
  setFile(file: File | null): void;
  setSide(side: AnalysisSide): void;
  setOwnCharacter(character: string): void;
  setOpponentCharacter(character: string): void;
  analyze(): Promise<CompletedAnalysis | null>;
  cancel(): void;
  reset(): void;
}

interface ActiveAnalysisRun {
  readonly id: number;
  readonly controller: AbortController;
  readonly progress: AnalysisProgressReporter;
}

interface ActivePreflightRun {
  readonly id: number;
  readonly file: File;
  readonly controller: AbortController;
}

const AnalysisSessionContext = createContext<AnalysisSessionValue | null>(null);

export function AnalysisSessionProvider({
  children,
  services,
}: {
  children: ReactNode;
  services: AnalysisServices;
}) {
  const runtime = useMemo(() => services.engine.readiness(), [services]);
  const [state, dispatch] = useReducer(
    AnalysisSession.reduce,
    undefined,
    AnalysisSession.initial,
  );
  const activeRun = useRef<ActiveAnalysisRun | null>(null);
  const activePreflight = useRef<ActivePreflightRun | null>(null);
  const nextRunId = useRef(1);
  const nextPreflightId = useRef(1);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    const confirmNavigation = (event: BeforeUnloadEvent) => {
      if (!activeRun.current) return;
      event.preventDefault();
      event.returnValue = "";
    };
    window.addEventListener("beforeunload", confirmNavigation);
    return () => {
      mounted.current = false;
      window.removeEventListener("beforeunload", confirmNavigation);
      const run = activeRun.current;
      run?.progress.dispose();
      run?.controller.abort(new AnalysisCanceledError());
      activePreflight.current?.controller.abort(
        new Error("動画の事前確認を中止しました"),
      );
    };
  }, []);

  const setFile = useCallback(
    (file: File | null) => {
      activePreflight.current?.controller.abort(
        new Error("別の動画が選択されました"),
      );
      activePreflight.current = null;
      dispatch({ type: "file", file });
      if (!file) return;
      if (!runtime.available) {
        dispatch({
          type: "videoPreflightInvalid",
          failure: videoPreflightFailure("frame_extraction", runtime.reason),
        });
        return;
      }

      const preflight: ActivePreflightRun = {
        id: nextPreflightId.current++,
        file,
        controller: new AbortController(),
      };
      activePreflight.current = preflight;
      void services.engine
        .preflight(file, preflight.controller.signal)
        .then((result) => {
          if (
            !mounted.current ||
            preflight.controller.signal.aborted ||
            activePreflight.current?.id !== preflight.id ||
            activePreflight.current.file !== file
          ) {
            return;
          }
          if (result.status === "valid") {
            dispatch({ type: "videoPreflightValid", video: result.video });
          } else {
            dispatch({ type: "videoPreflightInvalid", failure: result });
          }
        })
        .catch((error: unknown) => {
          if (
            !mounted.current ||
            preflight.controller.signal.aborted ||
            activePreflight.current?.id !== preflight.id
          ) {
            return;
          }
          const detail = error instanceof Error ? error.message : String(error);
          dispatch({
            type: "videoPreflightInvalid",
            failure: videoPreflightFailure(
              "invalid_mp4",
              `動画の事前確認に失敗しました。動画を選択し直してください。（詳細: ${detail}）`,
            ),
          });
        })
        .finally(() => {
          if (activePreflight.current?.id === preflight.id) {
            activePreflight.current = null;
          }
        });
    },
    [runtime, services],
  );
  const setSide = useCallback(
    (side: AnalysisSide) => dispatch({ type: "side", side }),
    [],
  );
  const setOwnCharacter = useCallback(
    (character: string) => dispatch({ type: "ownCharacter", character }),
    [],
  );
  const setOpponentCharacter = useCallback(
    (character: string) => dispatch({ type: "opponentCharacter", character }),
    [],
  );
  const reset = useCallback(() => dispatch({ type: "reset" }), []);

  const cancel = useCallback(() => {
    const run = activeRun.current;
    if (!run || run.controller.signal.aborted) return;
    run.progress.dispose();
    dispatch({ type: "cancel" });
    run.controller.abort(new AnalysisCanceledError());
  }, []);

  const analyze = useCallback(async (): Promise<CompletedAnalysis | null> => {
    if (activeRun.current) return null;
    const { file, side, ownCharacter, opponentCharacter, videoPreflight } =
      state;
    if (!runtime.available) {
      dispatch({ type: "fail", error: runtime.reason });
      return null;
    }
    if (
      !AnalysisSession.canStart(state) ||
      !file ||
      !side ||
      videoPreflight.status !== "valid"
    ) {
      return null;
    }

    const runId = nextRunId.current++;
    const run: ActiveAnalysisRun = {
      id: runId,
      controller: new AbortController(),
      progress: new AnalysisProgressReporter((progress, status) => {
        if (!mounted.current || activeRun.current?.id !== runId) return;
        dispatch({ type: "progress", progress, status });
      }),
    };
    activeRun.current = run;
    dispatch({ type: "start" });
    try {
      const completed = await runAnalysis(
        {
          file,
          validatedVideo: videoPreflight.video,
          side,
          ownCharacter,
          opponentCharacter,
        },
        run.progress.report,
        services,
        run.controller.signal,
      );
      if (
        run.controller.signal.aborted ||
        !mounted.current ||
        activeRun.current?.id !== run.id
      ) {
        if (mounted.current && activeRun.current?.id === run.id) {
          dispatch({ type: "canceled" });
        }
        return null;
      }
      flushSync(() => run.progress.finish());
      const { result, report, context } = completed;
      dispatch({ type: "complete", result, report, context });
      return completed;
    } catch (error) {
      if (!mounted.current || activeRun.current?.id !== run.id) return null;
      if (isAnalysisCanceled(error)) {
        dispatch({ type: "canceled" });
        return null;
      }
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "fail", error: `エラー: ${message}` });
      return null;
    } finally {
      run.progress.dispose();
      if (activeRun.current?.id === run.id) activeRun.current = null;
    }
  }, [runtime, services, state]);

  const value = useMemo(
    () => ({
      state,
      runtime,
      setFile,
      setSide,
      setOwnCharacter,
      setOpponentCharacter,
      analyze,
      cancel,
      reset,
    }),
    [
      state,
      runtime,
      setFile,
      setSide,
      setOwnCharacter,
      setOpponentCharacter,
      analyze,
      cancel,
      reset,
    ],
  );

  return (
    <AnalysisSessionContext.Provider value={value}>
      <span
        className="visually-hidden"
        role="status"
        aria-live="polite"
        aria-atomic="true"
      >
        {analysisStatusAnnouncement(state)}
      </span>
      {state.error && (
        <span
          className="visually-hidden"
          role="alert"
          aria-live="assertive"
          aria-atomic="true"
        >
          {state.error}
        </span>
      )}
      {children}
    </AnalysisSessionContext.Provider>
  );
}

function analysisStatusAnnouncement(state: AnalysisSessionState): string {
  if (state.error) return "";
  if (state.phase === "setup" && state.videoPreflight.status === "checking") {
    return "動画の形式と録画仕様を確認中です。";
  }
  if (state.phase === "setup" && state.videoPreflight.status === "valid") {
    return "動画の形式と録画仕様を確認しました。";
  }
  if (state.phase === "ready") return "動画解析が完了しました。";
  if (state.phase === "canceling") return "動画解析を中止しています。";
  if (state.phase === "canceled") return "動画解析を中止しました。";
  if (state.phase !== "analyzing") return "";
  switch (analysisProgressStage(state.status)) {
    case "frames":
      return "動画フレームを解析中です。";
    case "spatial":
      return "位置関係を確認中です。";
    case "report":
      return "解析レポートを生成中です。";
    case "complete":
      return "動画解析が完了しました。";
    default:
      return state.status;
  }
}

export function useAnalysisSession(): AnalysisSessionValue {
  const value = useContext(AnalysisSessionContext);
  if (!value) {
    throw new Error(
      "useAnalysisSession must be used within AnalysisSessionProvider",
    );
  }
  return value;
}
