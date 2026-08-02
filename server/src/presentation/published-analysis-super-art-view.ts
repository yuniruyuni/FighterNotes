import type { PublishedAnalysis } from "../models/published-analysis";
import type { PublishedTacticView } from "./published-analysis-tactic-view";

export function presentPublishedSuperArts(
  analysis: PublishedAnalysis,
): readonly PublishedTacticView[] | undefined {
  const aggregate = analysis.content.superArts;
  if (aggregate === undefined) return undefined;

  const views: PublishedTacticView[] = [];
  if (aggregate.own.availability === "unavailable") {
    views.push(unavailable("自分のSA / CA"));
  } else {
    const { levels, outcomes, contexts } = aggregate.own;
    views.push(
      {
        value: `${totalLevels(levels)}回`,
        label: "自分のSA / CA使用",
        detail: levelDetail(levels),
      },
      {
        value: `ヒット ${outcomes.hit}回`,
        label: "自分のSA / CA結果",
        detail: outcomeDetail(outcomes),
      },
      {
        value: `コンボ ${contexts.combo}回`,
        label: "自分のSA / CA利用文脈",
        detail: `確定反撃 ${contexts.punish}・切り返し ${contexts.reversal}・ニュートラル ${contexts.neutral}`,
      },
    );
  }

  if (aggregate.opponent.availability === "unavailable") {
    views.push(unavailable("相手のSA / CA"));
  } else {
    const { levels, outcomes } = aggregate.opponent;
    views.push(
      {
        value: `${totalLevels(levels)}回`,
        label: "相手のSA / CA使用",
        detail: levelDetail(levels),
      },
      {
        value: `ヒット ${outcomes.hit}回`,
        label: "相手のSA / CA結果",
        detail: outcomeDetail(outcomes),
      },
    );
  }
  return views;
}

function unavailable(label: string): PublishedTacticView {
  return {
    value: "集計不可",
    label,
    detail: "認識できなかったため、0回とは扱いません",
  };
}

function totalLevels(levels: {
  sa1: number;
  sa2: number;
  sa3: number;
  ca: number;
}): number {
  return levels.sa1 + levels.sa2 + levels.sa3 + levels.ca;
}

function levelDetail(levels: {
  sa1: number;
  sa2: number;
  sa3: number;
  ca: number;
}): string {
  return `SA1 ${levels.sa1}・SA2 ${levels.sa2}・SA3 ${levels.sa3}・CA ${levels.ca}`;
}

function outcomeDetail(outcomes: {
  hit: number;
  block: number;
  noImmediateContact: number;
  punished: number;
  ko: number;
}): string {
  return `ガード ${outcomes.block}・即時接触なし ${outcomes.noImmediateContact}・反撃を受けた ${outcomes.punished}・KO ${outcomes.ko}`;
}
