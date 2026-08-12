import { PLAYBACK_RATES, type PlaybackRate } from "./playback-rate.js";

/** 再生速度の選択。動画プレイヤーと認識デバッグで同じ体裁にする。 */
export function PlaybackRateControls({
  rate,
  onChange,
}: {
  rate: PlaybackRate;
  onChange(rate: PlaybackRate): void;
}) {
  return (
    <fieldset className="playback-rate-controls">
      <legend className="playback-rate-label">速度</legend>
      {PLAYBACK_RATES.map((candidate) => (
        <button
          key={candidate}
          type="button"
          className={`pbtn speed ${rate === candidate ? "active" : ""}`}
          title={`再生速度 ${candidate}倍`}
          aria-label={`再生速度 ${candidate}倍`}
          aria-pressed={rate === candidate}
          onClick={() => onChange(candidate)}
        >
          <span>{candidate}×</span>
        </button>
      ))}
    </fieldset>
  );
}
