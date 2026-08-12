import {
  type CSSProperties,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useObjectUrl } from "../../../../shared/browser/use-object-url.js";
import { FrameNavigation } from "../../domain/frame-navigation.js";
import {
  clipRange,
  frameToSeconds,
  secondsToFrame,
  shouldLoopBack,
} from "../../domain/frame-time.js";
import type { SceneSelection } from "../../domain/scene-selection.js";
import { type PlaybackRate, stepPlaybackRate } from "../playback-rate.js";
import { useShortcutKeys } from "../use-shortcut-keys.js";

interface VideoControllerOptions {
  active: boolean;
  file: File;
  frameTimestamps: readonly number[];
  scene: SceneSelection | null;
}

export function useVideoController({
  active,
  file,
  frameTimestamps,
  scene,
}: VideoControllerOptions) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const previousSeconds = useRef(0);
  const source = useObjectUrl(file);
  const [loopEnabled, setLoopEnabled] = useState(true);
  const [playing, setPlaying] = useState(false);
  const [playbackRate, setPlaybackRate] = useState<PlaybackRate>(1);
  const [duration, setDuration] = useState(0);
  const [currentTime, setCurrentTime] = useState(0);
  const range = useMemo(
    () =>
      scene ? clipRange(scene.frame, scene.endFrame, frameTimestamps) : null,
    [frameTimestamps, scene],
  );

  useEffect(() => {
    if (!active) videoRef.current?.pause();
  }, [active]);

  useEffect(() => {
    const video = videoRef.current;
    if (!video || !range) return;
    previousSeconds.current = range.startSec;
    video.currentTime = range.startSec;
    void video.play().catch(() => undefined);
  }, [range]);

  const progressStyle = useMemo(() => {
    if (!range || duration <= 0) return undefined;
    const start = (range.startSec / duration) * 100;
    const end = (Math.min(range.endSec, duration) / duration) * 100;
    return {
      "--seek-track": `linear-gradient(to right, #333 ${start}%, #c47a00 ${start}% ${end}%, #333 ${end}%)`,
    } as CSSProperties;
  }, [duration, range]);

  const stepFrame = (delta: number) => {
    const video = videoRef.current;
    if (!video) return;
    video.pause();
    const currentFrame = secondsToFrame(video.currentTime, frameTimestamps);
    const maxFrame =
      frameTimestamps.length > 0
        ? frameTimestamps.length - 1
        : Number.POSITIVE_INFINITY;
    const targetFrame = clamp(currentFrame + delta, 0, maxFrame);
    const upperBound = Number.isFinite(video.duration)
      ? video.duration
      : Number.POSITIVE_INFINITY;
    video.currentTime = clamp(
      frameToSeconds(targetFrame, frameTimestamps),
      0,
      upperBound,
    );
  };

  const changePlaybackRate = (rate: PlaybackRate) => {
    setPlaybackRate(rate);
    if (videoRef.current) videoRef.current.playbackRate = rate;
  };

  const togglePlayback = () => {
    const video = videoRef.current;
    if (!video) return;
    if (video.paused) void video.play().catch(() => undefined);
    else video.pause();
  };

  const restartScene = () => {
    const video = videoRef.current;
    if (!video) return;
    const startSec = range?.startSec ?? 0;
    previousSeconds.current = startSec;
    video.currentTime = startSec;
  };

  useShortcutKeys(active, (action) => {
    switch (action.type) {
      case "frame":
        stepFrame(FrameNavigation.delta(action.move));
        return true;
      case "playback":
        togglePlayback();
        return true;
      case "loop":
        setLoopEnabled((enabled) => !enabled);
        return true;
      case "rate":
        changePlaybackRate(stepPlaybackRate(playbackRate, action.direction));
        return true;
      case "sceneStart":
        restartScene();
        return true;
      // 認識デバッグだけが持つ操作。ここでは既定動作を止めない。
      default:
        return false;
    }
  });

  return {
    videoRef,
    source,
    state: {
      currentTime,
      duration,
      loopEnabled,
      playbackRate,
      playing,
      progressStyle,
    },
    controls: {
      seek(milliseconds: number) {
        if (videoRef.current)
          videoRef.current.currentTime = milliseconds / 1000;
      },
      stepFrame,
      changePlaybackRate,
      toggleLoop() {
        setLoopEnabled((enabled) => !enabled);
      },
      togglePlayback,
    },
    events: {
      loadedMetadata(video: HTMLVideoElement) {
        setDuration(video.duration || 0);
        video.playbackRate = playbackRate;
        if (!range) return;
        previousSeconds.current = range.startSec;
        video.currentTime = range.startSec;
        if (active) void video.play().catch(() => undefined);
      },
      seeking(video: HTMLVideoElement) {
        previousSeconds.current = video.currentTime;
      },
      timeUpdate(video: HTMLVideoElement) {
        const time = video.currentTime;
        if (shouldLoopBack(loopEnabled, range, previousSeconds.current, time)) {
          video.currentTime = range?.startSec ?? 0;
          return;
        }
        previousSeconds.current = time;
        setCurrentTime(time);
      },
      play() {
        setPlaying(true);
      },
      pause() {
        setPlaying(false);
      },
    },
  };
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.max(minimum, Math.min(maximum, value));
}
