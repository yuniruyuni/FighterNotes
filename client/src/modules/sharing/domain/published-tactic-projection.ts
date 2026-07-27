import type { TacticStats } from "~/modules/analysis/contracts.js";
import type { PublishedTacticStats } from "./published-analysis-contract.js";
import {
  boundedInteger,
  MAX_COUNT,
  MAX_DURATION_DECISECONDS,
  MAX_HP_BP,
  scaledInteger,
} from "./share-projection-value.js";

export function projectPublishedTactics(
  stats: TacticStats,
): PublishedTacticStats {
  const count = (value: number, field: string) =>
    boundedInteger(value, MAX_COUNT, field);
  return {
    antiAir: {
      opportunities: count(
        stats.anti_air_opportunities,
        "antiAir.opportunities",
      ),
      successes: count(stats.anti_air_successes, "antiAir.successes"),
      jumpInsAllowed: count(stats.jump_ins_allowed, "antiAir.jumpInsAllowed"),
    },
    driveImpact: {
      faced: count(stats.di_faced, "driveImpact.faced"),
      returned: count(stats.di_returned, "driveImpact.returned"),
      blocked: count(stats.di_blocked, "driveImpact.blocked"),
      parried: count(stats.di_parried, "driveImpact.parried"),
      hit: count(stats.di_hit, "driveImpact.hit"),
      avoided: count(stats.di_avoided, "driveImpact.avoided"),
      unconfirmed: count(stats.di_unconfirmed, "driveImpact.unconfirmed"),
    },
    rawDriveRush: {
      faced: count(stats.raw_drive_rushes_faced, "rawDriveRush.faced"),
      defended: count(stats.raw_drive_rushes_defended, "rawDriveRush.defended"),
      hit: count(stats.raw_drive_rushes_hit, "rawDriveRush.hit"),
      unconfirmed: count(
        stats.raw_drive_rushes_unconfirmed,
        "rawDriveRush.unconfirmed",
      ),
    },
    dashThrow: { faced: count(stats.dash_throws_faced, "dashThrow.faced") },
    throwWhiff: { count: count(stats.throw_whiffs, "throwWhiff.count") },
    fastestChallenge: {
      opportunities: count(
        stats.minus_defense_opportunities,
        "fastestChallenge.opportunities",
      ),
      strikeAttempts: count(
        stats.fastest_strike_challenges,
        "fastestChallenge.strikeAttempts",
      ),
      strikeLosses: count(
        stats.fastest_strike_losses,
        "fastestChallenge.strikeLosses",
      ),
      throwAttempts: count(
        stats.fastest_throw_challenges,
        "fastestChallenge.throwAttempts",
      ),
      throwLosses: count(
        stats.fastest_throw_losses,
        "fastestChallenge.throwLosses",
      ),
    },
    burnout: {
      count: count(stats.burnout_count, "burnout.count"),
      durationDeciseconds: scaledInteger(
        stats.burnout_seconds,
        10,
        MAX_DURATION_DECISECONDS,
        "burnout.durationDeciseconds",
      ),
      hpLostBp: scaledInteger(
        stats.burnout_hp_lost,
        10_000,
        MAX_HP_BP,
        "burnout.hpLostBp",
      ),
      hpDealtBp: scaledInteger(
        stats.burnout_hp_dealt,
        10_000,
        MAX_HP_BP,
        "burnout.hpDealtBp",
      ),
      selfInitiated: count(
        stats.burnout_self_initiated,
        "burnout.selfInitiated",
      ),
      forced: count(stats.burnout_forced, "burnout.forced"),
      mixed: count(stats.burnout_mixed, "burnout.mixed"),
      unknown: count(stats.burnout_unknown, "burnout.unknown"),
    },
  };
}
