import type { TacticStats } from "~/modules/analysis/contracts.js";
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

/**
 * SF6 の Drive ゲージは6本。解析側は「ゲージ全量に対する比」で消費を実測して
 * いるので、利用者が普段数えている本数へ直してから見せる。
 */
const DRIVE_GAUGE_BARS = 6;

function driveSpentBars(stats: TacticStats): number {
  return (
    ((stats.drive_spent_on_impacts ?? 0) + (stats.drive_spent_on_rushes ?? 0)) *
    DRIVE_GAUGE_BARS
  );
}

/**
 * 消費を実測できた行動が無ければ率を出さない。0 と書くと「使ったが無駄
 * だった」に読めるが、実際は測れていないだけで意味が違う。
 */
export function formatDriveEfficiency(stats: TacticStats): string {
  const bars = driveSpentBars(stats);
  if ((stats.drive_spend_samples ?? 0) === 0 || bars <= 0) return "確認なし";
  const damage =
    (stats.drive_damage_from_impacts ?? 0) +
    (stats.drive_damage_from_rushes ?? 0);
  return `${((damage / bars) * 100).toFixed(1)}%`;
}

export function driveSpendBreakdown(stats: TacticStats): string {
  if ((stats.drive_spend_samples ?? 0) === 0) {
    return "ゲージ消費を実測できた行動がありません";
  }
  const part = (spent: number, damage: number, label: string) => {
    const bars = spent * DRIVE_GAUGE_BARS;
    if (bars <= 0) return undefined;
    return `${label} ${bars.toFixed(1)}本→${(damage * 100).toFixed(0)}%`;
  };
  return (
    [
      part(
        stats.drive_spent_on_impacts ?? 0,
        stats.drive_damage_from_impacts ?? 0,
        "DI",
      ),
      part(
        stats.drive_spent_on_rushes ?? 0,
        stats.drive_damage_from_rushes ?? 0,
        "生ラッシュ",
      ),
    ]
      .filter(Boolean)
      .join(" / ") || "ゲージ消費を実測できた行動がありません"
  );
}
