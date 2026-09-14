import type {
  AdviceReport,
  AnalysisContext,
  OpponentMoveStat,
  TacticStats,
} from "~/modules/analysis/contracts.js";

export interface AnalysisVideoIdentity {
  size: number;
  lastModified: number;
}

export const ANALYSIS_HISTORY_ID_PREFIX = "v2:";

export interface AnalysisHistoryRecord {
  id: string;
  createdAt: string;
  rulesetVersion: number;
  ownCharacter: string;
  opponentCharacter: string;
  rounds: number;
  /** 勝ち/負けが確定したラウンド数。旧レコードは省略。 */
  roundsWon?: number;
  roundsLost?: number;
  tactics: TacticStats;
  /** 同定できた相手の技の内訳(v26+)。旧レコードは省略。 */
  opponentMoves?: OpponentMoveStat[];
}

async function createAnalysisHistoryId(
  video: AnalysisVideoIdentity,
  context: AnalysisContext,
  ownCharacter: string,
  opponentCharacter: string,
  rulesetVersion: number,
): Promise<string> {
  const input = JSON.stringify([
    video.size,
    video.lastModified,
    context.ownSide,
    ownCharacter,
    opponentCharacter,
    rulesetVersion,
  ]);
  const digest = new Uint8Array(
    await crypto.subtle.digest("SHA-256", new TextEncoder().encode(input)),
  );
  return `${ANALYSIS_HISTORY_ID_PREFIX}${Array.from(digest, (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("")}`;
}

export async function createAnalysisHistoryRecord(
  video: AnalysisVideoIdentity,
  context: AnalysisContext,
  report: AdviceReport,
  now = new Date(),
): Promise<AnalysisHistoryRecord> {
  const ownIsP2 = context.ownSide === "p2";
  const ownCharacter =
    (ownIsP2 ? context.p2.character : context.p1.character) || "未指定";
  const opponentCharacter =
    (ownIsP2 ? context.p1.character : context.p2.character) || "未指定";
  const id = await createAnalysisHistoryId(
    video,
    context,
    ownCharacter,
    opponentCharacter,
    report.ruleset_version,
  );
  const rounds = report.round_summaries ?? [];
  return {
    id,
    createdAt: now.toISOString(),
    rulesetVersion: report.ruleset_version,
    ownCharacter,
    opponentCharacter,
    rounds: report.rounds_detected,
    roundsWon: rounds.filter((round) => round.won === true).length,
    roundsLost: rounds.filter((round) => round.won === false).length,
    tactics: report.tactic_stats,
    opponentMoves: report.opponent_move_stats ?? [],
  };
}

export interface MatchupSummary {
  key: string;
  ownCharacter: string;
  opponentCharacter: string;
  matches: number;
  rounds: number;
  antiAirOpportunities: number;
  antiAirSuccesses: number;
  diFaced: number;
  diReturned: number;
  diUnconfirmed: number;
  rawRushesFaced: number;
  rawRushesDefended: number;
  rawRushesHit: number;
  rawRushesUnconfirmed: number;
  minusDefenseOpportunities: number;
  fastestStrikeChallenges: number;
  fastestStrikeLosses: number;
  fastestThrowChallenges: number;
  fastestThrowLosses: number;
  burnoutCount: number;
  burnoutSeconds: number;
  burnoutHpLost: number;
  burnoutHpDealt: number;
  roundsWon: number;
  roundsLost: number;
  /** 勝敗を記録できたレコード数。旧レコードは分母に入らない。 */
  matchesWithResults: number;
  /** 同定できた相手の技の通算(触られた回数順)。 */
  opponentMoves: OpponentMoveStat[];
}

