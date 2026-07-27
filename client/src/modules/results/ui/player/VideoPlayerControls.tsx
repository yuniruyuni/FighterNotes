import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Pause,
  Play,
  Repeat2,
} from "lucide-react";
import type { CSSProperties, ReactNode } from "react";
import { secondsToFrame } from "../../domain/frame-time.js";

interface VideoPlayerControlsProps {
  currentTime: number;
  duration: number;
  frameTimestamps: readonly number[];
  loopEnabled: boolean;
  playing: boolean;
  progressStyle: CSSProperties | undefined;
  onSeek(milliseconds: number): void;
  onStepFrame(delta: number): void;
  onToggleLoop(): void;
  onTogglePlayback(): void;
}

export function VideoPlayerControls(props: VideoPlayerControlsProps) {
  const progressMaximum = Math.max(1, Math.round(props.duration * 1000));
  return (
    <div className="player-ui">
      <div className="progress-row">
        <input
          id="player-progress"
          type="range"
          min={0}
          max={progressMaximum}
          value={Math.min(
            progressMaximum,
            Math.round(props.currentTime * 1000),
          )}
          step={1}
          style={props.progressStyle}
          aria-label="動画の再生位置"
          onChange={(event) => props.onSeek(Number(event.currentTarget.value))}
        />
        <span className="player-time">
          {props.currentTime.toFixed(2)}s / f
          {secondsToFrame(props.currentTime, props.frameTimestamps)}
        </span>
      </div>
      <div className="controls-row">
        <PlayerButton
          label="10フレーム戻る"
          onClick={() => props.onStepFrame(-10)}
        >
          <ChevronsLeft size={17} aria-hidden="true" />
          <span>10f</span>
        </PlayerButton>
        <PlayerButton
          label="1フレーム戻る"
          onClick={() => props.onStepFrame(-1)}
        >
          <ChevronLeft size={17} aria-hidden="true" />
          <span>1f</span>
        </PlayerButton>
        <PlayerButton
          label={props.playing ? "一時停止" : "再生"}
          className="play"
          onClick={props.onTogglePlayback}
        >
          {props.playing ? (
            <Pause size={18} aria-hidden="true" />
          ) : (
            <Play size={18} aria-hidden="true" />
          )}
        </PlayerButton>
        <PlayerButton
          label="1フレーム進む"
          onClick={() => props.onStepFrame(1)}
        >
          <span>1f</span>
          <ChevronRight size={17} aria-hidden="true" />
        </PlayerButton>
        <PlayerButton
          label="10フレーム進む"
          onClick={() => props.onStepFrame(10)}
        >
          <span>10f</span>
          <ChevronsRight size={17} aria-hidden="true" />
        </PlayerButton>
        <PlayerButton
          label="区間ループ"
          className={`loop ${props.loopEnabled ? "active" : ""}`}
          pressed={props.loopEnabled}
          onClick={props.onToggleLoop}
        >
          <Repeat2 size={17} aria-hidden="true" />
          <span>ループ</span>
        </PlayerButton>
      </div>
    </div>
  );
}

function PlayerButton({
  label,
  className = "",
  pressed,
  children,
  onClick,
}: {
  label: string;
  className?: string;
  pressed?: boolean;
  children: ReactNode;
  onClick(): void;
}) {
  return (
    <button
      type="button"
      className={`pbtn ${className}`.trim()}
      title={label}
      aria-label={label}
      aria-pressed={pressed}
      onClick={onClick}
    >
      {children}
    </button>
  );
}
