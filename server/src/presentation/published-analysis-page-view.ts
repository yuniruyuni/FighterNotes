import type { PublishedAnalysis } from "../models/published-analysis";
import { characterName } from "./published-analysis-catalog";
import {
  type PublishedFindingView,
  presentPublishedFindings,
} from "./published-analysis-finding-view";
import { presentPublishedSuperArts } from "./published-analysis-super-art-view";
import {
  type PublishedTacticView,
  presentPublishedTactics,
} from "./published-analysis-tactic-view";

export interface PublishedAnalysisPageUrls {
  readonly canonical: URL;
  readonly home: URL;
  readonly image: URL;
}

export interface PublishedAnalysisPageView {
  readonly ownCharacter: string;
  readonly opponentCharacter: string;
  readonly title: string;
  readonly description: string;
  readonly canonicalUrl: string;
  readonly homeUrl: string;
  readonly imageUrl: string;
  readonly managementUrl: string;
  readonly xIntentUrl: string;
  readonly rounds: PublishedAnalysis["content"]["rounds"];
  readonly findings: readonly PublishedFindingView[];
  readonly tactics: readonly PublishedTacticView[];
  readonly superArts: readonly PublishedTacticView[] | undefined;
  readonly createdDate: string;
  readonly expiresDate: string;
  readonly rulesetVersion: number;
}

const OFFICIAL_HASHTAG = "FighterNotes";

export const PublishedAnalysisPageView = {
  from(
    analysis: PublishedAnalysis,
    urls: PublishedAnalysisPageUrls,
  ): PublishedAnalysisPageView {
    const { content } = analysis;
    const ownCharacter = characterName(content.ownCharacter);
    const opponentCharacter = characterName(content.opponentCharacter);
    const findings = presentPublishedFindings(
      content.findings,
      content.rulesetVersion,
    );
    const topFinding = findings[0]?.title ?? "顕著な改善ポイントなし";
    const title = `${ownCharacter} vs ${opponentCharacter} 分析結果 | Fighter Notes`;
    const description = `${ownCharacter}側の対戦分析。${content.rounds.detected}ラウンド、優先項目: ${topFinding}。`;
    const xIntent = new URL("https://x.com/intent/tweet");
    xIntent.searchParams.set(
      "text",
      `${ownCharacter} vs ${opponentCharacter} の対戦分析結果 | Fighter Notes`,
    );
    xIntent.searchParams.set("url", urls.canonical.toString());
    xIntent.searchParams.set("hashtags", OFFICIAL_HASHTAG);

    return {
      ownCharacter,
      opponentCharacter,
      title,
      description,
      canonicalUrl: urls.canonical.toString(),
      homeUrl: urls.home.toString(),
      imageUrl: urls.image.toString(),
      managementUrl: new URL(`/manage/${analysis.id}`, urls.home).toString(),
      xIntentUrl: xIntent.toString(),
      rounds: content.rounds,
      findings,
      tactics: presentPublishedTactics(analysis),
      superArts: presentPublishedSuperArts(analysis),
      createdDate: dateText(analysis.createdAt),
      expiresDate: dateText(analysis.expiresAt),
      rulesetVersion: content.rulesetVersion,
    };
  },
};

function dateText(date: Date): string {
  return date.toISOString().slice(0, 10);
}