export function aggregateMatchups(
  records: AnalysisHistoryRecord[],
  rulesetVersion: number,
): MatchupSummary[] {
  const groups = new Map<string, MatchupSummary>();
  for (const record of records) {
    if (record.rulesetVersion !== rulesetVersion) continue;
    const key = `${record.ownCharacter}\u0000${record.opponentCharacter}`;
    const current = groups.get(key) ?? {
      key,
      ownCharacter: record.ownCharacter,
      opponentCharacter: record.opponentCharacter,
      matches: 0,
      rounds: 0,
      antiAirOpportunities: 0,
      antiAirSuccesses: 0,
      diFaced: 0,
      diReturned: 0,
      diUnconfirmed: 0,
      rawRushesFaced: 0,
      rawRushesDefended: 0,
      rawRushesHit: 0,
      rawRushesUnconfirmed: 0,
      minusDefenseOpportunities: 0,
      fastestStrikeChallenges: 0,
      fastestStrikeLosses: 0,
      fastestThrowChallenges: 0,
      fastestThrowLosses: 0,
      burnoutCount: 0,
      burnoutSeconds: 0,
      burnoutHpLost: 0,
      burnoutHpDealt: 0,
      roundsWon: 0,
      roundsLost: 0,
      matchesWithResults: 0,
      opponentMoves: [],
    };
    current.matches += 1;
    current.rounds += record.rounds;
    current.antiAirOpportunities += record.tactics.anti_air_opportunities;
    current.antiAirSuccesses += record.tactics.anti_air_successes;
    current.diFaced += record.tactics.di_faced;
    current.diReturned += record.tactics.di_returned;
    current.diUnconfirmed += record.tactics.di_unconfirmed;
    current.rawRushesFaced += record.tactics.raw_drive_rushes_faced;
    current.rawRushesDefended += record.tactics.raw_drive_rushes_defended;
    current.rawRushesHit += record.tactics.raw_drive_rushes_hit;
    current.rawRushesUnconfirmed += record.tactics.raw_drive_rushes_unconfirmed;
    current.minusDefenseOpportunities +=
      record.tactics.minus_defense_opportunities ?? 0;
    current.fastestStrikeChallenges += record.tactics.fastest_strike_challenges;
    current.fastestStrikeLosses += record.tactics.fastest_strike_losses;
    current.fastestThrowChallenges += record.tactics.fastest_throw_challenges;
    current.fastestThrowLosses += record.tactics.fastest_throw_losses;
    current.burnoutCount += record.tactics.burnout_count;
    current.burnoutSeconds += record.tactics.burnout_seconds;
    current.burnoutHpLost += record.tactics.burnout_hp_lost;
    current.burnoutHpDealt += record.tactics.burnout_hp_dealt;
    if (record.roundsWon !== undefined && record.roundsLost !== undefined) {
      current.roundsWon += record.roundsWon;
      current.roundsLost += record.roundsLost;
      current.matchesWithResults += 1;
    }
    for (const move of record.opponentMoves ?? []) {
      const merged = current.opponentMoves.find(
        (entry) => entry.name === move.name,
      );
      if (merged) {
        merged.projectile = merged.projectile || move.projectile;
        merged.touches += move.touches;
        merged.hits_taken += move.hits_taken;
        merged.hp_lost += move.hp_lost;
        merged.blocked += move.blocked;
        merged.punished += move.punished;
        merged.punish_missed += move.punish_missed;
      } else {
        current.opponentMoves.push({ ...move });
      }
    }
    current.opponentMoves.sort(
      (left, right) =>
        right.touches - left.touches || left.name.localeCompare(right.name),
    );
    groups.set(key, current);
  }
  return [...groups.values()].sort(
    (left, right) =>
      right.matches - left.matches ||
      left.opponentCharacter.localeCompare(right.opponentCharacter),
  );
}

export interface DefensiveResponseBias {
  action: "strike" | "throw";
  opportunities: number;
  selections: number;
  losses: number;
  selectionPercent: number;
}

/**
 * 単発の読み負けではなく、複数試合を通した同一回答への偏りだけを返す。
 * Rust の単試合カードと同じ成立条件を使う。
 */
export function defensiveResponseBias(
  summary: MatchupSummary,
): DefensiveResponseBias | null {
  if (summary.minusDefenseOpportunities < 4) return null;
  const candidates = [
    {
      action: "strike" as const,
      selections: summary.fastestStrikeChallenges,
      losses: summary.fastestStrikeLosses,
    },
    {
      action: "throw" as const,
      selections: summary.fastestThrowChallenges,
      losses: summary.fastestThrowLosses,
    },
  ];
  const biased = candidates
    .filter(
      (candidate) =>
        candidate.losses >= 2 &&
        candidate.selections * 100 >= summary.minusDefenseOpportunities * 70,
    )
    .sort(
      (left, right) =>
        right.losses - left.losses || right.selections - left.selections,
    )[0];
  if (!biased) return null;
  return {
    ...biased,
    opportunities: summary.minusDefenseOpportunities,
    selectionPercent: Math.floor(
      (biased.selections * 100) / summary.minusDefenseOpportunities,
    ),
  };
}

export function rate(successes: number, opportunities: number): string {
  if (opportunities === 0) return "-";
  return `${Math.round((successes / opportunities) * 100)}%`;
}
