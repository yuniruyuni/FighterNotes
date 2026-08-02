import type { ReactNode, Ref } from "react";
import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import { AdviceSection } from "./AdviceSection.js";
import { DamageOriginsSection } from "./DamageOriginsSection.js";
import { MatchupHistorySection } from "./MatchupHistorySection.js";
import { PracticeSection } from "./PracticeSection.js";
import {
  InputStatsSection,
  TacticStatsSection,
} from "./ReportStatsSections.js";
import { RoundSection } from "./RoundSection.js";
import { SummaryOverview } from "./SummaryOverview.js";

interface SummaryViewProps {
  active: boolean;
  focusRef?: Ref<HTMLHeadingElement>;
  file: File;
  context: AnalysisContext;
  report: AdviceReport;
  frameTimestamps: readonly number[];
  sharing?: ReactNode;
  onSceneChange(scene: Omit<SceneSelection, "key">): void;
}

export function SummaryView({
  active,
  focusRef,
  file,
  context,
  report,
  frameTimestamps,
  sharing,
  onSceneChange,
}: SummaryViewProps) {
  return (
    <section
      id="view-summary"
      aria-labelledby="summary-view-heading"
      hidden={!active}
      inert={!active}
      style={{ display: active ? "block" : "none" }}
    >
      <SummaryOverview
        context={context}
        report={report}
        sharing={sharing}
        headingRef={focusRef}
      />
      <AdviceSection
        report={report}
        frameTimestamps={frameTimestamps}
        onSceneChange={onSceneChange}
      />
      <RoundSection report={report} onSceneChange={onSceneChange} />
      <DamageOriginsSection
        breakdown={report.damage_breakdown}
        rounds={report.round_summaries ?? []}
        frameTimestamps={frameTimestamps}
        onSceneChange={onSceneChange}
      />
      <InputStatsSection stats={report.input_stats} />
      <TacticStatsSection stats={report.tactic_stats} />
      <MatchupHistorySection file={file} context={context} report={report} />
      <PracticeSection items={report.practice_items} />
    </section>
  );
}
