import type {
  FindingAssessment,
  PublishedFinding,
} from "../models/published-analysis";
import {
  FINDING_PRESENTATIONS,
  type FindingPresentation,
} from "./published-analysis-catalog";

export interface PublishedFindingView {
  readonly index: string;
  readonly title: string;
  readonly description: string;
  readonly practice: string;
  readonly tone: FindingPresentation["tone"];
  readonly count: string;
}

export function presentPublishedFindings(
  source: readonly PublishedFinding[],
  rulesetVersion: number,
): readonly PublishedFindingView[] {
  return [...source]
    .sort((left, right) => {
      const categoryDifference =
        findingPriority(rulesetVersion, right.assessment) -
        findingPriority(rulesetVersion, left.assessment);
      return categoryDifference || right.severityBp - left.severityBp;
    })
    .map((finding, index) => presentFinding(finding, index, rulesetVersion));
}

function presentFinding(
  finding: PublishedFinding,
  index: number,
  rulesetVersion: number,
): PublishedFindingView {
  const presentation = findingPresentation(finding);
  const category = findingCategoryLabel(rulesetVersion, finding.assessment);
  return {
    index: String(index + 1).padStart(2, "0"),
    title: presentation.title,
    description: presentation.description,
    practice: presentation.practice,
    tone: presentation.tone,
    count: `${category ? `${category}・` : ""}${finding.occurrences}件`,
  };
}

function findingPresentation(finding: PublishedFinding): FindingPresentation {
  const base = FINDING_PRESENTATIONS[finding.kind];
  return finding.assessment === "observation" && base.observation
    ? { ...base, ...base.observation }
    : base;
}

function findingPriority(
  rulesetVersion: number,
  assessment: FindingAssessment,
): number {
  if (rulesetVersion < 4) return 0;
  return { diagnosis: 2, observation: 1, statistic: 0 }[assessment];
}

function findingCategoryLabel(
  rulesetVersion: number,
  assessment: FindingAssessment,
): string | undefined {
  if (rulesetVersion < 4) return undefined;
  return {
    diagnosis: "原因診断",
    observation: "確認場面",
    statistic: "統計",
  }[assessment];
}
