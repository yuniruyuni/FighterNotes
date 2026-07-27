import { Play } from "lucide-react";
import type { AdviceCard } from "~/modules/analysis/contracts.js";
import { frameToSeconds } from "../../domain/frame-time.js";
import type { SceneSelection } from "../../domain/scene-selection.js";

interface CardAdviceProps {
  card: AdviceCard;
  frameTimestamps: readonly number[];
  onSceneChange(scene: Omit<SceneSelection, "key">): void;
}

export function CardAdvice({
  card,
  frameTimestamps,
  onSceneChange,
}: CardAdviceProps) {
  return (
    <>
      <strong>【{card.title}】</strong>
      <br />
      {card.description}
      <br />
      <br />📝 {card.practice}
      <br />
      <br />
      {card.evidence.map((evidence) => (
        <button
          type="button"
          className="evidence-chip"
          key={`${evidence.frame}-${evidence.end_frame ?? ""}-${evidence.label}`}
          onClick={() =>
            onSceneChange({
              frame: evidence.frame,
              card,
              endFrame: evidence.end_frame,
            })
          }
        >
          <Play size={12} aria-hidden="true" />
          {evidence.label} ({formatEvidenceRange(evidence, frameTimestamps)})
        </button>
      ))}
    </>
  );
}

function formatEvidenceRange(
  evidence: AdviceCard["evidence"][number],
  timestamps: readonly number[],
): string {
  const start = frameToSeconds(evidence.frame, timestamps).toFixed(1);
  return evidence.end_frame == null
    ? `${start}s`
    : `${start}s-${frameToSeconds(evidence.end_frame, timestamps).toFixed(1)}s`;
}
