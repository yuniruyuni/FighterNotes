import type {
  AttributedDamageEvent,
  DamageOrigin,
  StrikeKind,
} from "~/modules/analysis/contracts.js";

const ORIGIN_LABELS: Record<DamageOrigin, string> = {
  compound_threat: "弾＋テレポート",
  teleport: "テレポート",
  throw: "投げ",
  drive_impact: "ドライブインパクト",
  raw_drive_rush: "生ドライブラッシュ",
  own_jump_caught: "ジャンプを狩られた",
  opponent_jump_in: "相手の飛び込み",
  projectile: "飛び道具",
  strike: "打撃（属性不明）",
  unclassified: "未分類（要確認）",
};

const STRIKE_LABELS: Record<StrikeKind, string> = {
  high: "上段",
  overhead: "中段",
  low: "下段",
  air: "空中攻撃",
};

export type DamageGroupKey = DamageOrigin | `strike_${StrikeKind}`;

export interface DamageOriginRow {
  key: DamageGroupKey;
  origin: DamageOrigin;
  label: string;
  hpLost: number;
  compositionPercent: number;
  events: AttributedDamageEvent[];
}

export interface DamageOriginSummary {
  totalHpLost: number;
  classifiedHpLost: number;
  classifiedPercent: number;
  rows: DamageOriginRow[];
}

export function summarizeDamageOrigins(
  events: readonly AttributedDamageEvent[],
  round: "all" | number,
): DamageOriginSummary {
  const scoped = events.filter(
    (event) =>
      (round === "all" || event.round_no === round) &&
      Number.isFinite(event.hp_drop) &&
      event.hp_drop > 0,
  );
  const grouped = new Map<DamageGroupKey, AttributedDamageEvent[]>();
  for (const event of scoped) {
    const key: DamageGroupKey =
      event.origin === "strike" && event.strike_kind
        ? `strike_${event.strike_kind}`
        : event.origin;
    const group = grouped.get(key) ?? [];
    group.push(event);
    grouped.set(key, group);
  }
  const rows = [...grouped.entries()]
    .map(([key, groupedEvents]) => {
      const origin = groupedEvents[0].origin;
      const strikeKind = groupedEvents[0].strike_kind;
      return {
        key,
        origin,
        label:
          origin === "strike" && strikeKind
            ? STRIKE_LABELS[strikeKind]
            : ORIGIN_LABELS[origin],
        hpLost: groupedEvents.reduce((sum, event) => sum + event.hp_drop, 0),
        compositionPercent: 0,
        events: groupedEvents,
      };
    })
    .sort(
      (left, right) =>
        right.hpLost - left.hpLost ||
        left.label.localeCompare(right.label, "ja"),
    );
  const composition = allocateCompositionTenths(rows.map((row) => row.hpLost));
  rows.forEach((row, index) => {
    row.compositionPercent = composition[index] / 10;
  });

  const totalHpLost = rows.reduce((sum, row) => sum + row.hpLost, 0);
  const classifiedHpLost = rows
    .filter((row) => row.origin !== "unclassified")
    .reduce((sum, row) => sum + row.hpLost, 0);
  return {
    totalHpLost,
    classifiedHpLost,
    classifiedPercent:
      totalHpLost > 0 ? (classifiedHpLost / totalHpLost) * 100 : 0,
    rows,
  };
}

function allocateCompositionTenths(values: readonly number[]): number[] {
  const total = values.reduce((sum, value) => sum + value, 0);
  const exact = values.map((value) => (value / total) * 1000);
  const allocated = exact.map(Math.floor);
  const remainder = 1000 - allocated.reduce((sum, value) => sum + value, 0);
  const order = exact
    .map((value, index) => ({ index, fraction: value - allocated[index] }))
    // Array#sort is stable, so equal fractions retain row order.
    .sort((left, right) => right.fraction - left.fraction);
  for (const { index } of order.slice(0, remainder)) {
    allocated[index] += 1;
  }
  return allocated;
}
