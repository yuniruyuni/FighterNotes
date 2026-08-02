import type {
  AnalysisAvailability,
  AnalysisCoverage,
  InputStats,
  TacticStats,
} from "~/modules/analysis/contracts.js";
import {
  appendUnconfirmedCandidates,
  formatTacticCount,
} from "./tactic-stat-format.js";

const MIN_DETECTOR_COVERAGE_PERCENT = 60;
const MIN_SUPER_COVERAGE_PERCENT = 20;
const MIN_SPATIAL_COVERAGE_PERCENT = 20;

function coverageRatioIsSufficient(
  observed: number | undefined,
  total: number,
  requiredPercent: number,
): boolean {
  // availabilityが無いruleset v8以前だけの互換計算。新レポートの分母0は
  // Rust側の明示的なunavailable/not_applicableを使う。
  return total === 0 || (observed ?? 0) * 100 >= total * requiredPercent;
}

function detectorCoverageIsSufficient(
  coverage: AnalysisCoverage | undefined,
  key: keyof AnalysisAvailability,
  observed: number | undefined,
): boolean {
  const explicit = coverage?.availability?.[key];
  if (explicit !== undefined) return explicit === "available";
  const total = coverage?.detector_match_frames ?? 0;
  return coverageRatioIsSufficient(
    observed,
    total,
    MIN_DETECTOR_COVERAGE_PERCENT,
  );
}

function superCoverageIsSufficient(
  coverage: AnalysisCoverage | undefined,
  key: "own_super" | "opponent_super",
  observed: number | undefined,
): boolean {
  const explicit = coverage?.availability?.[key];
  if (explicit !== undefined) return explicit === "available";
  const total = coverage?.detector_match_frames ?? 0;
  return coverageRatioIsSufficient(observed, total, MIN_SUPER_COVERAGE_PERCENT);
}

function explicitOrLegacyAvailability(
  coverage: AnalysisCoverage | undefined,
  key: keyof AnalysisAvailability,
  legacy: () => boolean,
  notApplicableIsUsable = false,
): boolean {
  const explicit = coverage?.availability?.[key];
  return explicit === undefined
    ? legacy()
    : explicit === "available" ||
        (notApplicableIsUsable && explicit === "not_applicable");
}

type StatItem = [string, string, string?];

function coverageAwareItem(
  available: boolean,
  item: StatItem,
  observedOpportunities: number,
  unavailableReason: string,
): StatItem {
  if (available) return item;
  const observed =
    observedOpportunities > 0
      ? `少なくとも ${observedOpportunities} 件`
      : "確認不能";
  const known =
    observedOpportunities > 0
      ? ` 確認できた範囲: ${item[0]}${item[2] ? `（${item[2]}）` : ""}`
      : "";
  return [observed, item[1], `${unavailableReason}${known}`];
}

export function InputStatsSection({
  stats,
  coverage,
}: {
  stats: InputStats | null;
  coverage?: AnalysisCoverage;
}) {
  const inputCoverageMissing = !detectorCoverageIsSufficient(
    coverage,
    "own_input",
    coverage?.own_input_observed_frames,
  );
  const ownHpAvailable = detectorCoverageIsSufficient(
    coverage,
    "own_hp",
    coverage?.own_hp_reliable_frames,
  );
  const opponentHpAvailable = detectorCoverageIsSufficient(
    coverage,
    "opponent_hp",
    coverage?.opponent_hp_reliable_frames,
  );
  const meterAvailable =
    detectorCoverageIsSufficient(
      coverage,
      "own_meter",
      coverage?.own_meter_mapped_frames,
    ) &&
    detectorCoverageIsSufficient(
      coverage,
      "opponent_meter",
      coverage?.opponent_meter_mapped_frames,
    );
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
            [
              ownHpAvailable ? `${stats.jump_got_hit} 回` : "確認不能",
              "ジャンプを落とされた",
            ],
            [
              opponentHpAvailable ? `${stats.jump_landed} 回` : "確認不能",
              "飛びを通した",
            ],
            [
              opponentHpAvailable && meterAvailable
                ? `${stats.throw_hits} / ${stats.throw_attempts}`
                : "確認不能",
              "投げ 成功/試行",
            ],
            [`${stats.button_presses} 回`, "ボタン押下"],
            [`${Math.round(stats.auto_ratio * 100)}%`, "AUTO 使用率"],
            [`${stats.di_presses} 回`, "DI 使用"],
            [`${Math.round(stats.crouch_ratio * 100)}%`, "しゃがみ時間比率"],
          ]}
        />
      ) : (
        <p className="muted-note">
          {inputCoverageMissing
            ? "自分の入力履歴を十分に直接観測できなかったため、入力習慣の数値は確認不能です。"
            : "入力履歴を読み取れませんでした（入力履歴表示 ON のリプレイ録画が必要です）。"}
        </p>
      )}
    </section>
  );
}

