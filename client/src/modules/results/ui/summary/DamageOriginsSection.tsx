import { useMemo, useState } from "react";
import type {
  AnalysisCoverage,
  DamageBreakdown,
  RoundSummary,
} from "~/modules/analysis/contracts.js";
import { summarizeDamageOrigins } from "../../domain/damage-origin.js";
import { frameToSeconds } from "../../domain/frame-time.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import {
  attackEvidenceSceneRange,
  attackEvidenceStatus,
} from "./attack-evidence-format.js";
import { DamageOriginDetails } from "./DamageOriginDetails.js";
import { formatHpRatio, formatPercent } from "./damage-origin-format.js";

interface DamageOriginsSectionProps {
  breakdown: DamageBreakdown | undefined;
  coverage?: AnalysisCoverage;
  rounds: readonly RoundSummary[];
  frameTimestamps: readonly number[];
  onSceneChange(scene: Omit<SceneSelection, "key">): void;
}

export function DamageOriginsSection({
  breakdown,
  coverage,
  rounds,
  frameTimestamps,
  onSceneChange,
}: DamageOriginsSectionProps) {
  const [selectedRound, setSelectedRound] = useState<"all" | number>("all");
  const roundNumbers = useMemo(
    () =>
      [
        ...new Set([
          ...rounds.map((round) => round.round_no),
          ...(breakdown?.events ?? []).map((event) => event.round_no),
        ]),
      ].sort((a, b) => a - b),
    [breakdown, rounds],
  );
  const summary = useMemo(
    () => summarizeDamageOrigins(breakdown?.events ?? [], selectedRound),
    [breakdown, selectedRound],
  );
  const mismatchEvents = (breakdown?.events ?? []).filter((event) => {
    const evidence = event.attack_evidence;
    return (
      (selectedRound === "all" || event.round_no === selectedRound) &&
      evidence !== undefined &&
      attackEvidenceStatus(evidence) === "mismatch"
    );
  });

  if (!breakdown) return null;

  const hpAvailable =
    coverage?.availability === undefined ||
    coverage.availability.own_hp === "available";

  const scopeLabel =
    selectedRound === "all"
      ? roundNumbers.length > 0
        ? `全${roundNumbers.length}R合計`
        : "試合全体"
      : `R${selectedRound}`;
  const chartLabel = summary.rows
    .map((row) => `${row.label} ${formatPercent(row.compositionPercent)}`)
    .join("、");

  return (
    <section
      className="summary-section damage-origins"
      data-wm="Damage Sources"
    >
      <div className="damage-origin-heading">
        <h2>被ダメージの起点</h2>
        <fieldset className="damage-round-selector">
          <legend>集計ラウンド</legend>
          <button
            type="button"
            aria-pressed={selectedRound === "all"}
            onClick={() => setSelectedRound("all")}
          >
            全体
          </button>
          {roundNumbers.map((round) => (
            <button
              type="button"
              key={round}
              aria-pressed={selectedRound === round}
              onClick={() => setSelectedRound(round)}
            >
              R{round}
            </button>
          ))}
        </fieldset>
      </div>

      {!hpAvailable ? (
        <p className="muted-note">
          自分のHPバーを十分に認識できなかったため、被ダメージ量・件数・起点は確認不能です。
        </p>
      ) : summary.totalHpLost <= 0 ? (
        <p className="muted-note">被ダメージは検出されませんでした。</p>
      ) : (
        <>
          {mismatchEvents.length > 0 ? (
            <div className="attack-evidence-warning" role="note">
              <p>
                ⚠ ゲーム内表示damageとHPバー推定が一致しない場面が
                {mismatchEvents.length}
                件あります。表示値を断定に使わず、動画で確認してください。
              </p>
              <div>
                {mismatchEvents.map((event) => {
                  const label = `HP表示不一致・R${event.round_no}・${formatHpRatio(event.hp_drop)}`;
                  const sceneRange = attackEvidenceSceneRange(event);
                  const time = frameToSeconds(
                    sceneRange.frame,
                    frameTimestamps,
                  ).toFixed(1);
                  return (
                    <button
                      type="button"
                      key={`${event.round_no}-${event.sequence_no}`}
                      aria-label={`${label}。${time}秒を動画で確認`}
                      onClick={() =>
                        onSceneChange({
                          ...sceneRange,
                          card: null,
                          label,
                        })
                      }
                    >
                      R{event.round_no}・{time}s
                    </button>
                  );
                })}
              </div>
            </div>
          ) : null}
          <div className="damage-origin-overview">
            <DamageMetric
              value={formatHpRatio(summary.totalHpLost)}
              label={`最大体力比・${scopeLabel}`}
            />
            <DamageMetric
              value={Math.round(summary.totalHpLost * 10_000).toLocaleString(
                "ja-JP",
              )}
              label="標準体力10,000換算"
            />
            <DamageMetric
              value={formatPercent(summary.classifiedPercent)}
              label="分類済みダメージ"
            />
            <DamageMetric
              value={`${summary.rows.reduce((sum, row) => sum + row.events.length, 0)}件`}
              label="被弾シーン"
            />
          </div>

          <div
            className="damage-origin-chart"
            role="img"
            aria-label={`${scopeLabel}の被ダメージ構成。${chartLabel}`}
          >
            {summary.rows.map((row) => (
              <span
                key={row.key}
                data-origin={row.key}
                style={{ width: `${row.compositionPercent}%` }}
                title={`${row.label} ${formatPercent(row.compositionPercent)}`}
              />
            ))}
          </div>

          <DamageOriginDetails
            rows={summary.rows}
            frameTimestamps={frameTimestamps}
            onSceneChange={onSceneChange}
          />
        </>
      )}
    </section>
  );
}

function DamageMetric({ value, label }: { value: string; label: string }) {
  return (
    <div className="damage-origin-metric">
      <strong>{value}</strong>
      <span>{label}</span>
    </div>
  );
}
