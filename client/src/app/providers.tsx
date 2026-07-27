import type { ReactNode } from "react";
import { browserAnalysisServices } from "~/modules/analysis/browser.js";
import { AnalysisSessionProvider } from "~/modules/analysis/index.js";
import { browserResultsServices } from "~/modules/results/browser.js";
import { ResultsServicesProvider } from "~/modules/results/index.js";
import { browserSharingServices } from "~/modules/sharing/browser.js";
import {
  PublicationProvider,
  type PublicationRoutes,
  SharingServicesProvider,
} from "~/modules/sharing/index.js";
import { paths } from "./paths.js";

const publicationRoutes: PublicationRoutes = {
  home: paths.home,
  share: paths.share,
};

export function AppProviders({ children }: { children: ReactNode }) {
  return (
    <AnalysisSessionProvider services={browserAnalysisServices}>
      <ResultsServicesProvider services={browserResultsServices}>
        <SharingServicesProvider services={browserSharingServices}>
          <PublicationProvider routes={publicationRoutes}>
            {children}
          </PublicationProvider>
        </SharingServicesProvider>
      </ResultsServicesProvider>
    </AnalysisSessionProvider>
  );
}
