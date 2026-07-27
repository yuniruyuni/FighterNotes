import { useEffect, useState } from "react";
import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import { recordAndSummarizeMatchups } from "../../application/matchup-history.js";
import type { MatchupSummary } from "../../domain/history.js";
import { useResultsServices } from "../ResultsServicesProvider.js";

export interface MatchupHistoryState {
  phase: "loading" | "ready" | "error";
  summaries: MatchupSummary[];
}

export function useMatchupHistory(
  file: File,
  context: AnalysisContext,
  report: AdviceReport,
): MatchupHistoryState {
  const { history } = useResultsServices();
  const [state, setState] = useState<MatchupHistoryState>({
    phase: "loading",
    summaries: [],
  });

  useEffect(() => {
    let active = true;
    setState({ phase: "loading", summaries: [] });
    void (async () => {
      try {
        const summaries = await recordAndSummarizeMatchups(
          file,
          context,
          report,
          history,
        );
        if (active) setState({ phase: "ready", summaries });
      } catch {
        if (active) setState({ phase: "error", summaries: [] });
      }
    })();
    return () => {
      active = false;
    };
  }, [context, file, history, report]);

  return state;
}
