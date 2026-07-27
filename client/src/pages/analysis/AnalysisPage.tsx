import { useAnalysisSession } from "~/modules/analysis/index.js";
import { PageLayout } from "~/shared/ui/PageLayout.js";
import { AnalysisSetupPage } from "./AnalysisSetupPage.js";
import { AnalysisWorkspacePage } from "./AnalysisWorkspacePage.js";

export function AnalysisPage() {
  const { state } = useAnalysisSession();
  if (state.phase === "ready") return <AnalysisWorkspacePage />;
  return (
    <PageLayout>
      <AnalysisSetupPage />
    </PageLayout>
  );
}
