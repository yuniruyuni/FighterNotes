import { Play } from "lucide-react";
import type {
  AdviceCard,
  AdviceReport,
  EvidenceClip,
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
  return (
    <section className="summary-section" data-wm="Weak Points">
      <h2>指摘事項</h2>
      {cards.length === 0 ? (
        <p className="muted-note">顕著な改善ポイントは検出されませんでした。</p>
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
    </section>
  );
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

  return (
    <article className="advice-card">
      <div className="ac-title">
        {card.title}{" "}
        <span className="count-note">
          {metadata && `【${metadata}】 `}({card.evidence.length}件)
        </span>
      </div>
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
