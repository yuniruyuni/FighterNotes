import { router } from "../init";
import { publishedAnalysisRouter } from "./published-analysis";

export const appRouter = router({
  publishedAnalysis: publishedAnalysisRouter,
});

export type AppRouter = typeof appRouter;
