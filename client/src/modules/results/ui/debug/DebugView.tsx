import type {
  AnalysisResult,
  AnalysisSide,
} from "~/modules/analysis/contracts.js";
import {
  DebugFrameNavigation,
  type DebugFrameNavigationAction,
} from "../../domain/debug-frame-navigation.js";
import { useDebugViewer } from "./use-debug-viewer.js";

interface DebugViewProps {
  active: boolean;
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
    loading,
    error,
  } = useDebugViewer(props);
  return (
    <div id="view-debug" style={{ display: props.active ? "flex" : "none" }}>
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
          <DebugStepButton action="jump-backward" onNavigate={navigate} />
          <DebugStepButton action="skip-backward" onNavigate={navigate} />
          <DebugStepButton action="step-backward" onNavigate={navigate} />
          <DebugStepButton action="step-forward" onNavigate={navigate} />
          <DebugStepButton action="skip-forward" onNavigate={navigate} />
          <DebugStepButton action="jump-forward" onNavigate={navigate} />
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
              checked={visibility.input}
              onChange={(event) =>
                setOverlayVisibility("input", event.currentTarget.checked)
              }
            />{" "}
            入力履歴
          </label>
        </div>
      </div>
    </div>
  );
}

function DebugStepButton({
  action,
  onNavigate,
}: {
  action: DebugFrameNavigationAction;
  onNavigate(action: DebugFrameNavigationAction): void;
}) {
  const delta = DebugFrameNavigation.delta(action);
  const frames = Math.abs(delta);
  const backward = delta < 0;
  const label = `${frames}フレーム${backward ? "戻る" : "進む"}`;
  const arrow = frames === 1 ? (backward ? "‹" : "›") : backward ? "«" : "»";
  const text = backward ? `${arrow}${frames}f` : `${frames}f${arrow}`;
  return (
    <button
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
