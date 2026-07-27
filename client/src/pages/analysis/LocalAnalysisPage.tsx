import { useEffect } from "react";
import { useAnalysisSession } from "~/modules/analysis/index.js";
import { usePublication } from "~/modules/sharing/index.js";
import { PageLayout } from "~/shared/ui/PageLayout.js";
import { AnalysisWorkspacePage } from "./AnalysisWorkspacePage.js";

export function LocalAnalysisPage({ id }: { id: string }) {
  const { state } = useAnalysisSession();
  const publication = usePublication();
  if (state.phase === "ready" && publication.state.published?.id === id) {
    return <AnalysisWorkspacePage />;
  }
  return <ServerShareRedirect />;
}

function ServerShareRedirect() {
  useEffect(() => {
    window.location.reload();
  }, []);
  return (
    <PageLayout>
      <main className="route-status" aria-live="polite">
        公開された解析結果を読み込んでいます…
      </main>
    </PageLayout>
  );
}
