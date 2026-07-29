import type { PublishedAnalysis } from "../models/published-analysis";

export interface PublishedTacticView {
  readonly value: string;
  readonly label: string;
  readonly detail: string;
}

export function presentPublishedTactics(
  analysis: PublishedAnalysis,
): readonly PublishedTacticView[] {
  const stats = analysis.content.tactics;
  const burnoutBalance = stats.burnout.hpDealtBp - stats.burnout.hpLostBp;
  return [
    {
      value: countFraction(
        stats.antiAir.successes,
        stats.antiAir.opportunities,
      ),
      label: "対空 成功 / 機会",
      detail: `飛びを通された ${stats.antiAir.jumpInsAllowed}回`,
    },
    {
      value: countFraction(
        stats.driveImpact.returned,
        stats.driveImpact.faced,
        stats.driveImpact.unconfirmed,
      ),
      label: "DI返し / 相手DI",
      detail: appendUnconfirmedCandidates(
        `ガード ${stats.driveImpact.blocked}・パリィ ${stats.driveImpact.parried}・被弾 ${stats.driveImpact.hit}`,
        stats.driveImpact.unconfirmed,
      ),
    },
    {
      value: countFraction(
        stats.rawDriveRush.defended,
        stats.rawDriveRush.faced,
        stats.rawDriveRush.unconfirmed,
      ),
      label: "生ラッシュ対処 / 相手ラッシュ",
      detail: appendUnconfirmedCandidates(
        `被弾 ${stats.rawDriveRush.hit}回`,
        stats.rawDriveRush.unconfirmed,
      ),
    },
    {
      value: `${stats.dashThrow.faced}回`,
      label: "前ステップ投げを受けた",
      detail: "",
    },
    {
      value: `${stats.throwWhiff.count}回`,
      label: "自分の投げ空振り",
      detail: "",
    },
    {
      value: countFraction(
        stats.fastestChallenge.strikeLosses,
        stats.fastestChallenge.strikeAttempts,
      ),
      label: "最速打撃の被弾 / 試行",
      detail: `入力確認済みの不利状況 ${stats.fastestChallenge.opportunities}回`,
    },
    {
      value: countFraction(
        stats.fastestChallenge.throwLosses,
        stats.fastestChallenge.throwAttempts,
      ),
      label: "最速投げの被弾 / 試行",
      detail: `入力確認済みの不利状況 ${stats.fastestChallenge.opportunities}回`,
    },
    {
      value: `${stats.burnout.count}回・${(
        stats.burnout.durationDeciseconds / 10
      ).toFixed(1)}秒`,
      label: "バーンアウト",
      detail: `HP収支 ${signedPercent(burnoutBalance)}`,
    },
  ];
}

function countFraction(
  successes: number,
  opportunities: number,
  unconfirmed = 0,
): string {
  if (opportunities > 0) return `${successes} / ${opportunities}`;
  return unconfirmed > 0 ? `未確認 ${unconfirmed} 件` : "確認なし";
}

function appendUnconfirmedCandidates(
  detail: string,
  unconfirmed: number,
): string {
  return unconfirmed > 0 ? `${detail}・未確認候補 ${unconfirmed}件` : detail;
}

function signedPercent(basisPoints: number): string {
  const value = Math.round(basisPoints / 100);
  return `${value > 0 ? "+" : ""}${value}%`;
}
