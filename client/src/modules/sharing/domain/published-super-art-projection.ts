import type { TacticStats } from "~/modules/analysis/contracts.js";
import type {
  PublishedOpponentSuperArtStats,
  PublishedOwnSuperArtStats,
  PublishedSuperArtLevels,
  PublishedSuperArtOutcomes,
  PublishedSuperArtStats,
} from "./published-analysis-contract.js";
import {
  boundedInteger,
  MAX_COUNT,
  ShareProjectionError,
} from "./share-projection-value.js";

export function projectPublishedSuperArts(
  stats: TacticStats,
): PublishedSuperArtStats {
  return {
    own: projectOwn(stats),
    opponent: projectOpponent(stats),
  };
}

function projectOwn(stats: TacticStats): PublishedOwnSuperArtStats {
  const available = explicitAvailability(
    stats.super_art_stats_available,
    "superArts.own.availability",
  );
  if (!available) return { availability: "unavailable" };
  return {
    availability: "available",
    levels: levels(stats, "", "own"),
    outcomes: outcomes(stats, "", "own"),
    contexts: {
      combo: count(stats.super_combo_uses, "superArts.own.contexts.combo"),
      punish: count(stats.super_punish_uses, "superArts.own.contexts.punish"),
      reversal: count(
        stats.super_reversal_uses,
        "superArts.own.contexts.reversal",
      ),
      neutral: count(
        stats.super_neutral_uses,
        "superArts.own.contexts.neutral",
      ),
    },
  };
}

function projectOpponent(stats: TacticStats): PublishedOpponentSuperArtStats {
  const available = explicitAvailability(
    stats.opponent_super_art_stats_available,
    "superArts.opponent.availability",
  );
  if (!available) return { availability: "unavailable" };
  return {
    availability: "available",
    levels: levels(stats, "opponent_", "opponent"),
    outcomes: outcomes(stats, "opponent_", "opponent"),
  };
}

function levels(
  stats: TacticStats,
  prefix: "" | "opponent_",
  player: "own" | "opponent",
): PublishedSuperArtLevels {
  return {
    sa1: count(stats[`${prefix}sa1_used`], `superArts.${player}.levels.sa1`),
    sa2: count(stats[`${prefix}sa2_used`], `superArts.${player}.levels.sa2`),
    sa3: count(stats[`${prefix}sa3_used`], `superArts.${player}.levels.sa3`),
    ca: count(stats[`${prefix}ca_used`], `superArts.${player}.levels.ca`),
  };
}

function outcomes(
  stats: TacticStats,
  prefix: "" | "opponent_",
  player: "own" | "opponent",
): PublishedSuperArtOutcomes {
  return {
    hit: count(
      stats[`${prefix}super_hits`],
      `superArts.${player}.outcomes.hit`,
    ),
    block: count(
      stats[`${prefix}super_blocked`],
      `superArts.${player}.outcomes.block`,
    ),
    noImmediateContact: count(
      stats[`${prefix}super_no_immediate_contact`],
      `superArts.${player}.outcomes.noImmediateContact`,
    ),
    punished: count(
      stats[`${prefix}super_punished`],
      `superArts.${player}.outcomes.punished`,
    ),
    ko: count(stats[`${prefix}super_kos`], `superArts.${player}.outcomes.ko`),
  };
}

function explicitAvailability(value: unknown, field: string): boolean {
  if (typeof value === "boolean") return value;
  throw new ShareProjectionError(`${field} が不正です。`);
}

function count(value: unknown, field: string): number {
  if (typeof value !== "number") {
    throw new ShareProjectionError(`${field} が不正です。`);
  }
  return boundedInteger(value, MAX_COUNT, field);
}
