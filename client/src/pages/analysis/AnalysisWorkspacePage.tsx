import { useReducer } from "react";
import { paths } from "~/app/paths.js";
import { useAnalysisSession } from "~/modules/analysis/index.js";
import {
  DebugView,
  SummaryView,
  VideoView,
  WorkspaceNavigation,
  WorkspaceSidebar,
} from "~/modules/results/index.js";
import { SharePanel, usePublication } from "~/modules/sharing/index.js";

export function AnalysisWorkspacePage() {
  const { state, reset } = useAnalysisSession();
  const publication = usePublication();
  const [navigation, dispatch] = useReducer(
    WorkspaceNavigation.reduce,
    undefined,
    WorkspaceNavigation.initial,
  );
  if (!state.file || !state.result || !state.report || !state.context) {
    return null;
  }
  const { file, result, report, context } = state;
  const openScene = (
    scene: Parameters<typeof WorkspaceNavigation.openScene>[1],
  ) => dispatch({ type: "scene", scene });
  const backToSetup = () => {
    publication.reset();
    reset();
  };

  return (
    <div id="screen-clips" style={{ display: "flex" }}>
      <div className="clips-app">
        <WorkspaceSidebar
          filename={file.name}
          cards={report.cards ?? []}
          selected={navigation.selected}
          onBack={backToSetup}
          onSummary={() => dispatch({ type: "summary" })}
          onCard={(card, index) => dispatch({ type: "card", card, index })}
          onDebug={() => dispatch({ type: "debug" })}
        />
        <main className="clips-main">
          <SummaryView
            active={navigation.view === "summary"}
            file={file}
            context={context}
            report={report}
            frameTimestamps={result.frameTimestamps}
            sharing={
              <SharePanel
                context={context}
                manageHref={paths.manage}
                report={report}
              />
            }
            onSceneChange={openScene}
          />
          <VideoView
            active={navigation.view === "video"}
            file={file}
            frameTimestamps={result.frameTimestamps}
            scene={navigation.scene}
            onSceneChange={openScene}
          />
          <DebugView
            active={navigation.view === "debug"}
            file={file}
            result={result}
            side={context.ownSide}
          />
        </main>
      </div>
    </div>
  );
}
