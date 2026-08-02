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
    const partial = aggregate.own.availability === "partial";
    views.push(
      {
        value: `${totalLevels(levels)}回${partial ? "以上" : ""}`,
        label: `自分のSA / CA使用${partial ? "（下限）" : ""}`,
        detail: partial
          ? `確認できた範囲（各値は下限）: ${levelDetail(levels)}`
          : levelDetail(levels),
      },
      {
        value: partial ? "確認できた範囲" : `ヒット ${outcomes.hit}回`,
        label: `自分のSA / CA結果${partial ? "（下限）" : ""}`,
        detail: partial
          ? `各値は下限: ヒット ${outcomes.hit}・${outcomeDetail(outcomes)}`
          : outcomeDetail(outcomes),
      },
      {
        value: partial ? "確認できた範囲" : `コンボ ${contexts.combo}回`,
        label: `自分のSA / CA利用文脈${partial ? "（下限）" : ""}`,
        detail: contextDetail(contexts, partial),
      },
    );
  }

  if (aggregate.opponent.availability === "unavailable") {
    views.push(unavailable("相手のSA / CA"));
  } else {
    const { levels, outcomes } = aggregate.opponent;
    const partial = aggregate.opponent.availability === "partial";
    views.push(
      {
        value: `${totalLevels(levels)}回${partial ? "以上" : ""}`,
        label: `相手のSA / CA使用${partial ? "（下限）" : ""}`,
        detail: partial
          ? `確認できた範囲（各値は下限）: ${levelDetail(levels)}`
          : levelDetail(levels),
      },
      {
        value: partial ? "確認できた範囲" : `ヒット ${outcomes.hit}回`,
        label: `相手のSA / CA結果${partial ? "（下限）" : ""}`,
        detail: partial
          ? `各値は下限: ヒット ${outcomes.hit}・${outcomeDetail(outcomes)}`
          : outcomeDetail(outcomes),
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

function contextDetail(
  contexts: {
    combo: number;
    punish: number;
    reversal: number;
    neutral: number;
  },
  partial: boolean,
): string {
  const values = `コンボ ${contexts.combo}・確定反撃 ${contexts.punish}・切り返し ${contexts.reversal}・ニュートラル ${contexts.neutral}`;
  return partial
    ? `各値は下限: ${values}`
    : `確定反撃 ${contexts.punish}・切り返し ${contexts.reversal}・ニュートラル ${contexts.neutral}`;
}
