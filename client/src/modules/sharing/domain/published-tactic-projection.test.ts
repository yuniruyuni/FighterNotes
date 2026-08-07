import { describe, expect, test } from "bun:test";
import type { TacticStats } from "~/modules/analysis/contracts.js";
import { projectPublishedTactics } from "./published-tactic-projection.js";
import { ShareProjectionError } from "./share-projection-value.js";

function stats(overrides: Partial<TacticStats> = {}): TacticStats {
  return {
    anti_air_opportunities: 1,
    anti_air_successes: 2,
    jump_ins_allowed: 3,
    di_faced: 4,
    di_returned: 5,
    di_blocked: 6,
    di_parried: 7,
    di_hit: 8,
    di_avoided: 9,
    di_unconfirmed: 10,
    raw_drive_rushes_faced: 11,
    raw_drive_rushes_defended: 12,
    raw_drive_rushes_hit: 13,
    raw_drive_rushes_unconfirmed: 14,
    dash_throws_faced: 15,
    throw_whiffs: 16,
    minus_defense_opportunities: 17,
    advantage_opportunities: 0,
    advantage_continued: 0,
    advantage_abandoned: 0,
    advantage_turns_lost: 0,
    fastest_strike_challenges: 18,
    fastest_strike_losses: 19,
    fastest_throw_challenges: 20,
    fastest_throw_losses: 21,
    burnout_count: 22,
    burnout_seconds: 12.34,
    burnout_hp_lost: 0.2345,
    burnout_hp_dealt: 0.3456,
    burnout_self_initiated: 23,
    burnout_forced: 24,
    burnout_mixed: 25,
    burnout_unknown: 26,
    ...overrides,
  };
}

describe("published tactic projection", () => {
  test("全戦術値を共有用の整数fieldへ射影する", () => {
    expect(projectPublishedTactics(stats())).toEqual({
      antiAir: { opportunities: 1, successes: 2, jumpInsAllowed: 3 },
      driveImpact: {
        faced: 4,
        returned: 5,
        blocked: 6,
        parried: 7,
        hit: 8,
        avoided: 9,
        unconfirmed: 10,
      },
      rawDriveRush: { faced: 11, defended: 12, hit: 13, unconfirmed: 14 },
      dashThrow: { faced: 15 },
      throwWhiff: { count: 16 },
      fastestChallenge: {
        opportunities: 17,
        strikeAttempts: 18,
        strikeLosses: 19,
        throwAttempts: 20,
        throwLosses: 21,
      },
      burnout: {
        count: 22,
        durationDeciseconds: 123,
        hpLostBp: 2345,
        hpDealtBp: 3456,
        selfInitiated: 23,
        forced: 24,
        mixed: 25,
        unknown: 26,
      },
    });
  });

  test("不正な元値をfield名付きで拒否する", () => {
    const fields: [keyof TacticStats, string][] = [
      ["anti_air_opportunities", "antiAir.opportunities"],
      ["anti_air_successes", "antiAir.successes"],
      ["jump_ins_allowed", "antiAir.jumpInsAllowed"],
      ["di_faced", "driveImpact.faced"],
      ["di_returned", "driveImpact.returned"],
      ["di_blocked", "driveImpact.blocked"],
      ["di_parried", "driveImpact.parried"],
      ["di_hit", "driveImpact.hit"],
      ["di_avoided", "driveImpact.avoided"],
      ["di_unconfirmed", "driveImpact.unconfirmed"],
      ["raw_drive_rushes_faced", "rawDriveRush.faced"],
      ["raw_drive_rushes_defended", "rawDriveRush.defended"],
      ["raw_drive_rushes_hit", "rawDriveRush.hit"],
      ["raw_drive_rushes_unconfirmed", "rawDriveRush.unconfirmed"],
      ["dash_throws_faced", "dashThrow.faced"],
      ["throw_whiffs", "throwWhiff.count"],
      ["minus_defense_opportunities", "fastestChallenge.opportunities"],
      ["fastest_strike_challenges", "fastestChallenge.strikeAttempts"],
      ["fastest_strike_losses", "fastestChallenge.strikeLosses"],
      ["fastest_throw_challenges", "fastestChallenge.throwAttempts"],
      ["fastest_throw_losses", "fastestChallenge.throwLosses"],
      ["burnout_count", "burnout.count"],
      ["burnout_seconds", "burnout.durationDeciseconds"],
      ["burnout_hp_lost", "burnout.hpLostBp"],
      ["burnout_hp_dealt", "burnout.hpDealtBp"],
      ["burnout_self_initiated", "burnout.selfInitiated"],
      ["burnout_forced", "burnout.forced"],
      ["burnout_mixed", "burnout.mixed"],
      ["burnout_unknown", "burnout.unknown"],
    ];

    for (const [sourceField, projectedField] of fields) {
      expect(() =>
        projectPublishedTactics(stats({ [sourceField]: -1 })),
      ).toThrow(`${projectedField} が不正です。`);
    }
    expect(() =>
      projectPublishedTactics(stats({ burnout_hp_lost: Number.NaN })),
    ).toThrow(ShareProjectionError);
  });
});
