import { type ReactNode, useEffect, useMemo, useRef } from "react";
import { useLocation } from "wouter";
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
  useWorkspaceNavigation,
  VideoView,
  WorkspaceSidebar,
} from "~/modules/results/index.js";
import { SharePanel, usePublication } from "~/modules/sharing/index.js";

export function AnalysisWorkspacePage() {
  const { state, reset } = useAnalysisSession();
  const publication = usePublication();
  const [, navigateUrl] = useLocation();
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
      onHistoryUnwound={() => navigateUrl(paths.home, { replace: true })}
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
  onHistoryUnwound?(): void;
}

export function AnalysisWorkspace({
  file,
  result,
  report,
  context,
  sharing,
  onBack,
  onHistoryUnwound,
}: AnalysisWorkspaceProps) {
  const cards = useMemo(() => report.cards ?? [], [report.cards]);
  const { navigation, focusRevision, navigate, openScene, leave } =
    useWorkspaceNavigation(workspaceSession(file, context), cards);
  const summaryFocus = useRef<HTMLHeadingElement>(null);
  const videoFocus = useRef<HTMLInputElement>(null);
  const debugFocus = useRef<HTMLButtonElement>(null);

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

  const backToSetup = () => {
    onBack();
    // 解析結果を捨てた後に戻る操作が空振りしないよう、積んだ entry も畳む。
    leave(onHistoryUnwound);
  };

  return (
    <div id="screen-clips" style={{ display: "flex" }}>
      <div className="clips-app">
        <WorkspaceSidebar
          filename={file.name}
          cards={cards}
          selected={navigation.selected}
          onBack={backToSetup}
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

/** 解析ごとに history stack を分ける識別子。別の解析が残した entry を復元しないために使う。 */
function workspaceSession(file: File, context: AnalysisContext): string {
  return [
    file.name,
    file.size,
    file.lastModified,
    context.ownSide,
    context.p1.character ?? "",
    context.p2.character ?? "",
  ].join(":");
}
