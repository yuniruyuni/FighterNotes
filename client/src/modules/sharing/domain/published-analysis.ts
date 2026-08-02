import type {
  AdviceReport,
  AnalysisContext,
} from "~/modules/analysis/contracts.js";
import {
  type CharacterId,
  isCharacterId,
} from "~/modules/analysis/contracts.js";
import type { PublishedAnalysisCandidate as Candidate } from "./published-analysis-contract.js";
import { projectPublishedFindings } from "./published-finding-projection.js";
import { projectPublishedTactics } from "./published-tactic-projection.js";
import {
  boundedInteger,
  MAX_COUNT,
  MAX_ROUNDS,
  ShareProjectionError,
} from "./share-projection-value.js";

export type PublishedAnalysisCandidate = Candidate;
export { ShareProjectionError } from "./share-projection-value.js";

export const SHAREABLE_RULESET_VERSIONS = [3, 4, 5, 6, 7, 8] as const;
export const RULESET_V9_SHARE_UNAVAILABLE =
  "この解析結果（ruleset v9）は共有形式の更新が完了するまで公開できません。ローカルの解析結果は引き続き利用できます。";

export function sharingUnavailableReason(
  rulesetVersion: number,
): string | undefined {
  if (rulesetVersion === 9) return RULESET_V9_SHARE_UNAVAILABLE;
  if (
    Number.isInteger(rulesetVersion) &&
    (SHAREABLE_RULESET_VERSIONS as readonly number[]).includes(rulesetVersion)
  ) {
    return undefined;
  }
  return "この解析ルール世代は共有に対応していません。";
}

export const PublishedAnalysisCandidate = {
  from(context: AnalysisContext, report: AdviceReport): Candidate {
    const own = context.ownSide === "p1" ? context.p1 : context.p2;
    const opponent = context.ownSide === "p1" ? context.p2 : context.p1;
    const rounds = report.round_summaries.slice(0, MAX_ROUNDS);
    const won = rounds.filter((round) => round.won === true).length;
    const lost = rounds.filter((round) => round.won === false).length;
    const detected = rounds.length;

    const rulesetVersion = boundedInteger(
      report.ruleset_version,
      MAX_COUNT,
      "rulesetVersion",
    );
    const unavailableReason = sharingUnavailableReason(rulesetVersion);
    if (unavailableReason) throw new ShareProjectionError(unavailableReason);

    return {
      rulesetVersion,
      ownCharacter: selectedCharacter(own.character, "自分"),
      opponentCharacter: selectedCharacter(opponent.character, "相手"),
      rounds: {
        detected,
        won,
        lost,
        unresolved: detected - won - lost,
      },
      findings: projectPublishedFindings(report.cards),
      tactics: projectPublishedTactics(report.tactic_stats),
    };
  },
};

function selectedCharacter(
  value: string | undefined,
  playerLabel: string,
): CharacterId {
  if (value && isCharacterId(value)) return value;
  throw new ShareProjectionError(
    `${playerLabel}のキャラクターを選択すると共有できます。`,
  );
}
