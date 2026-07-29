import { rate } from "../../domain/history.js";

export function formatTacticCount(
  successes: number,
  opportunities: number,
  unconfirmed = 0,
): string {
  if (opportunities > 0) return `${successes} / ${opportunities}`;
  return unconfirmed > 0 ? `未確認 ${unconfirmed} 件` : "確認なし";
}

export function formatTacticRateWithCount(
  successes: number,
  opportunities: number,
  unconfirmed = 0,
): string {
  if (opportunities === 0) {
    return formatTacticCount(successes, opportunities, unconfirmed);
  }
  const confirmed = `${rate(successes, opportunities)} (${successes}/${opportunities})`;
  return unconfirmed > 0 ? `${confirmed}・未確認 ${unconfirmed} 件` : confirmed;
}

export function appendUnconfirmedCandidates(
  detail: string,
  unconfirmed: number,
): string {
  return unconfirmed > 0 ? `${detail} / 未確認候補 ${unconfirmed} 件` : detail;
}