export function TacticStatsSection({
  stats,
  coverage,
}: {
  stats: TacticStats;
  coverage?: AnalysisCoverage;
}) {
  const burnoutBalance = Math.round(
    (stats.burnout_hp_dealt - stats.burnout_hp_lost) * 100,
  );
  const balancePrefix = burnoutBalance > 0 ? "+" : "";
  const superUsed =
    (stats.sa1_used ?? 0) +
    (stats.sa2_used ?? 0) +
    (stats.sa3_used ?? 0) +
    (stats.ca_used ?? 0);
  const opponentSuperUsed =
    (stats.opponent_sa1_used ?? 0) +
    (stats.opponent_sa2_used ?? 0) +
    (stats.opponent_sa3_used ?? 0) +
    (stats.opponent_ca_used ?? 0);
  const ownInputAvailable = detectorCoverageIsSufficient(
    coverage,
    "own_input",
    coverage?.own_input_observed_frames,
  );
  const opponentInputAvailable = detectorCoverageIsSufficient(
    coverage,
    "opponent_input",
    coverage?.opponent_input_observed_frames,
  );
  const ownHpAvailable = detectorCoverageIsSufficient(
    coverage,
    "own_hp",
    coverage?.own_hp_reliable_frames,
  );
  const opponentHpAvailable = detectorCoverageIsSufficient(
    coverage,
    "opponent_hp",
    coverage?.opponent_hp_reliable_frames,
  );
  const meterAvailable =
    detectorCoverageIsSufficient(
      coverage,
      "own_meter",
      coverage?.own_meter_mapped_frames,
    ) &&
    detectorCoverageIsSufficient(
      coverage,
      "opponent_meter",
      coverage?.opponent_meter_mapped_frames,
    );
  const driveAvailable = detectorCoverageIsSufficient(
    coverage,
    "own_drive",
    coverage?.own_drive_reliable_frames,
  );
  const opponentDriveAvailable = detectorCoverageIsSufficient(
    coverage,
    "opponent_drive",
    coverage?.opponent_drive_reliable_frames,
  );
  const ownSuperAvailable = superCoverageIsSufficient(
    coverage,
    "own_super",
    coverage?.own_super_reliable_frames,
  );
  const opponentSuperAvailable = superCoverageIsSufficient(
    coverage,
    "opponent_super",
    coverage?.opponent_super_reliable_frames,
  );
  const attackInfoAvailable = explicitOrLegacyAvailability(
    coverage,
    "own_attack_info",
    () =>
      coverageRatioIsSufficient(
        coverage?.attack_damage_linked,
        coverage?.attack_damage_events ?? 0,
        MIN_DETECTOR_COVERAGE_PERCENT,
      ),
  );
  const spatialCandidates = coverage?.spatial_candidate_frames ?? 0;
  const spatialAvailable = explicitOrLegacyAvailability(
    coverage,
    "spatial",
    () =>
      coverageRatioIsSufficient(
        coverage?.spatial_sampled_frames,
        spatialCandidates,
        MIN_DETECTOR_COVERAGE_PERCENT,
      ) &&
      coverageRatioIsSufficient(
        coverage?.spatial_usable_frames,
        spatialCandidates,
        MIN_SPATIAL_COVERAGE_PERCENT,
      ),
    true,
  );
  const contactsAvailable = explicitOrLegacyAvailability(
    coverage,
    "contacts",
    () => meterAvailable && ownHpAvailable && opponentHpAvailable,
  );
  const punishesAvailable = explicitOrLegacyAvailability(
    coverage,
    "punishes",
    () => contactsAvailable && ownInputAvailable && opponentInputAvailable,
  );
  const items: StatItem[] = [
    coverageAwareItem(
      opponentInputAvailable && ownHpAvailable && opponentHpAvailable,
      [
        formatTacticCount(
          stats.anti_air_successes,
          stats.anti_air_opportunities,
        ),
        "対空 成功 / 機会",
        `飛びを通された ${stats.jump_ins_allowed} 回`,
      ],
      stats.anti_air_opportunities,
      "相手入力または両者のHPバーの認識率が不足しています。",
    ),
    coverageAwareItem(
      ownInputAvailable &&
        opponentInputAvailable &&
        meterAvailable &&
        contactsAvailable &&
        ownHpAvailable &&
        opponentHpAvailable,
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
      stats.di_faced + stats.di_unconfirmed,
      "自分・相手の入力、フレームメーター、接触、または両者のHPバーの認識率が不足しています。",
    ),
    coverageAwareItem(
      opponentInputAvailable &&
        opponentDriveAvailable &&
        meterAvailable &&
        contactsAvailable &&
        ownHpAvailable &&
        spatialAvailable,
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
      stats.raw_drive_rushes_faced + stats.raw_drive_rushes_unconfirmed,
      "相手入力・Driveゲージ、フレームメーター、接触、自分のHPバー、または空間解析の認識率が不足しています。",
    ),
    coverageAwareItem(
      opponentInputAvailable &&
        meterAvailable &&
        contactsAvailable &&
        ownHpAvailable &&
        spatialAvailable,
      [`${stats.dash_throws_faced} 回`, "前ステップ投げを受けた"],
      stats.dash_throws_faced,
      "相手入力、フレームメーター、自分のHPバー、または空間解析の認識率が不足しています。",
    ),
    coverageAwareItem(
      ownInputAvailable && meterAvailable,
      [`${stats.throw_whiffs} 回`, "自分の投げ空振り"],
      stats.throw_whiffs,
      "自分の入力またはフレームメーターの認識率が不足しています。",
    ),
    coverageAwareItem(
      ownInputAvailable &&
        meterAvailable &&
        contactsAvailable &&
        ownHpAvailable,
      [
        formatTacticCount(
          stats.fastest_strike_losses,
          stats.fastest_strike_challenges,
        ),
        "最速打撃の被弾 / 試行",
        `入力確認済みの不利状況 ${stats.minus_defense_opportunities ?? 0} 回`,
      ],
      stats.minus_defense_opportunities ?? 0,
      "自分の入力、フレームメーター、または自分のHPバーの認識率が不足しています。",
    ),
    coverageAwareItem(
      ownInputAvailable &&
        meterAvailable &&
        contactsAvailable &&
        ownHpAvailable,
      [
        formatTacticCount(
          stats.fastest_throw_losses,
          stats.fastest_throw_challenges,
        ),
        "最速投げの被弾 / 試行",
        `入力確認済みの不利状況 ${stats.minus_defense_opportunities ?? 0} 回`,
      ],
      stats.minus_defense_opportunities ?? 0,
      "自分の入力、フレームメーター、または自分のHPバーの認識率が不足しています。",
    ),
    coverageAwareItem(
      driveAvailable && ownHpAvailable && opponentHpAvailable,
      [
        `${stats.burnout_count} 回・${stats.burnout_seconds.toFixed(1)} 秒`,
        "バーンアウト",
        `与ダメ ${Math.round(stats.burnout_hp_dealt * 100)}% / 被ダメ ${Math.round(stats.burnout_hp_lost * 100)}% / 収支 ${balancePrefix}${burnoutBalance}%・自分の使用 ${stats.burnout_self_initiated} / ガード削り ${stats.burnout_forced} / 混在 ${stats.burnout_mixed} / 保留 ${stats.burnout_unknown}`,
      ],
      stats.burnout_count,
      "Driveゲージまたは両者のHPバーの認識率が不足しています。",
    ),
  ];
  if (stats.sa1_used !== undefined) {
    const hasDetectorCoverage = (coverage?.detector_match_frames ?? 0) > 0;
    const hasExplicitAvailability = coverage?.availability !== undefined;
    const ownSuperEndAvailable =
      ownSuperAvailable &&
      (coverage?.own_super_end_reliable === true ||
        (!hasExplicitAvailability && !hasDetectorCoverage));
    const opponentSuperEndAvailable =
      opponentSuperAvailable &&
      (coverage?.opponent_super_end_reliable === true ||
        (!hasExplicitAvailability && !hasDetectorCoverage));
    const superOutcomeAvailable =
      contactsAvailable &&
      punishesAvailable &&
      ownHpAvailable &&
      opponentHpAvailable;
    const ownSuperOutcomeDetail = superOutcomeAvailable
      ? `SA1 ${stats.sa1_used ?? 0} / SA2 ${stats.sa2_used ?? 0} / SA3 ${stats.sa3_used ?? 0} / CA ${stats.ca_used ?? 0}・ヒット ${stats.super_hits ?? 0} / ガード ${stats.super_blocked ?? 0} / 即時接触なし ${stats.super_no_immediate_contact ?? 0} / 反撃を受けた ${stats.super_punished ?? 0} / KO ${stats.super_kos ?? 0}`
      : `SA1 ${stats.sa1_used ?? 0} / SA2 ${stats.sa2_used ?? 0} / SA3 ${stats.sa3_used ?? 0} / CA ${stats.ca_used ?? 0}・接触、確反、またはHPバーの認識率不足のため結果内訳は確認不能`;
    const opponentSuperOutcomeDetail = superOutcomeAvailable
      ? `SA1 ${stats.opponent_sa1_used ?? 0} / SA2 ${stats.opponent_sa2_used ?? 0} / SA3 ${stats.opponent_sa3_used ?? 0} / CA ${stats.opponent_ca_used ?? 0}・ヒット ${stats.opponent_super_hits ?? 0} / ガード ${stats.opponent_super_blocked ?? 0} / 即時接触なし ${stats.opponent_super_no_immediate_contact ?? 0} / 反撃を受けた ${stats.opponent_super_punished ?? 0} / KO ${stats.opponent_super_kos ?? 0}`
      : `SA1 ${stats.opponent_sa1_used ?? 0} / SA2 ${stats.opponent_sa2_used ?? 0} / SA3 ${stats.opponent_sa3_used ?? 0} / CA ${stats.opponent_ca_used ?? 0}・接触、確反、またはHPバーの認識率不足のため結果内訳は確認不能`;
    items.push(
      coverageAwareItem(
        ownSuperAvailable,
        [`${superUsed} 回`, "自分のSA / CA", ownSuperOutcomeDetail],
        superUsed,
        "自分のSAゲージ認識率が不足しているため、全使用回数は確認不能です。",
      ),
      coverageAwareItem(
        ownSuperAvailable &&
          meterAvailable &&
          contactsAvailable &&
          punishesAvailable,
        [
          `コンボ ${stats.super_combo_uses ?? 0} / 確反 ${stats.super_punish_uses ?? 0} / 切り返し ${stats.super_reversal_uses ?? 0} / 単発 ${stats.super_neutral_uses ?? 0}`,
          "SAを使った文脈",
        ],
        superUsed,
        "自分のSAゲージ、フレームメーター、接触、または確反解析の認識率が不足しているため、全使用文脈は確認不能です。",
      ),
      ...((stats.super_damage_samples ?? 0) > 0 ||
      (superUsed > 0 &&
        (coverage?.own_attack_damage_events ??
          coverage?.attack_damage_events ??
          0) > 0 &&
        !attackInfoAvailable)
        ? ([
            coverageAwareItem(
              ownSuperAvailable && opponentHpAvailable && attackInfoAvailable,
              [
                `${stats.super_reported_marginal_damage ?? 0} ダメージ`,
                "SA投入後の表示ダメージ",
                `ゲーム内表示を帰属できた ${stats.super_damage_samples ?? 0} 回・コンボ全体 ${stats.super_reported_combo_damage ?? 0}・投入時50%以下かつ非KO ${stats.super_low_scaling_uses ?? 0} 回`,
              ],
              stats.super_damage_samples ?? 0,
              "SAゲージ、相手HPバー、または中央攻撃表示の認識率が不足しているため、確認済みの使用だけを示します。",
            ),
          ] satisfies StatItem[])
        : []),
      coverageAwareItem(
        opponentSuperAvailable,
        [
          `${opponentSuperUsed} 回`,
          "相手のSA / CA",
          opponentSuperOutcomeDetail,
        ],
        opponentSuperUsed,
        "相手のSAゲージ認識率が不足しているため、全使用回数は確認不能です。",
      ),
      [
        `自分 ${ownSuperEndAvailable ? (stats.super_gauge_end ?? 0).toFixed(1) : "確認不能"} / 相手 ${opponentSuperEndAvailable ? (stats.opponent_super_gauge_end ?? 0).toFixed(1) : "確認不能"}`,
        "最終確認時のSAゲージ",
        ownSuperEndAvailable && opponentSuperEndAvailable
          ? "未使用量は事実として表示し、使うべきだったとは断定しません"
          : "最終フレームのSAゲージを確実に読めた側だけを表示します",
      ],
    );
  }
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
