import type { SceneSelection } from "../../domain/scene-selection.js";
import { CardAdvice } from "./CardAdvice.js";
import { useVideoController } from "./use-video-controller.js";
import { VideoPlayerControls } from "./VideoPlayerControls.js";

interface VideoViewProps {
  active: boolean;
  file: File;
  frameTimestamps: readonly number[];
  scene: SceneSelection | null;
  onSceneChange(scene: Omit<SceneSelection, "key">): void;
}

export function VideoView(props: VideoViewProps) {
  const controller = useVideoController(props);
  const { videoRef, source, state, controls, events } = controller;
  return (
    <div id="view-video" style={{ display: props.active ? "flex" : "none" }}>
      <div className="video-area">
        <video
          id="player-video"
          ref={videoRef}
          src={source}
          preload="metadata"
          onLoadedMetadata={(event) =>
            events.loadedMetadata(event.currentTarget)
          }
          onSeeking={(event) => events.seeking(event.currentTarget)}
          onTimeUpdate={(event) => events.timeUpdate(event.currentTarget)}
          onPlay={events.play}
          onPause={events.pause}
          onEnded={events.pause}
        />
      </div>
      <VideoPlayerControls
        {...state}
        frameTimestamps={props.frameTimestamps}
        onSeek={controls.seek}
        onStepFrame={controls.stepFrame}
        onToggleLoop={controls.toggleLoop}
        onTogglePlayback={controls.togglePlayback}
      />
      <div className="clip-advice">
        {props.scene?.card ? (
          <CardAdvice
            card={props.scene.card}
            frameTimestamps={props.frameTimestamps}
            onSceneChange={props.onSceneChange}
          />
        ) : (
          props.scene?.label
        )}
      </div>
    </div>
  );
}
