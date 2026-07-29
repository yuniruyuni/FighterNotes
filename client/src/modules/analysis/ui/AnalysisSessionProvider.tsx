import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useMemo,
  useReducer,
} from "react";
import type { AnalysisServices } from "../application/ports.js";
import { runAnalysis } from "../application/run-analysis.js";
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
  reset(): void;
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

  const analyze = useCallback(async (): Promise<CompletedAnalysis | null> => {
    const { file, side, ownCharacter, opponentCharacter } = state;
    if (!runtime.available) {
      dispatch({ type: "fail", error: runtime.reason });
      return null;
    }
    if (!AnalysisSession.canStart(state) || !file || !side) {
      return null;
    }

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
      );
      const { result, report, context } = completed;
      dispatch({ type: "complete", result, report, context });
      return completed;
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      dispatch({ type: "fail", error: `エラー: ${message}` });
      return null;
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
