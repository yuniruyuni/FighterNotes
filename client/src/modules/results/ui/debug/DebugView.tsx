import { Pause, Play, Save } from "lucide-react";
import type { Ref } from "react";
import type {
  AnalysisResult,
  AnalysisSide,
} from "~/modules/analysis/contracts.js";
import {
  FrameNavigation,
  type FrameNavigationAction,
} from "../../domain/frame-navigation.js";
import { PlaybackRateControls } from "../PlaybackRateControls.js";
import { ShortcutLegend } from "../ShortcutLegend.js";
import {
  DEBUG_SHORTCUT_HELP,
  FRAME_SHORTCUT_HELP,
  PLAYBACK_SHORTCUT_HELP,
} from "../shortcuts.js";
import { useDebugViewer } from "./use-debug-viewer.js";

interface DebugViewProps {
  active: boolean;
  focusRef?: Ref<HTMLButtonElement>;
  file: File;
  result: AnalysisResult;
  side: AnalysisSide;
}

export function DebugView(props: DebugViewProps) {
  const {
    canvasRef,
    frameInfo,
    visibility,
    setOverlayVisibility,
    navigate,
    playing,
    playbackRate,
    togglePlayback,
    changePlaybackRate,
    saveCurrentFrame,
    saveCurrentFrameData,
    loading,
    error,
  } = useDebugViewer(props);
  return (
    <section
      id="view-debug"
      aria-labelledby="debug-view-heading"
      hidden={!props.active}
      inert={!props.active}
      style={{ display: props.active ? "flex" : "none" }}
    >
      <h2 id="debug-view-heading" className="visually-hidden">
        認識デバッグ
      </h2>
      <div className="debug-canvas-area">
        {(loading || error) && (
          <div className={error ? "debug-error" : "debug-loading"}>
            {error || "解析データを読み込み中…"}
          </div>
        )}
        <canvas ref={canvasRef} id="debug-canvas" />
      </div>
      <div className="debug-ui">
        <div className="debug-controls">
          <DebugStepButton
            action="jump-backward"
            buttonRef={props.focusRef}
            onNavigate={navigate}
          />
          <DebugStepButton action="skip-backward" onNavigate={navigate} />
          <DebugStepButton action="step-backward" onNavigate={navigate} />
          <DebugStepButton action="step-forward" onNavigate={navigate} />
          <DebugStepButton action="skip-forward" onNavigate={navigate} />
          <DebugStepButton action="jump-forward" onNavigate={navigate} />
          <button
            type="button"
            className="pbtn play"
            title={playing ? "一時停止" : "再生"}
            aria-label={playing ? "一時停止" : "再生"}
            onClick={togglePlayback}
          >
            {playing ? (
              <Pause size={17} aria-hidden="true" />
            ) : (
              <Play size={17} aria-hidden="true" />
            )}
          </button>
          <PlaybackRateControls
            rate={playbackRate}
            onChange={changePlaybackRate}
          />
          <button
            type="button"
            className="pbtn"
            title="表示中のフレームを画像で保存"
            aria-label="表示中のフレームを画像で保存"
            onClick={saveCurrentFrame}
          >
            <Save size={17} aria-hidden="true" />
            <span>画像</span>
          </button>
          <button
            type="button"
            className="pbtn"
            title="表示中のフレームデータを保存"
            aria-label="表示中のフレームデータを保存"
            onClick={saveCurrentFrameData}
          >
            <Save size={17} aria-hidden="true" />
            <span>データ</span>
          </button>
          <span className="player-time">{frameInfo}</span>
        </div>
        <div className="debug-toggles">
          <label>
            <input
              type="checkbox"
              checked={visibility.raw}
              onChange={(event) =>
                setOverlayVisibility("raw", event.currentTarget.checked)
              }
            />{" "}
            メーター生読み
          </label>
          <label>
            <input
              type="checkbox"
              checked={visibility.hue}
              onChange={(event) =>
                setOverlayVisibility("hue", event.currentTarget.checked)
              }
            />{" "}
            Hue
          </label>
          <label>
            <input
              type="checkbox"
              checked={visibility.hp}
              onChange={(event) =>
                setOverlayVisibility("hp", event.currentTarget.checked)
              }
            />{" "}
            HP
          </label>
          <label>
            <input
              type="checkbox"
              checked={visibility.drive}
              onChange={(event) =>
                setOverlayVisibility("drive", event.currentTarget.checked)
              }
            />{" "}
            OD
          </label>
          <label>
            <input
              type="checkbox"
              checked={visibility.super}
              onChange={(event) =>
                setOverlayVisibility("super", event.currentTarget.checked)
              }
            />{" "}
            SA
          </label>
          <label>
            <input
              type="checkbox"
              checked={visibility.input}
              onChange={(event) =>
                setOverlayVisibility("input", event.currentTarget.checked)
              }
            />{" "}
            入力履歴
          </label>
          <label>
            <input
              type="checkbox"
              checked={visibility.attackInfo}
              onChange={(event) =>
                setOverlayVisibility("attackInfo", event.currentTarget.checked)
              }
            />{" "}
            攻撃情報
          </label>
        </div>
        <ShortcutLegend
          entries={[
            ...FRAME_SHORTCUT_HELP,
            ...PLAYBACK_SHORTCUT_HELP,
            ...DEBUG_SHORTCUT_HELP,
          ]}
        />
      </div>
    </section>
  );
}

function DebugStepButton({
  action,
  buttonRef,
  onNavigate,
}: {
  action: FrameNavigationAction;
  buttonRef?: Ref<HTMLButtonElement>;
  onNavigate(action: FrameNavigationAction): void;
}) {
  const delta = FrameNavigation.delta(action);
  const frames = Math.abs(delta);
  const backward = delta < 0;
  const label = `${frames}フレーム${backward ? "戻る" : "進む"}`;
  const arrow = frames === 1 ? (backward ? "‹" : "›") : backward ? "«" : "»";
  const text = backward ? `${arrow}${frames}f` : `${frames}f${arrow}`;
  return (
    <button
      ref={buttonRef}
      type="button"
      className="pbtn"
      aria-label={label}
      title={label}
      onClick={() => onNavigate(action)}
    >
      {text}
    </button>
  );
}
