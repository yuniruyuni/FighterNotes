import { Play } from "lucide-react";
import type { DamageOriginRow } from "../../domain/damage-origin.js";
import { frameToSeconds } from "../../domain/frame-time.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import {
  attackAttributeLabel,
  attackEvidenceSceneRange,
  attackEvidenceStatus,
  attackEvidenceStatusLabel,
  formatAttackEvidenceAria,
  formatInteger,
  formatSignedInteger,
} from "./attack-evidence-format.js";
import {
  confidenceLabel,
  formatDamageContexts,
  formatHpRatio,
  formatPercent,
} from "./damage-origin-format.js";

interface DamageOriginDetailsProps {
  rows: readonly DamageOriginRow[];
  frameTimestamps: readonly number[];
  onSceneChange(scene: Omit<SceneSelection, "key">): void;
}

export function DamageOriginDetails({
  rows,
  frameTimestamps,
  onSceneChange,
}: DamageOriginDetailsProps) {
  return (
    <ol className="damage-origin-list">
      {rows.map((row) => (
        <li key={row.key}>
          <div className="damage-origin-row">
            <span className="damage-origin-swatch" data-origin={row.key} />
            <strong>{row.label}</strong>
            <span className="damage-origin-total">
              {formatHpRatio(row.hpLost)}
            </span>
          </div>
          <div className="damage-origin-detail">
            構成比 {formatPercent(row.compositionPercent)}・{row.events.length}
            件・10,000換算{" "}
            {Math.round(row.hpLost * 10_000).toLocaleString("ja-JP")}
          </div>
          <div className="damage-origin-scenes">
            {row.events.map((event) => {
              const contexts = formatDamageContexts(event.contexts);
              const confidence = confidenceLabel(
                event.strike_kind_confidence ?? event.confidence,
              );
              const label = `${row.label}・R${event.round_no}・${formatHpRatio(event.hp_drop)}`;
              const evidence = event.attack_evidence;
              const sceneRange = attackEvidenceSceneRange(event);
              const evidenceAria = evidence
                ? `。${formatAttackEvidenceAria(evidence)}`
                : "";
              const ariaLabel = `${label}。判定確度 ${confidence}${
                contexts ? `。状況 ${contexts}` : ""
              }${evidenceAria}。動画で確認`;
              return (
                <div
                  className="damage-scene"
                  key={`${event.round_no}-${event.sequence_no}`}
                >
                  <button
                    type="button"
                    className="damage-scene-button"
                    aria-label={ariaLabel}
                    title={ariaLabel}
                    onClick={() =>
                      onSceneChange({
                        ...sceneRange,
                        card: null,
                        label,
                      })
                    }
                  >
                    <Play size={13} aria-hidden="true" />
                    <span>R{event.round_no}</span>
                    <span>{formatHpRatio(event.hp_drop)}</span>
                    <span>
                      {frameToSeconds(
                        sceneRange.frame,
                        frameTimestamps,
                      ).toFixed(1)}
                      s
                    </span>
                  </button>
                  {evidence ? (
                    <AttackEvidenceDetails
                      evidence={evidence}
                      hpDrop={event.hp_drop}
                    />
                  ) : null}
                </div>
              );
            })}
          </div>
        </li>
      ))}
    </ol>
  );
}

function AttackEvidenceDetails({
  evidence,
  hpDrop,
}: {
  evidence: NonNullable<DamageOriginRow["events"][number]["attack_evidence"]>;
  hpDrop: number;
}) {
  const status = attackEvidenceStatus(evidence);
  const hpEquivalent = Math.round(hpDrop * 10_000);
  const difference = evidence.combo_damage - hpEquivalent;
  return (
    <details className="attack-evidence-details" data-status={status}>
      <summary>
        <span className="attack-evidence-status" data-status={status}>
          {attackEvidenceStatusLabel(status)}
        </span>
        <span>ゲーム内表示 {formatInteger(evidence.combo_damage)}</span>
      </summary>
      <dl>
        <div>
          <dt>ゲーム内表示の累積damage</dt>
          <dd>{formatInteger(evidence.combo_damage)}</dd>
        </div>
        <div>
          <dt>HPバーの標準10,000換算</dt>
          <dd>{formatInteger(hpEquivalent)}</dd>
        </div>
        <div>
          <dt>表示damageとの差</dt>
          <dd>{formatSignedInteger(difference)}</dd>
        </div>
        <div>
          <dt>帰属した攻撃連係</dt>
          <dd>{formatInteger(evidence.sequence_count)}件</dd>
        </div>
        <div>
          <dt>最終補正</dt>
          <dd>{formatInteger(evidence.final_scaling_percent)}%</dd>
        </div>
        <div>
          <dt>攻撃属性</dt>
          <dd>
            始動 {attackAttributeLabel(evidence.starter_attribute)} → 最終{" "}
            {attackAttributeLabel(evidence.final_attribute)}
          </dd>
        </div>
        <div>
          <dt>認識確度</dt>
          <dd>{confidenceLabel(evidence.confidence)}</dd>
        </div>
        <div>
          <dt>HP照合</dt>
          <dd>{attackEvidenceStatusLabel(evidence.hp_consistency)}</dd>
        </div>
      </dl>
      <p>
        {status === "incomplete"
          ? "表示の一部を補完した参考値です。断定的な指摘には使用していません。"
          : status === "mismatch"
            ? "ゲーム内表示とHPバー推定が一致しません。この場面の動画を確認してください。"
            : status === "unverified"
              ? "HPバー推定と照合できていないため、表示値は参考値です。"
              : "ゲーム内表示とHPバー推定の許容差内です。"}
      </p>
    </details>
  );
}
