import { Play } from "lucide-react";
import type { DamageOriginRow } from "../../domain/damage-origin.js";
import { frameToSeconds } from "../../domain/frame-time.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
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
              const ariaLabel = `${label}。判定確度 ${confidence}${
                contexts ? `。状況 ${contexts}` : ""
              }。動画で確認`;
              return (
                <button
                  type="button"
                  className="damage-scene-button"
                  key={event.sequence_no}
                  aria-label={ariaLabel}
                  title={ariaLabel}
                  onClick={() =>
                    onSceneChange({
                      frame: event.scene_frame,
                      endFrame: event.end_frame,
                      card: null,
                      label,
                    })
                  }
                >
                  <Play size={13} aria-hidden="true" />
                  <span>R{event.round_no}</span>
                  <span>{formatHpRatio(event.hp_drop)}</span>
                  <span>
                    {frameToSeconds(event.scene_frame, frameTimestamps).toFixed(
                      1,
                    )}
                    s
                  </span>
                </button>
              );
            })}
          </div>
        </li>
      ))}
    </ol>
  );
}
