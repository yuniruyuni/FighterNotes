import type { DamageAttackEvidence } from "~/modules/analysis/contracts.js";
import { confidenceLabel } from "./damage-origin-format.js";

export type AttackEvidenceDisplayStatus =
  | "consistent"
  | "mismatch"
  | "unverified"
  | "incomplete";

const ATTRIBUTE_LABELS: Record<
  NonNullable<DamageAttackEvidence["starter_attribute"]>,
  string
> = {
  upper: "上段",
  middle: "中段",
  lower: "下段",
  throw: "投げ",
};

const STATUS_LABELS: Record<AttackEvidenceDisplayStatus, string> = {
  consistent: "HPバーと整合",
  mismatch: "HPバーと不一致",
  unverified: "HP未照合",
  incomplete: "表示認識が不完全",
};

export function attackEvidenceStatus(
  evidence: DamageAttackEvidence,
): AttackEvidenceDisplayStatus {
  if (!evidence.complete || evidence.recovered_from_max) return "incomplete";
  return evidence.hp_consistency;
}

export function attackEvidenceStatusLabel(
  status: AttackEvidenceDisplayStatus,
): string {
  return STATUS_LABELS[status];
}

export function attackAttributeLabel(
  attribute: DamageAttackEvidence["starter_attribute"],
): string {
  if (!attribute) return "未認識";
  return ATTRIBUTE_LABELS[attribute];
}

export function formatAttackEvidenceAria(
  evidence: DamageAttackEvidence,
): string {
  const status = attackEvidenceStatus(evidence);
  return [
    `ゲーム内表示 累積ダメージ ${formatInteger(evidence.combo_damage)}`,
    `${formatInteger(evidence.sequence_count)} hit`,
    `最終補正 ${formatInteger(evidence.final_scaling_percent)}%`,
    `始動 ${attackAttributeLabel(evidence.starter_attribute)}`,
    `最終 ${attackAttributeLabel(evidence.final_attribute)}`,
    attackEvidenceStatusLabel(status),
    `認識確度 ${confidenceLabel(evidence.confidence)}`,
  ].join("、");
}

export function formatInteger(value: number): string {
  return Math.round(value).toLocaleString("ja-JP");
}

export function formatSignedInteger(value: number): string {
  const rounded = Math.round(value);
  return `${rounded > 0 ? "+" : ""}${rounded.toLocaleString("ja-JP")}`;
}
