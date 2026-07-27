import type {
  AttributedDamageEvent,
  DamageContext,
} from "~/modules/analysis/contracts.js";

const CONTEXT_LABELS: Record<DamageContext, string> = {
  mashing: "守勢のボタン押し",
  press_while_minus: "不利フレーム中",
  guard_break: "ガード入力崩れ",
  reversal_punished: "リバーサル失敗",
  punish_whiff: "確反空振り",
  burnout: "バーンアウト中",
};

export function formatHpRatio(ratio: number): string {
  return formatPercent(ratio * 100);
}

export function formatPercent(percent: number): string {
  return `${new Intl.NumberFormat("ja-JP", {
    maximumFractionDigits: 1,
  }).format(percent)}%`;
}

export function formatDamageContexts(contexts: readonly DamageContext[]) {
  return contexts.map((context) => CONTEXT_LABELS[context]).join("、");
}

export function confidenceLabel(
  confidence: AttributedDamageEvent["confidence"],
): string {
  if (confidence === "high") return "高";
  if (confidence === "medium") return "中";
  return "低";
}
