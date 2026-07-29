import type { InputStats, TacticStats } from "~/modules/analysis/contracts.js";
import {
  appendUnconfirmedCandidates,
  formatTacticCount,
} from "./tactic-stat-format.js";

export function InputStatsSection({ stats }: { stats: InputStats | null }) {
  return (
    <section className="summary-section" data-wm="Stats">
      <h2>入力習慣の統計</h2>
      {stats ? (
        <StatGrid
          items={[
            [
              `${stats.jumps} 回（${stats.jumps_per_min.toFixed(1)}/分）`,
              "ジャンプ",
            ],
            [`${stats.jump_got_hit} 回`, "ジャンプを落とされた"],
            [`${stats.jump_landed} 回`, "飛びを通した"],
            [`${stats.throw_hits} / ${stats.throw_attempts}`, "投げ 成功/試行"],
            [`${stats.button_presses} 回`, "ボタン押下"],
            [`${Math.round(stats.auto_ratio * 100)}%`, "AUTO 使用率"],
            [`${stats.di_presses} 回`, "DI 使用"],
            [`${Math.round(stats.crouch_ratio * 100)}%`, "しゃがみ時間比率"],
          ]}
        />
      ) : (
        <p className="muted-note">
          入力履歴を読み取れませんでした（入力履歴表示 ON
          のリプレイ録画が必要です）。
        </p>
      )}
    </section>
  );
}

export function TacticStatsSection({ stats }: { stats: TacticStats }) {
  const burnoutBalance = Math.round(
    (stats.burnout_hp_dealt - stats.burnout_hp_lost) * 100,
  );
  const balancePrefix = burnoutBalance > 0 ? "+" : "";
  const items: Array<[string, string, string?]> = [
    [
      formatTacticCount(stats.anti_air_successes, stats.anti_air_opportunities),
      "対空 成功 / 機会",
      `飛びを通された ${stats.jump_ins_allowed} 回`,
    ],
    [
      formatTacticCount(
        stats.di_returned,
        stats.di_faced,
        stats.di_unconfirmed,
      ),
      "DI返し / 相手DI",
      appendUnconfirmedCandidates(
        `ガード ${stats.di_blocked} / パリィ ${stats.di_parried} / 被弾 ${stats.di_hit}`,
        stats.di_unconfirmed,
      ),
    ],
    [
      formatTacticCount(
        stats.raw_drive_rushes_defended,
        stats.raw_drive_rushes_faced,
        stats.raw_drive_rushes_unconfirmed,
      ),
      "生ラッシュ対処 / 相手の生ラッシュ",
      appendUnconfirmedCandidates(
        `被弾 ${stats.raw_drive_rushes_hit} 回`,
        stats.raw_drive_rushes_unconfirmed,
      ),
    ],
    [`${stats.dash_throws_faced} 回`, "前ステップ投げを受けた"],
    [`${stats.throw_whiffs} 回`, "自分の投げ空振り"],
    [
      formatTacticCount(
        stats.fastest_strike_losses,
        stats.fastest_strike_challenges,
      ),
      "最速打撃の被弾 / 試行",
      `入力確認済みの不利状況 ${stats.minus_defense_opportunities ?? 0} 回`,
    ],
    [
      formatTacticCount(
        stats.fastest_throw_losses,
        stats.fastest_throw_challenges,
      ),
      "最速投げの被弾 / 試行",
      `入力確認済みの不利状況 ${stats.minus_defense_opportunities ?? 0} 回`,
    ],
    [
      `${stats.burnout_count} 回・${stats.burnout_seconds.toFixed(1)} 秒`,
      "バーンアウト",
      `与ダメ ${Math.round(stats.burnout_hp_dealt * 100)}% / 被ダメ ${Math.round(stats.burnout_hp_lost * 100)}% / 収支 ${balancePrefix}${burnoutBalance}%・自分の使用 ${stats.burnout_self_initiated} / ガード削り ${stats.burnout_forced} / 混在 ${stats.burnout_mixed} / 保留 ${stats.burnout_unknown}`,
    ],
  ];
  return (
    <section className="summary-section" data-wm="Tactics">
      <h2>戦術別の結果</h2>
      <StatGrid items={items} />
    </section>
  );
}

function StatGrid({ items }: { items: Array<[string, string, string?]> }) {
  return (
    <div className="stat-grid">
      {items.map(([value, label, detail]) => (
        <div className="stat-item" key={label}>
          <div className="sv">{value}</div>
          <div className="sl">{label}</div>
          {detail && <div className="sd">{detail}</div>}
        </div>
      ))}
    </div>
  );
}
