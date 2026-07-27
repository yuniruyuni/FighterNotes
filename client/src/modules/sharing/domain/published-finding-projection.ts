import type { AdviceCard } from "~/modules/analysis/contracts.js";
import {
  type PublishedFindingCandidate,
  SHAREABLE_FINDING_KINDS,
  type ShareableFindingKind,
} from "./published-analysis-contract.js";
import {
  MAX_COUNT,
  MAX_SEVERITY_BP,
  ShareProjectionError,
  scaledInteger,
} from "./share-projection-value.js";

export function projectPublishedFindings(
  cards: readonly AdviceCard[],
): PublishedFindingCandidate[] {
  const knownKinds = new Set<string>(SHAREABLE_FINDING_KINDS);
  const seen = new Set<string>();
  const findings = cards.map((card) => {
    if (!knownKinds.has(card.id)) {
      throw new ShareProjectionError(`未対応の指摘種別です: ${card.id}`);
    }
    if (seen.has(card.id)) {
      throw new ShareProjectionError(`指摘種別が重複しています: ${card.id}`);
    }
    seen.add(card.id);
    if (card.evidence.length === 0) {
      throw new ShareProjectionError(
        `証拠のない指摘は共有できません: ${card.id}`,
      );
    }
    return {
      kind: card.id as ShareableFindingKind,
      assessment: card.kind ?? "observation",
      occurrences: Math.min(MAX_COUNT, card.evidence.length),
      severityBp: scaledInteger(
        card.severity,
        10_000,
        MAX_SEVERITY_BP,
        `${card.id}.severity`,
      ),
    };
  });
  const order = new Map(
    SHAREABLE_FINDING_KINDS.map((kind, index) => [kind, index]),
  );
  findings.sort(
    (left, right) =>
      (order.get(left.kind) ?? Number.MAX_SAFE_INTEGER) -
      (order.get(right.kind) ?? Number.MAX_SAFE_INTEGER),
  );
  return findings;
}
