import { Play } from "lucide-react";
import type {
  AdviceCard,
  AdviceReport,
  EvidenceClip,
  EvidenceRequirement,
} from "~/modules/analysis/contracts.js";
import { frameToSeconds } from "../../domain/frame-time.js";
import type { SceneSelection } from "../../domain/scene-selection.js";

interface AdviceSectionProps {
  report: AdviceReport;
  frameTimestamps: readonly number[];
  onSceneChange(scene: Omit<SceneSelection, "key">): void;
}

export function AdviceSection({
  report,
  frameTimestamps,
  onSceneChange,
}: AdviceSectionProps) {
  const cards = report.cards ?? [];
  const suppressed = report.suppressed_cards ?? [];
  const evidenceIncomplete = Object.values(
    report.coverage?.availability ?? {},
  ).includes("unavailable");
  return (
    <section className="summary-section" data-wm="Weak Points">
      <h2>指摘事項</h2>
      {cards.length === 0 && suppressed.length === 0 ? (
        <p
          className="muted-note"
          role={evidenceIncomplete ? "note" : undefined}
        >
          {evidenceIncomplete
            ? "認識率不足のため、改善ポイントを十分に判定できませんでした。改善点がないという意味ではありません。"
            : "顕著な改善ポイントは検出されませんでした。"}
        </p>
      ) : (
        cards.map((card) => (
          <AdviceResultCard
            key={card.id}
            card={card}
            frameTimestamps={frameTimestamps}
            onSceneChange={onSceneChange}
          />
        ))
      )}
      {suppressed.length > 0 && (
        <div className="muted-note" role="note">
          <p>
            証拠の認識率不足により、{suppressed.length}
            件の指摘候補を確認不能として非表示にしています。改善点がないという意味ではありません。
          </p>
          <ul>
            {suppressed.map((card) => (
              <li key={card.id}>
                {card.title}:{" "}
                {card.missing_requirements.map(requirementLabel).join("・")}
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}

function requirementLabel(requirement: EvidenceRequirement): string {
  const labels: Record<EvidenceRequirement, string> = {
    own_hp: "自分のHPバー",
    opponent_hp: "相手のHPバー",
    own_drive: "自分のDriveゲージ",
    opponent_drive: "相手のDriveゲージ",
    own_super: "自分のSAゲージ",
    opponent_super: "相手のSAゲージ",
    own_input: "自分の入力履歴",
    opponent_input: "相手の入力履歴",
    frame_meter: "フレームメーター",
    contacts: "接触解析",
    punishes: "確反解析",
    spatial: "空間解析",
    own_attack_info: "自分の攻撃表示",
    opponent_attack_info: "相手の攻撃表示",
  };
  return labels[requirement];
}

function AdviceResultCard({
  card,
  frameTimestamps,
  onSceneChange,
}: {
  card: AdviceCard;
  frameTimestamps: readonly number[];
  onSceneChange(scene: Omit<SceneSelection, "key">): void;
}) {
  const metadata = [
    adviceKindLabel(card.kind),
    adviceConfidenceLabel(card.confidence),
  ]
    .filter(Boolean)
    .join("・");
  const hpLost = adviceHpLostLabel(card.hp_lost);

  return (
    <article className="advice-card">
      <div className="ac-title">
        {card.title}{" "}
        <span className="count-note">
          {metadata && `【${metadata}】 `}({card.evidence.length}件)
        </span>
      </div>
      {hpLost && <div className="ac-cost">この場面で失った体力 {hpLost}</div>}
      <div className="ac-desc">{card.description}</div>
      <div className="ac-practice">📝 {card.practice}</div>
      <div>
        {card.evidence.map((evidence) => (
          <EvidenceButton
            key={`${evidence.frame}-${evidence.end_frame ?? ""}-${evidence.label}`}
            evidence={evidence}
            frameTimestamps={frameTimestamps}
            onClick={() =>
              onSceneChange({
                frame: evidence.frame,
                card,
                endFrame: evidence.end_frame,
              })
            }
          />
        ))}
      </div>
    </article>
  );
}

function EvidenceButton({
  evidence,
  frameTimestamps,
  onClick,
}: {
  evidence: EvidenceClip;
  frameTimestamps: readonly number[];
  onClick(): void;
}) {
  const start = frameToSeconds(evidence.frame, frameTimestamps).toFixed(1);
  const range =
    evidence.end_frame == null
      ? `${start}s`
      : `${start}s-${frameToSeconds(
          evidence.end_frame,
          frameTimestamps,
        ).toFixed(1)}s`;
  return (
    <button type="button" className="evidence-chip" onClick={onClick}>
      <Play size={12} aria-hidden="true" />
      {evidence.label} ({range})
    </button>
  );
}

/**
 * 被ダメージが直接の結果である指摘だけが hp_lost を持つ。確反の取りこぼしの
 * ように損失が機会費用であるものは未設定で、0% とは意味が異なるため表示しない。
 */
function adviceHpLostLabel(hpLost: AdviceCard["hp_lost"]): string | undefined {
  if (hpLost === undefined || hpLost === null) return undefined;
  if (hpLost <= 0) return undefined;
  return `-${(hpLost * 100).toFixed(0)}%`;
}

function adviceKindLabel(kind: AdviceCard["kind"]): string | undefined {
  if (kind === "diagnosis") return "原因診断";
  if (kind === "observation") return "確認場面";
  if (kind === "statistic") return "統計";
  return undefined;
}

function adviceConfidenceLabel(
  confidence: AdviceCard["confidence"],
): string | undefined {
  if (confidence === "high") return "確度高";
  if (confidence === "medium") return "確度中";
  if (confidence === "low") return "確度低";
  return undefined;
}
