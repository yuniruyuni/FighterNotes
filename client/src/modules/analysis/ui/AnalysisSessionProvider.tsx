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
  const nextRunId = useRef(1);
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
      activeRun.current?.controller.abort(new AnalysisCanceledError());
    };
  }, []);

  const setFile = useCallback(
    (file: File | null) => dispatch({ type: "file", file }),
    [],
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
    dispatch({ type: "cancel" });
    run.controller.abort(new AnalysisCanceledError());
  }, []);

  const analyze = useCallback(async (): Promise<CompletedAnalysis | null> => {
    if (activeRun.current) return null;
    const { file, side, ownCharacter, opponentCharacter } = state;
    if (!runtime.available) {
      dispatch({ type: "fail", error: runtime.reason });
      return null;
    }
    if (!AnalysisSession.canStart(state) || !file || !side) {
      return null;
    }

    const run: ActiveAnalysisRun = {
      id: nextRunId.current++,
      controller: new AbortController(),
    };
    activeRun.current = run;
    dispatch({ type: "start" });
    try {
      const completed = await runAnalysis(
        {
          file,
          side,
          ownCharacter,
          opponentCharacter,
        },
        (progress, status) => dispatch({ type: "progress", progress, status }),
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
      {children}
    </AnalysisSessionContext.Provider>
  );
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
