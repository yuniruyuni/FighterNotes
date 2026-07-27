import { Route, Switch } from "wouter";
import { isShareId } from "~/modules/sharing/index.js";
import { AnalysisPage } from "~/pages/analysis/AnalysisPage.js";
import { LocalAnalysisPage } from "~/pages/analysis/LocalAnalysisPage.js";
import { LicensesPage, PrivacyPage } from "~/pages/legal/LegalPages.js";
import { NotFoundPage } from "~/pages/not-found/NotFoundPage.js";
import { ShareManagementPage } from "~/pages/share-management/ShareManagementPage.js";
import { PageLayout } from "~/shared/ui/PageLayout.js";
import { paths, routePatterns } from "./paths.js";

export function AppRoutes() {
  return (
    <Switch>
      <Route path={paths.home}>
        <AnalysisPage />
      </Route>
      <Route path={paths.privacy}>
        <PageLayout>
          <PrivacyPage />
        </PageLayout>
      </Route>
      <Route path={paths.licenses}>
        <PageLayout>
          <LicensesPage />
        </PageLayout>
      </Route>
      <Route path={routePatterns.share}>
        {(params) => <LocalAnalysisPage id={params.id} />}
      </Route>
      <Route path={routePatterns.manageShare}>
        {(params) => (
          <PageLayout>
            <ShareManagementPage
              initialId={isShareId(params.id) ? params.id : ""}
            />
          </PageLayout>
        )}
      </Route>
      <Route path={paths.manage}>
        <PageLayout>
          <ShareManagementPage />
        </PageLayout>
      </Route>
      <Route>
        <NotFoundPage />
      </Route>
    </Switch>
  );
}
