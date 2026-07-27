import { CHARACTER_CATALOG } from "../../domain/character.js";
import type { AnalysisSide } from "../../domain/context.js";

interface AnalysisSettingsProps {
  side: AnalysisSide;
  ownCharacter: string;
  opponentCharacter: string;
  busy: boolean;
  canAnalyze: boolean;
  unavailableReason?: string;
  onSideChange(side: AnalysisSide): void;
  onOwnCharacterChange(character: string): void;
  onOpponentCharacterChange(character: string): void;
  onSubmit(): void;
}

export function AnalysisSettings(props: AnalysisSettingsProps) {
  return (
    <div className="card">
      <h2>設定</h2>
      <div className="row">
        <div className="field">
          <label htmlFor="side-select">自分のサイド</label>
          <select
            id="side-select"
            value={props.side}
            disabled={props.busy}
            onChange={(event) =>
              props.onSideChange(event.currentTarget.value as AnalysisSide)
            }
          >
            <option value="p1">1P（左）</option>
            <option value="p2">2P（右）</option>
          </select>
        </div>
        <CharacterSelect
          id="char-select"
          label="自分のキャラクター（必須・確反提案に使用）"
          value={props.ownCharacter}
          disabled={props.busy}
          onChange={props.onOwnCharacterChange}
        />
        <CharacterSelect
          id="opponent-char-select"
          label="相手のキャラクター（必須・状況判定に使用）"
          value={props.opponentCharacter}
          disabled={props.busy}
          onChange={props.onOpponentCharacterChange}
        />
      </div>
      {props.unavailableReason && (
        <p
          className="analysis-warning"
          id="analysis-runtime-warning"
          role="alert"
        >
          {props.unavailableReason}
        </p>
      )}
      <button
        type="button"
        className="analyze-btn"
        aria-describedby={
          props.unavailableReason ? "analysis-runtime-warning" : undefined
        }
        disabled={!props.canAnalyze}
        onClick={props.onSubmit}
      >
        {props.busy ? "解析中…" : "解析する"}
      </button>
    </div>
  );
}

function CharacterSelect({
  id,
  label,
  value,
  disabled,
  onChange,
}: {
  id: string;
  label: string;
  value: string;
  disabled: boolean;
  onChange(character: string): void;
}) {
  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      <select
        id={id}
        required
        value={value}
        disabled={disabled}
        onChange={(event) => onChange(event.currentTarget.value)}
      >
        <option value="">選択してください</option>
        {CHARACTER_CATALOG.map(({ id: character, label: displayName }) => (
          <option key={character} value={character}>
            {displayName}
          </option>
        ))}
      </select>
    </div>
  );
}
