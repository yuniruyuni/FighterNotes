import { type ReactNode, useEffect, useReducer, useRef } from "react";
import { paths } from "~/app/paths.js";
import {
  type AdviceReport,
  type AnalysisContext,
  type AnalysisResult,
  useAnalysisSession,
} from "~/modules/analysis/index.js";
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
  if (!state.file || !state.result || !state.report || !state.context) {
    return null;
  }
  const { file, result, report, context } = state;
  const backToSetup = () => {
    publication.reset();
    reset();
  };

  return (
    <AnalysisWorkspace
      file={file}
      result={result}
      report={report}
      context={context}
      onBack={backToSetup}
      sharing={
        <SharePanel
          context={context}
          manageHref={paths.manage}
          report={report}
        />
      }
    />
  );
}

interface AnalysisWorkspaceProps {
  file: File;
  result: AnalysisResult;
  report: AdviceReport;
  context: AnalysisContext;
  sharing?: ReactNode;
  onBack(): void;
}

export function AnalysisWorkspace({
  file,
  result,
  report,
  context,
  sharing,
  onBack,
}: AnalysisWorkspaceProps) {
  const [navigation, dispatch] = useReducer(
    WorkspaceNavigation.reduce,
    undefined,
    WorkspaceNavigation.initial,
  );
  const [focusRevision, requestFocus] = useReducer(
    (revision: number) => revision + 1,
    0,
  );
  const summaryFocus = useRef<HTMLHeadingElement>(null);
  const videoFocus = useRef<HTMLInputElement>(null);
  const debugFocus = useRef<HTMLButtonElement>(null);
  const cards = report.cards ?? [];
  const openScene = (
    scene: Parameters<typeof WorkspaceNavigation.openScene>[1],
  ) => {
    const cardIndex = scene.card
      ? cards.findIndex((card) => card.id === scene.card?.id)
      : -1;
    dispatch({
      type: "scene",
      scene,
      selected: cardIndex >= 0 ? `card-${cardIndex}` : "video",
    });
    requestFocus();
  };

  // biome-ignore lint/correctness/useExhaustiveDependencies: repeated activation of the current item must restore panel focus.
  useEffect(() => {
    const target =
      navigation.view === "summary"
        ? summaryFocus.current
        : navigation.view === "video"
          ? videoFocus.current
          : debugFocus.current;
    target?.focus({ preventScroll: true });
  }, [focusRevision, navigation.view]);

  const navigate = (action: Parameters<typeof dispatch>[0]) => {
    dispatch(action);
    requestFocus();
  };

  return (
    <div id="screen-clips" style={{ display: "flex" }}>
      <div className="clips-app">
        <WorkspaceSidebar
          filename={file.name}
          cards={cards}
          selected={navigation.selected}
          onBack={onBack}
          onSummary={() => navigate({ type: "summary" })}
          onCard={(card, index) => navigate({ type: "card", card, index })}
          onVideo={() => navigate({ type: "video" })}
          onDebug={() => navigate({ type: "debug" })}
        />
        <main className="clips-main">
          <SummaryView
            active={navigation.view === "summary"}
            focusRef={summaryFocus}
            file={file}
            context={context}
            report={report}
            frameTimestamps={result.frameTimestamps}
            sharing={sharing}
            onSceneChange={openScene}
          />
          <VideoView
            active={navigation.view === "video"}
            focusRef={videoFocus}
            file={file}
            frameTimestamps={result.frameTimestamps}
            scene={navigation.scene}
            onSceneChange={openScene}
          />
          <DebugView
            active={navigation.view === "debug"}
            focusRef={debugFocus}
            file={file}
            result={result}
            side={context.ownSide}
          />
        </main>
      </div>
    </div>
  );
}
